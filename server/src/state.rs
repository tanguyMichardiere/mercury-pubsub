use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::senders::Senders;

#[derive(Debug)]
pub struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) senders: Senders,
}

pub type SharedState = Arc<RwLock<AppState>>;

impl AppState {
    pub(crate) fn new(pool: PgPool) -> SharedState {
        Arc::new(RwLock::new(AppState {
            pool,
            senders: Senders::default(),
        }))
    }
}
