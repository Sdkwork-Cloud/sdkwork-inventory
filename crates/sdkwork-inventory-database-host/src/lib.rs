use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{
    DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule, SpiError,
};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};
use std::path::PathBuf;
use std::sync::Arc;

pub struct InventoryDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl InventoryDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

/// Returns the inventory [`DefaultDatabaseModule`] loaded from the inventory
/// repository's `database/` directory.
///
/// # Convention
///
/// Each `*-database-host` crate exports this function so that federated hosts
/// (e.g. CloudRouter) can register the module in a `DatabaseModuleRegistry`
/// and run init + migrate + seed through
/// `RegistryLifecycleOrchestrator::bootstrap_all`.
pub fn database_module() -> Result<DefaultDatabaseModule, SpiError> {
    let app_root = std::env::var("SDKWORK_INVENTORY_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Canonicalize the CARGO_MANIFEST_DIR + "../.." path so that the
            // resulting app_root does not contain ".." components. The seed
            // security validator rejects paths containing ".." as path
            // traversal.
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        });
    DefaultDatabaseModule::from_app_root(&app_root)
}

pub async fn bootstrap_inventory_database_from_env() -> Result<InventoryDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("INVENTORY")
        .map_err(|error| format!("read inventory database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create inventory database pool failed: {error}"))?;
    bootstrap_inventory_database_with_pool(pool).await
}

/// Bootstrap inventory assets against a caller-provided database pool so the
/// platform cloud gateway can share its process-wide PostgreSQL pool.
pub async fn bootstrap_inventory_database_with_pool(
    pool: DatabasePool,
) -> Result<InventoryDatabaseHost, String> {
    let module = Arc::new(
        database_module()
            .map_err(|error| format!("load inventory database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read inventory database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("INVENTORY", &manifest);
    let orchestrator = LifecycleOrchestrator::new(pool.clone(), module.clone())
        .with_applied_by("sdkwork-inventory");
    orchestrator.init().await.map_err(|e| format!("{e}"))?;
    if options.auto_migrate {
        orchestrator.migrate().await.map_err(|e| format!("{e}"))?;
    }
    Ok(InventoryDatabaseHost { pool, module })
}
