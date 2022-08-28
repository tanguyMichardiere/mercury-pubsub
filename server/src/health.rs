use axum::extract::State;
use tracing::instrument;

use error::Result;

use crate::state::SharedState;

/// Check that the database connexion works, and that all migrations are successfully applied.
#[instrument]
pub(crate) async fn health(State(state): State<SharedState>) -> Result<()> {
    assert_eq!(
        sqlx::query_scalar!(r#"SELECT COUNT(*) FROM "_sqlx_migrations""#)
            .fetch_one(&state.read().await.pool)
            .await?
            .expect("NULL from SELECT scalar"),
        4
    );
    assert_eq!(
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM "_sqlx_migrations"
                WHERE success = false
            "#
        )
        .fetch_one(&state.read().await.pool)
        .await?
        .expect("NULL from SELECT scalar"),
        0
    );
    Ok(())
}

pub(crate) mod error {
    use axum::response::IntoResponse;
    use tracing::{debug, error};

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {}

    impl From<sqlx::Error> for Error {
        fn from(error: sqlx::Error) -> Self {
            error!(?error);
            panic!("unknown database error");
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {}
        }
    }
}
