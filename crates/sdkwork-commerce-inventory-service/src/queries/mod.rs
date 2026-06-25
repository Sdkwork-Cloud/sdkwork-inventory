use sdkwork_commerce_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryStockQuery {
    pub sku_id: String,
    pub tenant_id: String,
    pub warehouse_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryReservationListQuery {
    pub order_id: Option<String>,
    pub sku_id: Option<String>,
    pub tenant_id: String,
}

impl InventoryStockQuery {
    pub fn new(
        tenant_id: &str,
        sku_id: &str,
        warehouse_id: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            sku_id: required_text("sku_id", sku_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
            warehouse_id: optional_text(warehouse_id),
        })
    }
}

impl InventoryReservationListQuery {
    pub fn new(
        tenant_id: &str,
        order_id: Option<&str>,
        sku_id: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            order_id: optional_text(order_id),
            sku_id: optional_text(sku_id),
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
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
