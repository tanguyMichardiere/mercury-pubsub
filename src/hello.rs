use axum::{response::IntoResponse, routing::get, Extension, Router};
use sqlx::PgPool;

pub fn app() -> Router {
    Router::new().route("/", get(hello))
}

async fn hello(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    sqlx::query_scalar::<_, String>("SELECT 'Hello, Mercury!'")
        .fetch_one(&pool)
        .await
        .expect("query")
}
