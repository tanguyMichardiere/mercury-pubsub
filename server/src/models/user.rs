use error::{Error, Result};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// An application user.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

impl User {
    /// Create a new user.
    pub async fn new(pool: &PgPool, name: &str, password: &str) -> Result<Self> {
        Ok(sqlx::query_as!(
            User,
            r#"
            INSERT INTO "User" (name, password_hash)
                VALUES ($1, crypt($2, gen_salt('md5')))
            RETURNING *
            "#,
            name,
            password
        )
        .fetch_one(pool)
        .await?)
    }

    /// Get the number of existing users.
    pub async fn count(pool: &PgPool) -> Result<i64> {
        Ok(sqlx::query_scalar!(r#"SELECT COUNT(*) FROM "User""#)
            .fetch_one(pool)
            .await
            .map(|option| option.expect("NULL from SELECT COUNT"))?)
    }

    /// Get a user with an ID.
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Self> {
        sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "User"
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(Error::NotFound)
    }

    /// Get a user with a name and password.
    pub async fn get_by_name_and_password(
        pool: &PgPool,
        name: &str,
        password: &str,
    ) -> Result<Self> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "User"
                WHERE name = $1
            "#,
            name
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
}

pub mod error {
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::error;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("User not found")]
        NotFound,
        #[error("Wrong password")]
        WrongPassword,
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
                Error::NotFound => (StatusCode::NOT_FOUND, self).into_response(),
                Error::WrongPassword => (StatusCode::UNAUTHORIZED, self).into_response(),
            }
        }
    }
}
