use crate::{
    InventoryMovementDraft, InventoryReservationDraft, InventoryReservationListQuery,
    InventoryStockDraft, InventoryStockQuery,
};
use sdkwork_commerce_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InventoryRepositoryCommand {
    UpsertStock,
    CreateReservation,
    ConsumeReservation,
    ReleaseReservation,
    AppendMovement,
    AdjustStock,
}

pub struct InventoryPortRequirement;

pub trait InventoryRepositoryPort {
    fn retrieve_stock(
        &self,
        query: &InventoryStockQuery,
    ) -> Result<Option<InventoryStockDraft>, CommerceServiceError>;

    fn upsert_stock(&self, draft: &InventoryStockDraft) -> Result<(), CommerceServiceError>;

    fn create_reservation(
        &self,
        draft: &InventoryReservationDraft,
    ) -> Result<(), CommerceServiceError>;

    fn list_reservations(
        &self,
        query: &InventoryReservationListQuery,
    ) -> Result<Vec<InventoryReservationDraft>, CommerceServiceError>;

    fn append_movement(&self, draft: &InventoryMovementDraft) -> Result<(), CommerceServiceError>;
}

pub const INVENTORY_REPOSITORY_PORT: &str = "inventory.repository";
pub const IDEMPOTENCY_REPOSITORY_PORT: &str = "idempotency.repository";

impl InventoryPortRequirement {
    pub fn standard_commands() -> Vec<InventoryRepositoryCommand> {
        vec![
            InventoryRepositoryCommand::UpsertStock,
            InventoryRepositoryCommand::CreateReservation,
            InventoryRepositoryCommand::ConsumeReservation,
            InventoryRepositoryCommand::ReleaseReservation,
            InventoryRepositoryCommand::AppendMovement,
            InventoryRepositoryCommand::AdjustStock,
        ]
    }
}
