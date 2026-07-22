use sdkwork_contract_service::CommerceServiceContract;

pub fn inventory_service_contract() -> CommerceServiceContract {
    CommerceServiceContract::new(
        "inventory",
        "commerce.inventory",
        vec![
            "inventory.stocks.update",
            "shops.current.inventory.stocks.adjustments.create",
        ],
        vec![
            "inventory.stocks.list",
            "inventory.reservations.list",
            "inventory.movements.list",
            "shops.current.inventory.stocks.list",
        ],
        vec![
            crate::ports::INVENTORY_REPOSITORY_PORT,
            crate::ports::IDEMPOTENCY_REPOSITORY_PORT,
        ],
        true,
    )
}
