use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_inventory_service_host::InventoryServiceHost;
use std::sync::Arc;

use crate::web_bootstrap::wrap_router_with_web_framework_from_env;
use crate::{
    app_merchant_inventory_router_with_postgres_pool,
    app_merchant_inventory_router_with_sqlite_pool,
};

pub fn build_inventory_app_router(host: Arc<InventoryServiceHost>) -> Router {
    match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            app_merchant_inventory_router_with_postgres_pool(pool.clone())
        }
        DatabasePool::Sqlite(pool, _) => {
            app_merchant_inventory_router_with_sqlite_pool(pool.clone())
        }
    }
}

pub async fn build_inventory_app_router_with_framework(host: Arc<InventoryServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_inventory_app_router(host)).await
}
