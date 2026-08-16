//! API assembly for sdkwork-inventory.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod environment;
mod generated;

pub use bootstrap::{
    assemble_api_router, assemble_app_api_contribution, assemble_backend_api_contribution,
    ApiAssembly, ApiAssemblyContext,
};
pub use environment::{
    assemble_api_router_from_env, assemble_api_router_runtime, assemble_api_router_with_pool,
    assemble_app_api_contribution_from_env, assemble_backend_api_contribution_from_env,
    ApiAssemblyRuntime,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
