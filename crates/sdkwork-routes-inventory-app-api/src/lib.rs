pub mod app_merchant_inventory_router;
pub mod routes;
pub mod subject;
pub mod web_bootstrap;

pub use app_merchant_inventory_router::{
    app_merchant_inventory_router_with_postgres_pool,
    app_merchant_inventory_router_with_sqlite_pool, build_app_merchant_inventory_router,
    CommerceMerchantInventoryStore,
};
pub use routes::build_inventory_app_router_with_framework;
pub use web_bootstrap::wrap_router_with_web_framework_from_env;

use axum::Router;
use sdkwork_inventory_service_host::InventoryServiceHost;
use std::sync::Arc;

pub async fn gateway_mount(host: Arc<InventoryServiceHost>) -> Router {
    build_inventory_app_router_with_framework(host).await
}
