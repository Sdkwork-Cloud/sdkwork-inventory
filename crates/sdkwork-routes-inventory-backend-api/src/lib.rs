pub mod backend_inventory_router;
pub mod subject;

pub use backend_inventory_router::{
    backend_inventory_router_with_postgres_pool, backend_inventory_router_with_sqlite_pool,
    build_backend_inventory_router, CommerceBackendInventoryStore,
};

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_inventory_service_host::InventoryServiceHost;
use std::sync::Arc;

pub fn build_inventory_backend_router(host: Arc<InventoryServiceHost>) -> Router {
    match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            backend_inventory_router_with_postgres_pool(pool.clone())
        }
        DatabasePool::Sqlite(pool, _) => {
            backend_inventory_router_with_sqlite_pool(pool.clone())
        }
    }
}

pub async fn build_inventory_backend_router_with_framework(
    host: Arc<InventoryServiceHost>,
) -> Router {
    build_inventory_backend_router(host)
}

pub async fn gateway_mount(host: Arc<InventoryServiceHost>,) -> Router {
    build_inventory_backend_router_with_framework(host).await
}
