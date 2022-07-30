use hyper::StatusCode;
use tracing::error;

pub fn handle_database_error(error: sqlx::error::Error) -> StatusCode {
    error!(?error);
    StatusCode::BAD_GATEWAY
}
