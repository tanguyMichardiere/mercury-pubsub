use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

impl User {
    pub async fn new(pool: &PgPool, name: &str, password: &str) -> sqlx::Result<Self> {
        sqlx::query_as!(
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
        .await
    }

    pub async fn count(pool: &PgPool) -> sqlx::Result<i64> {
        sqlx::query_scalar!(r#"SELECT COUNT(*) FROM "User""#)
            .fetch_one(pool)
            .await
            .map(|option| option.expect("NULL from SELECT COUNT"))
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "User"
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get_by_name_and_password(
        pool: &PgPool,
        name: &str,
        password: &str,
    ) -> sqlx::Result<Option<Self>> {
        match sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "User"
                WHERE name = $1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?
        {
            Some(user) => {
                if sqlx::query_scalar!(r#"SELECT $1 = crypt($2, $1)"#, user.password_hash, password)
                    .fetch_one(pool)
                    .await
                    .map(|option| option.expect("NULL from SELECT scalar"))?
                {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
