use crate::constants::SESSION_COOKIE_NAME;
use crate::models::user::User;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use hyper::{header, HeaderMap};
use serde::Serialize;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: Uuid,
    pub session_token: String,
    pub expires: DateTime<Utc>,
    pub user_id: Uuid,
}

impl Session {
    pub async fn get(pool: &PgPool, session_token: &str) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"
                SELECT * FROM "Session"
                    WHERE "Session".session_token = $1
                        AND expires > CURRENT_TIMESTAMP
                "#,
            session_token
        )
        .fetch_one(pool)
        .await
    }

    pub async fn with_user(self, pool: &PgPool) -> sqlx::Result<SessionWithUser> {
        Ok(SessionWithUser {
            id: self.id,
            session_token: self.session_token,
            expires: self.expires,
            user: User::get(pool, self.user_id).await?,
        })
    }

    pub async fn extend(&mut self, pool: &PgPool) -> sqlx::Result<()> {
        self.expires = sqlx::query_scalar!(
            r#"
                UPDATE "Session"
                    SET expires = CURRENT_TIMESTAMP + interval '1 day'
                    WHERE id = $1
                RETURNING expires
                "#,
            self.id
        )
        .fetch_one(pool)
        .await?;
        Ok(())
    }

    pub async fn clean_expired(pool: &PgPool) -> sqlx::Result<PgQueryResult> {
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWithUser {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub session_token: String,
    pub expires: DateTime<Utc>,
    pub user: User,
}

impl SessionWithUser {
    pub async fn new(pool: &PgPool, user: User) -> sqlx::Result<Self> {
        let session = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO "Session" (expires, user_id)
                VALUES (CURRENT_TIMESTAMP + interval '1 day', $1)
            RETURNING *
            "#,
            user.id
        )
        .fetch_one(pool)
        .await?;
        Ok(Self {
            id: session.id,
            session_token: session.session_token,
            expires: session.expires,
            user,
        })
    }
}

impl IntoResponse for SessionWithUser {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::SET_COOKIE,
            format!(
                "{SESSION_COOKIE_NAME}={}; Expires={}",
                self.session_token,
                self.expires.to_rfc2822()
            )
            .parse()
            .unwrap(),
        );
        (headers, Json(self)).into_response()
    }
}
