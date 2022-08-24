use axum::extract::{FromRequest, RequestParts};
use axum::headers::authorization::Basic;
use axum::headers::Authorization;
use axum::{async_trait, Extension, TypedHeader};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use error::{Error, Result};

/// An application user.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    id: Uuid,
    name: String,
    #[serde(skip_serializing)]
    password_hash: String,
    #[serde(skip_serializing)]
    pub(crate) rank: i32,
}

/// CRUD
impl User {
    /// Create a new user.
    pub(crate) async fn new(pool: &PgPool, name: &str, password: &str, rank: i32) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO "User" (name, password_hash, rank)
                VALUES ($1, crypt($2, gen_salt('md5')), $3)
            RETURNING *
            "#,
            name,
            password,
            rank,
        )
        .fetch_one(pool)
        .await?)
    }

    /// Get a user.
    pub(crate) async fn get(pool: &PgPool, id: Uuid) -> Result<Self> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM "User"
                WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(Error::NotFound)
    }

    /// Get a user by its name and password.
    pub(crate) async fn get_by_name_and_password(
        pool: &PgPool,
        name: &str,
        password: &str,
    ) -> Result<Self> {
        let user = sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM "User"
                WHERE name = $1
            "#,
            name,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(Error::NotFound)?;
        // check if the password is correct
        if sqlx::query_scalar!(r#"SELECT $1 = crypt($2, $1)"#, user.password_hash, password)
            .fetch_one(pool)
            .await
            .map(|option| option.expect("NULL from SELECT scalar"))?
        {
            Ok(user)
        } else {
            Err(Error::WrongPassword)
        }
    }

    /// Get all users whose rank is greater than or equal to `min_rank`.
    pub(crate) async fn get_all(pool: &PgPool, min_rank: i32) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM "User"
                WHERE rank >= $1
            "#,
            min_rank,
        )
        .fetch_all(pool)
        .await?)
    }

    /// Rename the user.
    pub(crate) async fn rename(&mut self, pool: &PgPool, name: &str) -> Result<()> {
        self.name = sqlx::query_scalar!(
            r#"
            UPDATE "User"
                SET name = $1
                WHERE id = $2
            RETURNING name
            "#,
            name,
            self.id,
        )
        .fetch_one(pool)
        .await?;
        Ok(())
    }

    /// Change the user's password.
    pub(crate) async fn change_password(&mut self, pool: &PgPool, password: &str) -> Result<()> {
        self.password_hash = sqlx::query_scalar!(
            r#"
            UPDATE "User"
                SET password_hash = crypt($1, gen_salt('md5'))
                WHERE id = $2
            RETURNING password_hash
            "#,
            password,
            self.id,
        )
        .fetch_one(pool)
        .await?;
        Ok(())
    }

    /// Delete the user.
    pub(crate) async fn delete(self, pool: &PgPool) -> Result<()> {
        if self.rank == 0 {
            panic!("cannot delete root user");
        }
        sqlx::query!(
            r#"
                DELETE FROM "User"
                    WHERE id = $1
                "#,
            self.id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = Error;

    async fn from_request(req: &mut RequestParts<B>) -> std::result::Result<Self, Self::Rejection> {
        let authorization_header = TypedHeader::<Authorization<Basic>>::from_request(req).await?;
        let pool = Extension::<PgPool>::from_request(req)
            .await
            .expect("missing pool extension");
        Ok(Self::get_by_name_and_password(
            &pool,
            authorization_header.username(),
            authorization_header.password(),
        )
        .await?)
    }
}

pub(crate) mod error {
    use axum::extract::rejection::TypedHeaderRejection;
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::{debug, error};

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        #[error(transparent)]
        TypedHeaderRejection(#[from] TypedHeaderRejection),
        #[error("User not found")]
        NotFound,
        #[error("Wrong password")]
        WrongPassword,
        #[error("Duplicate user name")]
        DuplicateName,
    }

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            if let Some(database_error) = error.as_database_error() {
                if database_error.constraint() == Some("User_name_key") {
                    return Self::DuplicateName;
                }
            }
            error!(?error);
            panic!("unknown database error");
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Error::TypedHeaderRejection(error) => error.into_response(),
                Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
                Error::WrongPassword => {
                    (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
                }
                Error::DuplicateName => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            }
        }
    }
}
