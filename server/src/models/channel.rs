use crate::models::key::Key;
use error::{Error, Result};
use jsonschema::{ErrorIterator, JSONSchema};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub struct RawChannel {
    id: Uuid,
    name: String,
    schema: Value,
}

#[derive(Serialize)]
pub struct Channel {
    id: Uuid,
    name: String,
    schema: Value,
    #[serde(skip_serializing)]
    compiled_schema: JSONSchema,
}

impl Channel {
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    fn from_raw_channel(raw_channel: RawChannel) -> Self {
        Self {
            id: raw_channel.id,
            name: raw_channel.name,
            compiled_schema: JSONSchema::compile(&raw_channel.schema)
                .expect("invalid schema in database"),
            schema: raw_channel.schema,
        }
    }

    pub async fn new(pool: &PgPool, name: &str, schema: &Value) -> Result<Self> {
        JSONSchema::compile(schema)?;
        Ok(Self::from_raw_channel(
            sqlx::query_as!(
                RawChannel,
                r#"
                INSERT INTO "Channel" (name, schema)
                    VALUES ($1, $2)
                RETURNING *
                "#,
                name,
                schema
            )
            .fetch_one(pool)
            .await?,
        ))
    }

    pub async fn get(pool: &PgPool, name: &str) -> Result<Self> {
        Ok(Self::from_raw_channel(
            sqlx::query_as!(
                RawChannel,
                r#"
                SELECT * FROM "Channel"
                    WHERE name = $1
                "#,
                name
            )
            .fetch_optional(pool)
            .await?
            .ok_or(Error::NotFound)?,
        ))
    }

    pub async fn get_all(pool: &PgPool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(RawChannel, r#"SELECT * FROM "Channel""#)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(Self::from_raw_channel)
            .collect())
    }

    pub async fn get_from_key(pool: &PgPool, key: &Key) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            RawChannel,
            r#"
            SELECT id, name, schema FROM "Channel"
                JOIN "Access"
                    ON "Channel".id = "Access".channel_id
                WHERE key_id = $1
            "#,
            key.get_id()
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(Self::from_raw_channel)
        .collect())
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        self.compiled_schema.is_valid(instance)
    }

    pub fn validate<'a>(
        &'a self,
        instance: &'a Value,
    ) -> std::result::Result<(), ErrorIterator<'a>> {
        self.compiled_schema.validate(instance)
    }
}

pub mod error {
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use jsonschema::ValidationError;
    use tracing::error;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        // TODO: include information about the error
        #[error("Invalid schema")]
        InvalidSchema,
        #[error("Channel not found")]
        NotFound,
    }

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            error!(?error);
            panic!("unknown database error");
        }
    }

    impl<'a> From<ValidationError<'a>> for Error {
        fn from(_: ValidationError<'a>) -> Self {
            Self::InvalidSchema
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Error::InvalidSchema => (StatusCode::UNPROCESSABLE_ENTITY, self).into_response(),
                Error::NotFound => (StatusCode::NOT_FOUND, self).into_response(),
            }
        }
    }
}
