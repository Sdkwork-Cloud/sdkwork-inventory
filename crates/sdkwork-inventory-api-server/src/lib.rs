use axum::Router;
use axum::routing::get;

pub fn inventory_health_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(|| async { "ready" }))
}
