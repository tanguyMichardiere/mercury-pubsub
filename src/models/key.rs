use crate::models::channel::Channel;
use axum::extract::{FromRequest, RequestParts};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::{async_trait, Extension, TypedHeader};
use error::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "keytype")]
#[sqlx(rename_all = "lowercase")]
pub enum KeyType {
    Publisher,
    Subscriber,
}

pub struct Secret(String);

impl Secret {
    async fn new(pool: &PgPool) -> Result<Self> {
        Ok(Self(
            sqlx::query_scalar!(r#"SELECT encode(gen_random_bytes(48), 'base64')"#)
                .fetch_one(pool)
                .await
                .map(|option| option.expect("NULL from SELECT scalar"))?,
        ))
    }
}

impl Deref for Secret {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize)]
pub struct Key {
    id: Uuid,
    r#type: KeyType,
    #[serde(skip_serializing)]
    hash: String,
}

impl Key {
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub async fn new(pool: &PgPool, r#type: KeyType) -> Result<(Self, Secret)> {
        let secret = Secret::new(pool).await?;
        let key = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO "Key" (type, hash)
                VALUES ($1, crypt($2, gen_salt('md5')))
            RETURNING id, type as "type: _", hash
            "#,
            r#type as KeyType,
            secret.deref()
        )
        .fetch_one(pool)
        .await?;
        Ok((key, secret))
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            SELECT id, type as "type: _", hash FROM "Key"
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn check_secret(&self, pool: &PgPool, secret: &Secret) -> Result<bool> {
        Ok(
            sqlx::query_scalar!(r#"SELECT $1 = crypt($2, $1)"#, self.hash, secret.deref())
                .fetch_one(pool)
                .await
                .map(|option| option.expect("NULL from SELECT scalar"))?,
        )
    }

    pub async fn get_all(pool: &PgPool) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as!(Self, r#"SELECT id, type as "type: _", hash FROM "Key""#)
                .fetch_all(pool)
                .await?,
        )
    }

    pub async fn set_channels(&self, pool: &PgPool, ids: Vec<Uuid>) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM "Access"
                WHERE key_id = $1
            "#,
            self.id
        )
        .execute(pool)
        .await?;
        let mut query =
            QueryBuilder::<Postgres>::new(r#"INSERT INTO "Access" (key_id, channel_id) "#);
        query.push_values(ids, |mut b, channel_id| {
            b.push_bind(self.id).push_bind(channel_id);
        });
        query.build().execute(pool).await?;
        Ok(())
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            DELETE FROM "Key"
                WHERE id = $1
            RETURNING id, type as "type: _", hash
            "#,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    pub fn is_publisher(&self) -> bool {
        matches!(self.r#type, KeyType::Publisher)
    }

    pub fn is_subscriber(&self) -> bool {
        matches!(self.r#type, KeyType::Subscriber)
    }

    pub async fn authorizes(&self, pool: &PgPool, channel: &Channel) -> Result<bool> {
        Ok(sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM "Access"
                WHERE key_id = $1 AND channel_id = $2
            "#,
            self.id,
            channel.get_id()
        )
        .fetch_one(pool)
        .await?
        .expect("NULL from SELECT scalar")
            > 0)
    }
}

#[async_trait]
impl<B> FromRequest<B> for Key
where
    B: axum::body::HttpBody + Send,
{
    type Rejection = Error;

    /// Parse the Authorization header, expecting a token containing the key's id and secret
    /// separated by a semi-colon and returning the key directly
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self> {
        let authorization_header_value =
            TypedHeader::<Authorization<Bearer>>::from_request(req).await?;
        let token = authorization_header_value.token();
        let (id, secret) = token.split_once(";").ok_or(Error::MissingSemiColon)?;
        let id = Uuid::try_parse(id).map_err(|_| Error::InvalidKeyId)?;
        let secret = Secret(secret.to_owned());
        let Extension(pool) = Extension::<PgPool>::from_request(req)
            .await
            .expect("missing pool extension");
        let key = Key::get(&pool, id).await?;
        if key.check_secret(&pool, &secret).await? {
            Ok(key)
        } else {
            Err(Error::InvalidSecretKey)
        }
    }
}

pub mod error {
    use axum::extract::rejection::TypedHeaderRejection;
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::error;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        TypedHeaderRejection(#[from] TypedHeaderRejection),
        #[error("Missing semi-colon in API key")]
        MissingSemiColon,
        #[error("Invalid API key ID")]
        InvalidKeyId,
        #[error("Invalid secret key")]
        InvalidSecretKey,
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
                Error::TypedHeaderRejection(error) => error.into_response(),
                Error::MissingSemiColon => (StatusCode::UNAUTHORIZED, self).into_response(),
                Error::InvalidKeyId => (StatusCode::UNAUTHORIZED, self).into_response(),
                Error::InvalidSecretKey => (StatusCode::UNAUTHORIZED, self).into_response(),
            }
        }
    }
}
