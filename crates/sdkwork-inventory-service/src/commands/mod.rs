use sdkwork_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateInventoryReservationCommand {
    pub idempotency_key: String,
    pub order_id: String,
    pub quantity: i64,
    pub request_no: String,
    pub sku_id: String,
    pub tenant_id: String,
}

impl CreateInventoryReservationCommand {
    pub fn new(
        tenant_id: &str,
        order_id: &str,
        sku_id: &str,
        quantity: i64,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        if quantity <= 0 {
            return Err(CommerceServiceError::validation(
                "inventory reservation quantity must be greater than zero",
            ));
        }

        Ok(Self {
            idempotency_key: required_text("idempotency_key", idempotency_key)?,
            order_id: required_text("order_id", order_id)?,
            quantity,
            request_no: required_text("request_no", request_no)?,
            sku_id: required_text("sku_id", sku_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }
}

fn required_text(field_name: &str, value: &str) -> Result<String, CommerceServiceError> {
    crate::validation::require_non_empty(field_name, value)?;
    Ok(value.trim().to_string())
}
