use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::database::pool;
use crate::senders::Senders;

#[derive(Debug)]
pub struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) senders: Senders,
}

pub type SharedState = Arc<RwLock<AppState>>;

impl AppState {
    pub(crate) async fn new() -> Result<SharedState> {
        let pool = pool().await?;
        let senders = Senders::default();
        let state = Arc::new(RwLock::new(AppState { pool, senders }));

        Ok(state)
    }
}
