use jsonschema::{ErrorIterator, JSONSchema};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::key::Key;

use error::{Error, Result};

pub(crate) struct RawChannel {
    id: Uuid,
    name: String,
    schema: Value,
}

#[derive(Serialize)]
pub(crate) struct Channel {
    pub(crate) id: Uuid,
    name: String,
    schema: Value,
    #[serde(skip_serializing)]
    compiled_schema: JSONSchema,
}

/// CRUD
impl Channel {
    /// Create a `Channel` from a `RawChannel`.
    fn from_raw_channel(raw_channel: RawChannel) -> Self {
        Self {
            id: raw_channel.id,
            name: raw_channel.name,
            compiled_schema: JSONSchema::compile(&raw_channel.schema)
                .expect("invalid schema in database"),
            schema: raw_channel.schema,
        }
    }

    /// Create a new channel.
    pub(crate) async fn new(pool: &PgPool, name: &str, schema: &Value) -> Result<Self> {
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
                schema,
            )
            .fetch_one(pool)
            .await?,
        ))
    }

    /// Get a channel.
    pub(crate) async fn get(pool: &PgPool, id: Uuid) -> Result<Self> {
        Ok(Self::from_raw_channel(
            sqlx::query_as!(
                RawChannel,
                r#"
                SELECT * FROM "Channel"
                    WHERE id = $1
                "#,
                id,
            )
            .fetch_optional(pool)
            .await?
            .ok_or(Error::NotFound)?,
        ))
    }

    /// Get a channel by its name.
    pub(crate) async fn get_by_name(pool: &PgPool, name: &str) -> Result<Self> {
        Ok(Self::from_raw_channel(
            sqlx::query_as!(
                RawChannel,
                r#"
                SELECT * FROM "Channel"
                    WHERE name = $1
                "#,
                name,
            )
            .fetch_optional(pool)
            .await?
            .ok_or(Error::NotFound)?,
        ))
    }

    /// Get all channels.
    pub(crate) async fn get_all(pool: &PgPool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(RawChannel, r#"SELECT * FROM "Channel""#)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(Self::from_raw_channel)
            .collect())
    }

    /// Get channels by their key.
    pub(crate) async fn get_from_key(pool: &PgPool, key: &Key) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            RawChannel,
            r#"
            SELECT id, name, schema FROM "Channel"
                JOIN "Access"
                    ON "Channel".id = "Access".channel_id
                WHERE key_id = $1
            "#,
            key.id,
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(Self::from_raw_channel)
        .collect())
    }

    /// Delete the channel.
    pub(crate) async fn delete(self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM "Channel"
                WHERE id = $1
            "#,
            self.id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl Channel {
    /// Run validation on the instance, only returning a boolean indicating success or failure.
    pub(crate) fn is_valid(&self, instance: &Value) -> bool {
        self.compiled_schema.is_valid(instance)
    }

    /// Validate the instance. This is slower than `Channel::is_valid`.
    pub(crate) fn validate<'a>(
        &'a self,
        instance: &'a Value,
    ) -> std::result::Result<(), ErrorIterator<'a>> {
        self.compiled_schema.validate(instance)
    }
}

pub(crate) mod error {
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use jsonschema::ValidationError;
    use tracing::{debug, error};

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        // TODO: include information about the error
        #[error("Invalid schema")]
        InvalidSchema,
        #[error("Channel not found")]
        NotFound,
        #[error("Duplicate channel name")]
        DuplicateName,
    }

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            if let Some(database_error) = error.as_database_error() {
                if database_error.constraint() == Some("Channel_name_key") {
                    return Self::DuplicateName;
                }
            }
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
            debug!(?self);
            match self {
                Error::InvalidSchema => {
                    (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
                }
                Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
                Error::DuplicateName => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            }
        }
    }
}
