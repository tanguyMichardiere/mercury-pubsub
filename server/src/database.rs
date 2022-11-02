use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::CONFIG;

pub async fn pool() -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&CONFIG.database_url)
        .await?;

    Ok(pool)
}
