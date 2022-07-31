use std::io;

use hyper::StatusCode;
use tracing::error;

pub fn handle_database_error(error: sqlx::error::Error) -> (StatusCode, &'static str) {
    error!(?error);
    (StatusCode::BAD_GATEWAY, "Database connection error")
}

pub async fn handle_io_error(error: io::Error) -> (StatusCode, &'static str) {
    error!(?error);
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}
