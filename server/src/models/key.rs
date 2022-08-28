use axum::extract::{FromRef, FromRequestParts};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::{async_trait, TypedHeader};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::channel::Channel;
use crate::state::SharedState;

use error::{Error, Result};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "keytype")]
#[sqlx(rename_all = "lowercase")]
pub(crate) enum KeyType {
    Publisher,
    Subscriber,
}

pub(crate) struct Secret(String);

/// CRUD
impl Secret {
    /// Create a new secret.
    async fn new(pool: &PgPool) -> Result<Self> {
        Ok(Self(
            sqlx::query_scalar!(r#"SELECT encode(gen_random_bytes(48), 'base64')"#)
                .fetch_one(pool)
                .await
                .map(|option| option.expect("NULL from SELECT scalar"))?,
        ))
    }
}

impl AsRef<str> for Secret {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Key {
    pub(crate) id: Uuid,
    r#type: KeyType,
    #[serde(skip_serializing)]
    hash: String,
}

/// CRUD
impl Key {
    /// Create a new key.
    ///
    /// Returns the key and its secret. The secret will only be returned once, when the key is
    /// created.
    pub(crate) async fn new(
        pool: &PgPool,
        r#type: KeyType,
        channel_ids: Vec<Uuid>,
    ) -> Result<(Self, Secret)> {
        let secret = Secret::new(pool).await?;
        let key = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO "Key" (type, hash)
                VALUES ($1, crypt($2, gen_salt('md5')))
            RETURNING id, type as "type: _", hash
            "#,
            r#type as KeyType,
            secret.as_ref(),
        )
        .fetch_one(pool)
        .await?;
        let mut query =
            QueryBuilder::<Postgres>::new(r#"INSERT INTO "Access" (key_id, channel_id) "#);
        query.push_values(channel_ids, |mut b, channel_id| {
            b.push_bind(key.id).push_bind(channel_id);
        });
        query.build().execute(pool).await?;
        Ok((key, secret))
    }

    /// Get a key.
    pub(crate) async fn get(pool: &PgPool, id: Uuid) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            SELECT id, type as "type: _", hash FROM "Key"
                WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(Error::NotFound)?)
    }

    /// Check that the secret is the key's.
    async fn check_secret(&self, pool: &PgPool, secret: &Secret) -> Result<bool> {
        Ok(
            sqlx::query_scalar!(r#"SELECT $1 = crypt($2, $1)"#, self.hash, secret.as_ref())
                .fetch_one(pool)
                .await
                .map(|option| option.expect("NULL from SELECT scalar"))?,
        )
    }

    /// Get all keys.
    pub(crate) async fn get_all(pool: &PgPool) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as!(Self, r#"SELECT id, type as "type: _", hash FROM "Key""#)
                .fetch_all(pool)
                .await?,
        )
    }

    /// Delete the key.
    pub(crate) async fn delete(self, pool: &PgPool) -> Result<()> {
        sqlx::query_as!(
            Self,
            r#"
            DELETE FROM "Key"
                WHERE id = $1
            "#,
            self.id,
        )
        .fetch_one(pool)
        .await?;
        Ok(())
    }
}

impl Key {
    /// Returns whether the key is a publisher key.
    pub(crate) fn is_publisher(&self) -> bool {
        matches!(self.r#type, KeyType::Publisher)
    }

    /// Returns whether the key is a subscriber key.
    pub(crate) fn is_subscriber(&self) -> bool {
        matches!(self.r#type, KeyType::Subscriber)
    }

    /// Returns whether the key authorizes the channel.
    pub(crate) async fn authorizes(&self, pool: &PgPool, channel: &Channel) -> Result<bool> {
        Ok(sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM "Access"
                WHERE key_id = $1 AND channel_id = $2
            "#,
            self.id,
            channel.id,
        )
        .fetch_one(pool)
        .await?
        .expect("NULL from SELECT scalar")
            == 1)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Key
where
    S: Send + Sync,
    SharedState: FromRef<S>,
{
    type Rejection = Error;

    /// Parse the Authorization header, expecting a token containing the key's id and secret
    /// separated by a semi-colon and returning the key directly
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let authorization_header_value =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await?;
        let token = authorization_header_value.token();
        let (id, secret) = token.split_once(';').ok_or(Error::MissingSemiColon)?;
        let id = Uuid::try_parse(id).map_err(|_| Error::InvalidKeyId)?;
        let secret = Secret(secret.to_owned());
        let state = SharedState::from_ref(state);
        let key = Key::get(&state.read().await.pool, id).await?;
        if key.check_secret(&state.read().await.pool, &secret).await? {
            Ok(key)
        } else {
            Err(Error::InvalidSecretKey)
        }
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
        #[error("Missing semi-colon in API key")]
        MissingSemiColon,
        #[error("Invalid API key ID")]
        InvalidKeyId,
        #[error("Invalid secret key")]
        InvalidSecretKey,
        #[error("Key not found")]
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
            debug!(?self);
            match self {
                Error::TypedHeaderRejection(error) => error.into_response(),
                Error::MissingSemiColon => {
                    (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
                }
                Error::InvalidKeyId => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
                Error::InvalidSecretKey => {
                    (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
                }
                Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            }
        }
    }
}
