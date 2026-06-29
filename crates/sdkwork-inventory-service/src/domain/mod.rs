use sdkwork_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryStockDraft {
    pub tenant_id: String,
    pub sku_id: String,
    pub warehouse_id: Option<String>,
    pub available_quantity: i64,
    pub reserved_quantity: i64,
    pub sold_quantity: i64,
    pub version: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InventoryReservationStatus {
    Reserved,
    Consumed,
    Released,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryReservationTransition {
    from: InventoryReservationStatus,
    to: InventoryReservationStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryReservationDraft {
    pub tenant_id: String,
    pub reservation_id: String,
    pub order_id: String,
    pub sku_id: String,
    pub warehouse_id: Option<String>,
    pub quantity: i64,
    pub expires_at: String,
    pub idempotency_key: String,
    pub status: InventoryReservationStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InventoryMovementType {
    Reserve,
    Release,
    Consume,
    Adjust,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryMovementDraft {
    pub tenant_id: String,
    pub movement_no: String,
    pub sku_id: String,
    pub warehouse_id: Option<String>,
    pub movement_type: InventoryMovementType,
    pub quantity: i64,
    pub business_type: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryDeductionPolicy {
    pub reserve_on_order_create: bool,
    pub consume_on_payment_success: bool,
    pub release_on_order_cancel: bool,
    pub release_on_reservation_expire: bool,
    pub release_on_refund: bool,
}

impl InventoryStockDraft {
    pub fn new(
        tenant_id: &str,
        sku_id: &str,
        warehouse_id: Option<&str>,
        available_quantity: i64,
        reserved_quantity: i64,
        sold_quantity: i64,
        version: i64,
    ) -> Result<Self, CommerceServiceError> {
        if available_quantity < 0 || reserved_quantity < 0 || sold_quantity < 0 || version < 0 {
            return Err(CommerceServiceError::validation(
                "inventory quantities and version must not be negative",
            ));
        }

        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            sku_id: required_text("sku_id", sku_id)?,
            warehouse_id: optional_text(warehouse_id),
            available_quantity,
            reserved_quantity,
            sold_quantity,
            version,
        })
    }
}

impl InventoryReservationStatus {
    pub fn as_storage_str(&self) -> &'static str {
        match self {
            Self::Reserved => "reserved",
            Self::Consumed => "consumed",
            Self::Released => "released",
            Self::Expired => "expired",
        }
    }
}

impl InventoryReservationTransition {
    pub fn new(from: InventoryReservationStatus, to: InventoryReservationStatus) -> Self {
        Self { from, to }
    }

    pub fn validate(&self) -> Result<(), CommerceServiceError> {
        match (&self.from, &self.to) {
            (InventoryReservationStatus::Reserved, InventoryReservationStatus::Consumed)
            | (InventoryReservationStatus::Reserved, InventoryReservationStatus::Released)
            | (InventoryReservationStatus::Reserved, InventoryReservationStatus::Expired) => Ok(()),
            _ => Err(CommerceServiceError::invalid_state(
                "invalid inventory reservation transition",
            )),
        }
    }
}

impl InventoryReservationDraft {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        reservation_id: &str,
        order_id: &str,
        sku_id: &str,
        warehouse_id: Option<&str>,
        quantity: i64,
        expires_at: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        if quantity <= 0 {
            return Err(CommerceServiceError::validation(
                "inventory reservation quantity must be greater than zero",
            ));
        }

        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            reservation_id: required_text("reservation_id", reservation_id)?,
            order_id: required_text("order_id", order_id)?,
            sku_id: required_text("sku_id", sku_id)?,
            warehouse_id: optional_text(warehouse_id),
            quantity,
            expires_at: required_text("expires_at", expires_at)?,
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
            status: InventoryReservationStatus::Reserved,
        })
    }
}

impl InventoryMovementType {
    pub fn as_storage_str(&self) -> &'static str {
        match self {
            Self::Reserve => "reserve",
            Self::Release => "release",
            Self::Consume => "consume",
            Self::Adjust => "adjust",
        }
    }
}

impl InventoryMovementDraft {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        movement_no: &str,
        sku_id: &str,
        warehouse_id: Option<&str>,
        movement_type: InventoryMovementType,
        quantity: i64,
        business_type: &str,
        source_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        if quantity <= 0 {
            return Err(CommerceServiceError::validation(
                "inventory movement quantity must be greater than zero",
            ));
        }

        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            movement_no: required_text("movement_no", movement_no)?,
            sku_id: required_text("sku_id", sku_id)?,
            warehouse_id: optional_text(warehouse_id),
            movement_type,
            quantity,
            business_type: required_text("business_type", business_type)?,
            source_id: required_text("source_id", source_id)?,
        })
    }
}

impl InventoryDeductionPolicy {
    pub fn reserve_on_order_and_consume_on_payment() -> Self {
        Self {
            reserve_on_order_create: true,
            consume_on_payment_success: true,
            release_on_order_cancel: true,
            release_on_reservation_expire: true,
            release_on_refund: false,
        }
    }
}

fn required_text(field_name: &str, value: &str) -> Result<String, CommerceServiceError> {
    crate::validation::require_non_empty(field_name, value)?;
    Ok(value.trim().to_string())
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
