use crate::models::access_token::AccessToken;
use crate::models::refresh_token::RefreshToken;
use crate::models::user::User;
use axum::extract::{FromRequest, RequestParts};
use axum::{async_trait, Extension};
use chrono::{DateTime, Utc};
use error::{Error, Result};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// A session as they are stored in the database.
struct RawSession {
    id: Uuid,
    expires: DateTime<Utc>,
    user_id: Uuid,
}

/// A session augmented with its user and sometimes its access token.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(skip_serializing)]
    pub id: Uuid,
    /// Some if just created or refreshed, None otherwise
    pub access_token: Option<AccessToken>,
    pub expires: DateTime<Utc>,
    pub user: User,
}

impl Session {
    /// Create a session from a RawSession, optionally also setting the access token field.
    async fn from_raw_session(
        raw_session: RawSession,
        pool: &PgPool,
        access_token: Option<AccessToken>,
    ) -> Result<Self> {
        Ok(Self {
            id: raw_session.id,
            access_token,
            expires: raw_session.expires,
            user: User::get(pool, raw_session.user_id).await?,
        })
    }

    /// Create a new session for this user, with an access token and a refresh token, expiring in 1
    /// day.
    pub async fn new(pool: &PgPool, user: User) -> Result<(Self, RefreshToken)> {
        let access_token = AccessToken::new(pool).await?;
        let refresh_token = RefreshToken::new(pool).await?;
        let raw_session = sqlx::query_as!(
            RawSession,
            r#"
            INSERT INTO "Session" (access_token_hash, refresh_token_hash, expires, user_id)
                VALUES (crypt($1, gen_salt('md5')), crypt($2, gen_salt('md5')), CURRENT_TIMESTAMP + interval '1 day', $3)
            RETURNING id, expires, user_id
            "#,
            access_token.as_ref(),
            refresh_token.as_ref(),
            user.id
        )
        .fetch_one(pool)
        .await?;
        Ok((
            Self {
                id: raw_session.id,
                access_token: Some(access_token),
                expires: raw_session.expires,
                user,
            },
            refresh_token,
        ))
    }

    /// Get a session with an access token.
    ///
    /// It won't have the access token field.
    pub async fn get(pool: &PgPool, access_token: &AccessToken) -> Result<Self> {
        Self::delete_expired(pool).await?;
        if let Some(raw_session) = sqlx::query_as!(
            RawSession,
            r#"
            SELECT id, expires, user_id FROM "Session"
                WHERE access_token_hash = crypt($1, access_token_hash)
            "#,
            access_token.as_ref()
        )
        .fetch_optional(pool)
        .await?
        {
            Ok(Self::from_raw_session(raw_session, pool, None).await?)
        } else {
            Err(Error::NotFound)
        }
    }

    /// Refresh a session with a refresh token, issuing a new access token and a new expiration date
    /// for the refresh token.
    pub async fn refresh(pool: &PgPool, refresh_token: &RefreshToken) -> Result<Self> {
        Self::delete_expired(pool).await?;
        let access_token = AccessToken::new(pool).await?;
        if let Some(raw_session) = sqlx::query_as!(
            RawSession,
            r#"
            UPDATE "Session"
                SET access_token_hash = crypt($1, gen_salt('md5')),
                    expires = CURRENT_TIMESTAMP + interval '1 day'
                WHERE refresh_token_hash = crypt($2, refresh_token_hash)
            RETURNING id, expires, user_id
            "#,
            access_token.as_ref(),
            refresh_token.as_ref()
        )
        .fetch_optional(pool)
        .await?
        {
            Ok(Self::from_raw_session(raw_session, pool, Some(access_token)).await?)
        } else {
            Err(Error::NotFound)
        }
    }

    /// Delete a session with a refresh token.
    pub async fn delete(pool: &PgPool, refresh_token: &RefreshToken) -> Result<()> {
        if sqlx::query!(
            r#"
            DELETE FROM "Session"
                WHERE refresh_token_hash = crypt($1, refresh_token_hash)
            "#,
            refresh_token.as_ref()
        )
        .execute(pool)
        .await?
        .rows_affected()
            == 1
        {
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    /// Delete all expired sessions from the database.
    async fn delete_expired(pool: &PgPool) -> Result<u64> {
        Ok(sqlx::query!(
            r#"
            DELETE FROM "Session"
                WHERE expires < CURRENT_TIMESTAMP
            "#
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}

#[async_trait]
impl<B> FromRequest<B> for Session
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self> {
        let access_token = AccessToken::from_request(req).await?;
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("pool extension missing");
        Session::get(&pool, &access_token).await
    }
}

pub mod error {
    use crate::models::{access_token, refresh_token, user};
    use axum::extract::rejection::TypedHeaderRejection;
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::error;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        UserError(#[from] user::error::Error),
        #[error(transparent)]
        TypedHeaderRejection(#[from] TypedHeaderRejection),
        #[error(transparent)]
        AccessTokenError(#[from] access_token::error::Error),
        #[error(transparent)]
        RefreshTokenError(#[from] refresh_token::error::Error),
        #[error("Session not found")]
        NotFound,
    }

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            error!(?error);
            panic!("unknown database error");
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::UserError(error) => error.into_response(),
                Error::TypedHeaderRejection(error) => error.into_response(),
                Error::AccessTokenError(error) => error.into_response(),
                Error::RefreshTokenError(error) => error.into_response(),
                Error::NotFound => (StatusCode::NOT_FOUND, self).into_response(),
            }
        }
    }
}
