use axum::Router;
use sdkwork_router_inventory_app_api::build_inventory_app_router_with_framework;
use sdkwork_router_inventory_backend_api::build_inventory_backend_router_with_framework;
use sdkwork_inventory_api_server::inventory_health_router;
use sdkwork_inventory_service_host::InventoryServiceHost;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let host = Arc::new(InventoryServiceHost::new().await);
    let app = Router::new()
        .merge(inventory_health_router())
        .merge(build_inventory_app_router_with_framework(host.clone()).await)
        .merge(build_inventory_backend_router_with_framework(host).await)
        .layer(CorsLayer::permissive());
    let addr = std::env::var("INVENTORY_API_BIND").unwrap_or_else(|_| "0.0.0.0:18092".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
