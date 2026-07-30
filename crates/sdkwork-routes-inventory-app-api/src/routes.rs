use axum::Router;
use sdkwork_inventory_service_host::InventoryServiceHost;
use std::sync::Arc;

use crate::app_merchant_inventory_router_with_postgres_pool;
use crate::web_bootstrap::wrap_router_with_web_framework_from_env;

pub fn build_inventory_app_router(host: Arc<InventoryServiceHost>) -> Router {
    let pool = host
        .database_pool()
        .as_postgres()
        .expect("inventory app-api requires an authoritative PostgreSQL pool");
    app_merchant_inventory_router_with_postgres_pool(pool.clone())
}

pub async fn build_inventory_app_router_with_framework(host: Arc<InventoryServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_inventory_app_router(host)).await
}
