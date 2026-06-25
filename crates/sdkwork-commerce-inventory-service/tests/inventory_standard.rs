use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_inventory_service::{
    inventory_service_contract, InventoryDeductionPolicy, InventoryMovementDraft,
    InventoryMovementType, InventoryPortRequirement, InventoryRepositoryCommand,
    InventoryReservationDraft, InventoryReservationStatus, InventoryReservationTransition,
    InventoryStockDraft,
};

#[test]
fn validates_stock_quantities_with_available_reserved_and_sold_buckets() {
    let stock = InventoryStockDraft::new(
        "tenant-1",
        "sku-physical-standard",
        Some("warehouse-shanghai"),
        100,
        10,
        5,
        7,
    )
    .unwrap();

    assert_eq!(stock.available_quantity, 100);
    assert_eq!(stock.reserved_quantity, 10);
    assert_eq!(stock.sold_quantity, 5);
    assert_eq!(stock.version, 7);
    assert!(InventoryStockDraft::new("tenant-1", "sku-1", None, -1, 0, 0, 0).is_err());
}

#[test]
fn validates_inventory_reservations_for_order_prehold_flow() {
    let reservation = InventoryReservationDraft::new(
        "tenant-1",
        "reservation-1",
        "order-1",
        "sku-physical-standard",
        Some("warehouse-shanghai"),
        2,
        "2026-05-21T12:00:00Z",
        "idem-reserve-1",
    )
    .unwrap();

    assert_eq!(reservation.quantity, 2);
    assert_eq!(reservation.status, InventoryReservationStatus::Reserved);
    assert_eq!(
        InventoryReservationStatus::Reserved.as_storage_str(),
        "reserved"
    );
    assert_eq!(
        InventoryReservationStatus::Consumed.as_storage_str(),
        "consumed"
    );
    assert_eq!(
        InventoryReservationStatus::Released.as_storage_str(),
        "released"
    );
    assert!(InventoryReservationDraft::new(
        "tenant-1",
        "reservation-1",
        "order-1",
        "sku-physical-standard",
        None,
        0,
        "2026-05-21T12:00:00Z",
        "idem-reserve-1",
    )
    .is_err());
}

#[test]
fn validates_reservation_transitions_and_deduction_policy() {
    assert_eq!(
        InventoryReservationTransition::new(
            InventoryReservationStatus::Reserved,
            InventoryReservationStatus::Consumed
        )
        .validate(),
        Ok(()),
    );
    assert_eq!(
        InventoryReservationTransition::new(
            InventoryReservationStatus::Reserved,
            InventoryReservationStatus::Released
        )
        .validate(),
        Ok(()),
    );
    assert_eq!(
        InventoryReservationTransition::new(
            InventoryReservationStatus::Released,
            InventoryReservationStatus::Consumed
        )
        .validate(),
        Err(CommerceServiceError::invalid_state(
            "invalid inventory reservation transition"
        )),
    );

    let policy = InventoryDeductionPolicy::reserve_on_order_and_consume_on_payment();
    assert!(policy.reserve_on_order_create);
    assert!(policy.consume_on_payment_success);
    assert!(policy.release_on_order_cancel);
    assert!(policy.release_on_reservation_expire);
}

#[test]
fn validates_inventory_movement_ledger_shape() {
    let movement = InventoryMovementDraft::new(
        "tenant-1",
        "move-1",
        "sku-physical-standard",
        Some("warehouse-shanghai"),
        InventoryMovementType::Reserve,
        2,
        "order",
        "order-1",
    )
    .unwrap();

    assert_eq!(movement.movement_type.as_storage_str(), "reserve");
    assert_eq!(InventoryMovementType::Release.as_storage_str(), "release");
    assert_eq!(InventoryMovementType::Consume.as_storage_str(), "consume");
    assert_eq!(InventoryMovementType::Adjust.as_storage_str(), "adjust");
    assert!(InventoryMovementDraft::new(
        "tenant-1",
        "move-1",
        "sku-physical-standard",
        None,
        InventoryMovementType::Reserve,
        -1,
        "order",
        "order-1",
    )
    .is_err());
}

#[test]
fn inventory_repository_contract_exposes_required_commands() {
    assert_eq!(
        InventoryPortRequirement::standard_commands(),
        vec![
            InventoryRepositoryCommand::UpsertStock,
            InventoryRepositoryCommand::CreateReservation,
            InventoryRepositoryCommand::ConsumeReservation,
            InventoryRepositoryCommand::ReleaseReservation,
            InventoryRepositoryCommand::AppendMovement,
            InventoryRepositoryCommand::AdjustStock,
        ],
    );
}

#[test]
fn inventory_service_contract_exposes_domain_operations() {
    let contract = inventory_service_contract();

    assert_eq!(contract.domain, "inventory");
    assert_eq!(contract.service_name, "commerce.inventory");
    assert!(contract.validate().is_ok());
    for query in [
        "inventory.stocks.list",
        "inventory.reservations.list",
        "inventory.movements.list",
    ] {
        assert!(
            contract.read_queries.contains(&query),
            "inventory contract must expose read query {query}",
        );
    }
    assert!(!contract
        .read_queries
        .contains(&"inventory.ledgerEntries.list"));
    assert!(!contract.read_queries.contains(&"inventory.ledger.list"));
    let command = "inventory.stocks.update";
    assert!(
        contract.write_commands.contains(&command),
        "inventory contract must expose write command {command}",
    );
}
