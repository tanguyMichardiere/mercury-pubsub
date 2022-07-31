use crate::api::error_handlers::handle_database_error;
use crate::api::extractors::refresh_token::REFRESH_TOKEN_COOKIE_NAME;
use crate::models::user::User;
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::{async_trait, Extension};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use hyper::{header, HeaderMap};
use serde::Serialize;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use tokio::try_join;
use tracing::debug;
use uuid::Uuid;

/// A session as they are stored in the database.
struct RawSession {
    id: Uuid,
    expires: DateTime<Utc>,
    user_id: Uuid,
}

/// A session augmented with its user and sometimes its access and/or refresh token.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(skip_serializing)]
    pub id: Uuid,
    /// Some if just created or refreshed, None otherwise
    pub access_token: Option<String>,
    /// Some if just created or refreshed, None otherwise
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    pub expires: DateTime<Utc>,
    pub user: User,
}

impl Session {
    /// Generate 48 random bytes converted to base64 (64 characters) with the database
    async fn gen_token(pool: &PgPool) -> sqlx::Result<String> {
        sqlx::query_scalar!(r#"SELECT encode(gen_random_bytes(48), 'base64')"#)
            .fetch_one(pool)
            .await
            .map(|option| option.expect("NULL from SELECT scalar"))
    }

    /// Create a session from a RawSession, optionally also setting the access and refresh token
    /// fields.
    async fn from_raw_session(
        raw_session: RawSession,
        pool: &PgPool,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
    ) -> sqlx::Result<Self> {
        Ok(Self {
            id: raw_session.id,
            access_token: access_token.map(|access_token| access_token.to_owned()),
            refresh_token: refresh_token.map(|refresh_token| refresh_token.to_owned()),
            expires: raw_session.expires,
            user: User::get(pool, raw_session.user_id)
                .await?
                .expect("session without user"),
        })
    }

    /// Create a new session for this user, with an access token and a refresh token, expiring in 1
    /// day.
    pub async fn new(pool: &PgPool, user: User) -> sqlx::Result<Self> {
        let (access_token, refresh_token) =
            try_join!(Self::gen_token(pool), Self::gen_token(pool))?;
        let raw_session = sqlx::query_as!(
            RawSession,
            r#"
            INSERT INTO "Session" (access_token_hash, refresh_token_hash, expires, user_id)
                VALUES (crypt($1, gen_salt('md5')), crypt($2, gen_salt('md5')), CURRENT_TIMESTAMP + interval '1 day', $3)
            RETURNING id, expires, user_id
            "#,
            access_token,
            refresh_token,
            user.id
        )
        .fetch_one(pool)
        .await?;
        Ok(Self {
            id: raw_session.id,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            expires: raw_session.expires,
            user,
        })
    }

    /// Get a session with an access token.
    ///
    /// The session returned won't have the access and refresh token fields.
    pub async fn get(pool: &PgPool, access_token: &str) -> sqlx::Result<Option<Self>> {
        Self::delete_expired(pool).await?;
        if let Some(raw_session) = sqlx::query_as!(
            RawSession,
            r#"
            SELECT id, expires, user_id FROM "Session"
                WHERE access_token_hash = crypt($1, access_token_hash)
            "#,
            access_token
        )
        .fetch_optional(pool)
        .await?
        {
            Ok(Some(
                Self::from_raw_session(raw_session, pool, None, None).await?,
            ))
        } else {
            Ok(None)
        }
    }

    /// Refresh a session with a refresh token, issuing a new access token and a new expiration date
    /// for the refresh token.
    pub async fn refresh(pool: &PgPool, refresh_token: &str) -> sqlx::Result<Option<Self>> {
        Self::delete_expired(pool).await?;
        let access_token = Self::gen_token(pool).await?;
        if let Some(raw_session) = sqlx::query_as!(
            RawSession,
            r#"
            UPDATE "Session"
                SET access_token_hash = crypt($1, gen_salt('md5')),
                    expires = CURRENT_TIMESTAMP + interval '1 day'
                WHERE refresh_token_hash = crypt($2, refresh_token_hash)
            RETURNING id, expires, user_id
            "#,
            access_token,
            refresh_token
        )
        .fetch_optional(pool)
        .await?
        {
            Ok(Some(
                Self::from_raw_session(raw_session, pool, Some(&access_token), Some(refresh_token))
                    .await?,
            ))
        } else {
            Ok(None)
        }
    }

    /// Delete a session with a refresh token.
    pub async fn delete(pool: &PgPool, refresh_token: &str) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"
            DELETE FROM "Session"
                WHERE refresh_token_hash = crypt($1, refresh_token_hash)
            "#,
            refresh_token
        )
        .execute(pool)
        .await
    }

    /// Delete all expired sessions from the database.
    async fn delete_expired(pool: &PgPool) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"
            DELETE FROM "Session"
                WHERE expires < CURRENT_TIMESTAMP
            "#
        )
        .execute(pool)
        .await
    }
}

#[async_trait]
impl<B> FromRequest<B> for Session
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // read the Authorization header, expecting a Bearer token
        let access_token = TypedHeader::<Authorization<Bearer>>::from_request(req)
            .await
            .map_err(|error| {
                debug!(?error, "missing Authorization header");
                (StatusCode::UNAUTHORIZED, "Missing Authorization header")
            })?
            .token()
            .to_owned();
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("pool extension missing");
        // get the session for this access token
        Session::get(&pool, &access_token)
            .await
            .map_err(handle_database_error)?
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "No session found with this refresh token",
            ))
    }
}

impl IntoResponse for Session {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        if let Some(refresh_token) = &self.refresh_token {
            // if the session has a refresh token, send it as a cookie
            headers.insert(
                header::SET_COOKIE,
                format!(
                    "{REFRESH_TOKEN_COOKIE_NAME}={}; Expires={}; SameSite=Strict; Secure; HttpOnly; Path=/",
                    refresh_token,
                    self.expires.to_rfc2822()
                )
                .parse()
                .expect("wrong formatting"),
            );
        }
        (headers, Json(self)).into_response()
    }
}
