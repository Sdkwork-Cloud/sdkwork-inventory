use sdkwork_commerce_contract_service::CommerceServiceError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Deserialize)]
pub struct MerchantInventoryScopeQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
}

impl MerchantInventoryScopeQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        let tenant_id = tenant_id.trim();
        if tenant_id.is_empty() {
            return Err(CommerceServiceError::validation("tenant_id is required"));
        }
        let organization_id = organization_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        Ok(Self {
            tenant_id: tenant_id.to_owned(),
            organization_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SqliteCommerceInventoryStore {
    pool: SqlitePool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendInventoryStockListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub sku_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBackendInventoryStockCommand {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub stock_id: String,
    pub available_quantity: Option<i64>,
    pub safety_stock_quantity: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendInventoryReservationListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub order_id: Option<String>,
    pub sku_id: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendInventoryMovementListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub sku_id: Option<String>,
    pub movement_type: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendInventoryListPage {
    pub items: Vec<serde_json::Value>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

impl SqliteCommerceInventoryStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn list_backend_stocks(
        &self,
        query: BackendInventoryStockListQuery,
    ) -> Result<BackendInventoryListPage, CommerceServiceError> {
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 200);
        let offset = (page - 1) * page_size;
        let organization_id = query.organization_id.as_deref().unwrap_or("");

        let mut count_sql = String::from(
            r#"
            SELECT COUNT(*) AS total
            FROM commerce_inventory_stock
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? = ''))
            "#,
        );
        if query.sku_id.is_some() {
            count_sql.push_str(" AND sku_id = CAST(? AS TEXT)");
        }
        if query.warehouse_id.is_some() {
            count_sql.push_str(" AND warehouse_id = CAST(? AS TEXT)");
        }
        if query.status.is_some() {
            count_sql.push_str(" AND status = CAST(? AS TEXT)");
        }

        let mut count_query = sqlx::query(&count_sql)
            .bind(&query.tenant_id)
            .bind(organization_id)
            .bind(organization_id);
        if let Some(sku_id) = query.sku_id.as_deref() {
            count_query = count_query.bind(sku_id);
        }
        if let Some(warehouse_id) = query.warehouse_id.as_deref() {
            count_query = count_query.bind(warehouse_id);
        }
        if let Some(status) = query.status.as_deref() {
            count_query = count_query.bind(status);
        }
        let total_row = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("failed to count inventory stocks", error))?;
        let total: i64 = total_row.try_get("total").unwrap_or(0);

        let mut list_sql = String::from(
            r#"
            SELECT id, tenant_id, organization_id, shop_id, sku_id, warehouse_id, fulfillment_node_id,
                   on_hand_quantity, available_quantity, locked_quantity, reserved_quantity,
                   sold_quantity, safety_stock_quantity, version, status, created_at, updated_at
            FROM commerce_inventory_stock
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? = ''))
            "#,
        );
        if query.sku_id.is_some() {
            list_sql.push_str(" AND sku_id = CAST(? AS TEXT)");
        }
        if query.warehouse_id.is_some() {
            list_sql.push_str(" AND warehouse_id = CAST(? AS TEXT)");
        }
        if query.status.is_some() {
            list_sql.push_str(" AND status = CAST(? AS TEXT)");
        }
        list_sql.push_str(" ORDER BY updated_at DESC, id DESC LIMIT ? OFFSET ?");

        let mut list_query = sqlx::query(&list_sql)
            .bind(&query.tenant_id)
            .bind(organization_id)
            .bind(organization_id);
        if let Some(sku_id) = query.sku_id.as_deref() {
            list_query = list_query.bind(sku_id);
        }
        if let Some(warehouse_id) = query.warehouse_id.as_deref() {
            list_query = list_query.bind(warehouse_id);
        }
        if let Some(status) = query.status.as_deref() {
            list_query = list_query.bind(status);
        }
        list_query = list_query.bind(page_size).bind(offset);

        let rows = list_query
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("failed to list inventory stocks", error))?;

        Ok(BackendInventoryListPage {
            items: rows.iter().map(map_stock_row).collect(),
            page,
            page_size,
            total,
        })
    }

    pub async fn update_backend_stock(
        &self,
        command: UpdateBackendInventoryStockCommand,
    ) -> Result<serde_json::Value, CommerceServiceError> {
        let now = current_timestamp_string();
        let mut sets = vec!["updated_at = ?".to_owned()];
        let mut values: Vec<String> = vec![now.clone()];

        if let Some(available_quantity) = command.available_quantity {
            sets.push("available_quantity = CAST(? AS INTEGER)".to_owned());
            values.push(available_quantity.to_string());
        }
        if let Some(safety_stock_quantity) = command.safety_stock_quantity {
            sets.push("safety_stock_quantity = CAST(? AS INTEGER)".to_owned());
            values.push(safety_stock_quantity.to_string());
        }
        if let Some(status) = command.status.as_deref() {
            sets.push("status = ?".to_owned());
            values.push(status.to_owned());
        }
        if sets.len() == 1 {
            return Err(CommerceServiceError::validation(
                "at least one stock field must be provided",
            ));
        }

        let sql = format!(
            "UPDATE commerce_inventory_stock SET {} WHERE tenant_id = CAST(? AS TEXT) AND id = CAST(? AS TEXT)",
            sets.join(", ")
        );
        let mut db_query = sqlx::query(&sql);
        for value in &values {
            db_query = db_query.bind(value);
        }
        db_query = db_query.bind(&command.tenant_id).bind(&command.stock_id);
        let result = db_query
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("failed to update inventory stock", error))?;
        if result.rows_affected() == 0 {
            return Err(CommerceServiceError::not_found("inventory stock was not found"));
        }

        let row = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, shop_id, sku_id, warehouse_id, fulfillment_node_id,
                   on_hand_quantity, available_quantity, locked_quantity, reserved_quantity,
                   sold_quantity, safety_stock_quantity, version, status, created_at, updated_at
            FROM commerce_inventory_stock
            WHERE tenant_id = CAST(? AS TEXT) AND id = CAST(? AS TEXT)
            LIMIT 1
            "#,
        )
        .bind(&command.tenant_id)
        .bind(&command.stock_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve updated inventory stock", error))?;

        row.map(|row| map_stock_row(&row))
            .ok_or_else(|| CommerceServiceError::not_found("inventory stock was not found"))
    }

    pub async fn list_backend_reservations(
        &self,
        query: BackendInventoryReservationListQuery,
    ) -> Result<BackendInventoryListPage, CommerceServiceError> {
        list_paged_rows(
            &self.pool,
            &query.tenant_id,
            query.organization_id.as_deref(),
            "commerce_inventory_reservation",
            reservation_select_columns(),
            query.order_id.as_deref(),
            query.sku_id.as_deref(),
            Some(("status", query.status.as_deref())),
            query.page,
            query.page_size,
            map_reservation_row,
        )
        .await
    }

    pub async fn list_backend_movements(
        &self,
        query: BackendInventoryMovementListQuery,
    ) -> Result<BackendInventoryListPage, CommerceServiceError> {
        list_paged_rows(
            &self.pool,
            &query.tenant_id,
            query.organization_id.as_deref(),
            "commerce_inventory_movement",
            movement_select_columns(),
            None,
            query.sku_id.as_deref(),
            Some(("movement_type", query.movement_type.as_deref())),
            query.page,
            query.page_size,
            map_movement_row,
        )
        .await
    }

    pub async fn list_merchant_stocks(
        &self,
        scope: MerchantInventoryScopeQuery,
    ) -> Result<Vec<serde_json::Value>, CommerceServiceError> {
        let organization_id = scope.organization_id.as_deref().unwrap_or("");
        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, sku_id, warehouse_id, fulfillment_node_id,
                   available_quantity, reserved_quantity, inbound_quantity, damaged_quantity,
                   status, version, created_at, updated_at
            FROM commerce_inventory_stock
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? = ''))
            ORDER BY updated_at DESC, id DESC
            "#,
        )
        .bind(&scope.tenant_id)
        .bind(organization_id)
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list merchant inventory stocks", error))?;

        Ok(rows.iter().map(map_merchant_stock_row).collect())
    }

    pub async fn create_merchant_adjustment(
        &self,
        scope: MerchantInventoryScopeQuery,
        stock_id: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, CommerceServiceError> {
        let quantity_delta = payload
            .get("quantityDelta")
            .or_else(|| payload.get("quantity_delta"))
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        sqlx::query(
            r#"
            UPDATE commerce_inventory_stock
            SET available_quantity = CAST(available_quantity AS INTEGER) + CAST(? AS INTEGER),
                updated_at = ?
            WHERE tenant_id = CAST(? AS TEXT)
              AND id = CAST(? AS TEXT)
            "#,
        )
        .bind(quantity_delta)
        .bind(current_timestamp_string())
        .bind(&scope.tenant_id)
        .bind(stock_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("failed to adjust merchant inventory stock", error))?;

        let row = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, sku_id, warehouse_id, fulfillment_node_id,
                   available_quantity, reserved_quantity, inbound_quantity, damaged_quantity,
                   status, version, created_at, updated_at
            FROM commerce_inventory_stock
            WHERE tenant_id = CAST(? AS TEXT)
              AND id = CAST(? AS TEXT)
            LIMIT 1
            "#,
        )
        .bind(&scope.tenant_id)
        .bind(stock_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve adjusted merchant stock", error))?;

        row.map(|row| map_merchant_stock_row(&row))
            .ok_or_else(|| CommerceServiceError::not_found("inventory stock was not found"))
    }
}

async fn list_paged_rows(
    pool: &SqlitePool,
    tenant_id: &str,
    organization_id: Option<&str>,
    table: &str,
    select_columns: &str,
    order_id: Option<&str>,
    sku_id: Option<&str>,
    field_filter: Option<(&str, Option<&str>)>,
    page: i64,
    page_size: i64,
    map_row: fn(&sqlx::sqlite::SqliteRow) -> serde_json::Value,
) -> Result<BackendInventoryListPage, CommerceServiceError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;
    let organization_id = organization_id.unwrap_or("");

    let mut filters = String::from(
        " WHERE tenant_id = CAST(? AS TEXT) AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? = ''))",
    );
    if order_id.is_some() {
        filters.push_str(" AND order_id = CAST(? AS TEXT)");
    }
    if sku_id.is_some() {
        filters.push_str(" AND sku_id = CAST(? AS TEXT)");
    }
    if let Some((column, Some(_))) = field_filter {
        filters.push_str(&format!(" AND {column} = CAST(? AS TEXT)"));
    }

    let order_by = if table == "commerce_inventory_movement" {
        "occurred_at DESC, id DESC"
    } else {
        "updated_at DESC, id DESC"
    };

    let count_sql = format!("SELECT COUNT(*) AS total FROM {table}{filters}");
    let mut count_query = sqlx::query(&count_sql)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(organization_id);
    if let Some(order_id) = order_id {
        count_query = count_query.bind(order_id);
    }
    if let Some(sku_id) = sku_id {
        count_query = count_query.bind(sku_id);
    }
    if let Some((_, Some(value))) = field_filter {
        count_query = count_query.bind(value);
    }
    let total_row = count_query
        .fetch_one(pool)
        .await
        .map_err(|error| store_error("failed to count inventory rows", error))?;
    let total: i64 = total_row.try_get("total").unwrap_or(0);

    let list_sql = format!(
        "SELECT {select_columns} FROM {table}{filters} ORDER BY {order_by} LIMIT ? OFFSET ?"
    );
    let mut list_query = sqlx::query(&list_sql)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(organization_id);
    if let Some(order_id) = order_id {
        list_query = list_query.bind(order_id);
    }
    if let Some(sku_id) = sku_id {
        list_query = list_query.bind(sku_id);
    }
    if let Some((_, Some(value))) = field_filter {
        list_query = list_query.bind(value);
    }
    list_query = list_query.bind(page_size).bind(offset);
    let rows = list_query
        .fetch_all(pool)
        .await
        .map_err(|error| store_error("failed to list inventory rows", error))?;

    Ok(BackendInventoryListPage {
        items: rows.iter().map(map_row).collect(),
        page,
        page_size,
        total,
    })
}

fn reservation_select_columns() -> &'static str {
    "id, tenant_id, organization_id, reservation_no, order_id, checkout_session_id, order_item_id, reservation_source_type, reservation_source_id, reservation_type, sku_id, warehouse_id, fulfillment_node_id, quantity, reserved_quantity, consumed_quantity, released_quantity, status, release_reason_code, request_no, idempotency_key, expires_at, consumed_at, released_at, created_at, updated_at"
}

fn movement_select_columns() -> &'static str {
    "id, tenant_id, organization_id, movement_no, sku_id, warehouse_id, fulfillment_node_id, movement_type, source_type, quantity, direction, quantity_before, quantity_after, business_type, source_id, request_no, idempotency_key, occurred_at, created_at"
}

fn map_stock_row(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    serde_json::json!({
        "id": string_cell(row, "id"),
        "tenantId": string_cell(row, "tenant_id"),
        "organizationId": optional_string_cell(row, "organization_id"),
        "shopId": optional_string_cell(row, "shop_id"),
        "skuId": string_cell(row, "sku_id"),
        "warehouseId": optional_string_cell(row, "warehouse_id"),
        "fulfillmentNodeId": string_cell(row, "fulfillment_node_id"),
        "onHandQuantity": i64_cell(row, "on_hand_quantity"),
        "availableQuantity": i64_cell(row, "available_quantity"),
        "lockedQuantity": i64_cell(row, "locked_quantity"),
        "reservedQuantity": i64_cell(row, "reserved_quantity"),
        "soldQuantity": i64_cell(row, "sold_quantity"),
        "safetyStockQuantity": i64_cell(row, "safety_stock_quantity"),
        "version": i64_cell(row, "version"),
        "status": string_cell(row, "status"),
        "createdAt": string_cell(row, "created_at"),
        "updatedAt": string_cell(row, "updated_at"),
    })
}

fn map_reservation_row(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    serde_json::json!({
        "id": string_cell(row, "id"),
        "tenantId": string_cell(row, "tenant_id"),
        "organizationId": optional_string_cell(row, "organization_id"),
        "reservationNo": string_cell(row, "reservation_no"),
        "orderId": optional_string_cell(row, "order_id"),
        "checkoutSessionId": optional_string_cell(row, "checkout_session_id"),
        "orderItemId": optional_string_cell(row, "order_item_id"),
        "reservationSourceType": string_cell(row, "reservation_source_type"),
        "reservationSourceId": string_cell(row, "reservation_source_id"),
        "reservationType": string_cell(row, "reservation_type"),
        "skuId": string_cell(row, "sku_id"),
        "warehouseId": optional_string_cell(row, "warehouse_id"),
        "fulfillmentNodeId": string_cell(row, "fulfillment_node_id"),
        "quantity": i64_cell(row, "quantity"),
        "reservedQuantity": i64_cell(row, "reserved_quantity"),
        "consumedQuantity": i64_cell(row, "consumed_quantity"),
        "releasedQuantity": i64_cell(row, "released_quantity"),
        "status": string_cell(row, "status"),
        "releaseReasonCode": optional_string_cell(row, "release_reason_code"),
        "requestNo": string_cell(row, "request_no"),
        "idempotencyKey": string_cell(row, "idempotency_key"),
        "expiresAt": string_cell(row, "expires_at"),
        "consumedAt": optional_string_cell(row, "consumed_at"),
        "releasedAt": optional_string_cell(row, "released_at"),
        "createdAt": string_cell(row, "created_at"),
        "updatedAt": string_cell(row, "updated_at"),
    })
}

fn map_merchant_stock_row(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    serde_json::json!({
        "id": string_cell(row, "id"),
        "tenantId": string_cell(row, "tenant_id"),
        "organizationId": optional_string_cell(row, "organization_id"),
        "skuId": string_cell(row, "sku_id"),
        "warehouseId": optional_string_cell(row, "warehouse_id"),
        "fulfillmentNodeId": string_cell(row, "fulfillment_node_id"),
        "availableQuantity": i64_cell(row, "available_quantity"),
        "reservedQuantity": i64_cell(row, "reserved_quantity"),
        "inboundQuantity": i64_cell(row, "inbound_quantity"),
        "damagedQuantity": i64_cell(row, "damaged_quantity"),
        "status": string_cell(row, "status"),
        "version": i64_cell(row, "version"),
        "createdAt": string_cell(row, "created_at"),
        "updatedAt": string_cell(row, "updated_at"),
    })
}

fn map_movement_row(row: &sqlx::sqlite::SqliteRow) -> serde_json::Value {
    serde_json::json!({
        "id": string_cell(row, "id"),
        "tenantId": string_cell(row, "tenant_id"),
        "organizationId": optional_string_cell(row, "organization_id"),
        "movementNo": string_cell(row, "movement_no"),
        "skuId": string_cell(row, "sku_id"),
        "warehouseId": optional_string_cell(row, "warehouse_id"),
        "fulfillmentNodeId": string_cell(row, "fulfillment_node_id"),
        "movementType": string_cell(row, "movement_type"),
        "sourceType": string_cell(row, "source_type"),
        "quantity": i64_cell(row, "quantity"),
        "direction": string_cell(row, "direction"),
        "quantityBefore": optional_i64_cell(row, "quantity_before"),
        "quantityAfter": optional_i64_cell(row, "quantity_after"),
        "businessType": string_cell(row, "business_type"),
        "sourceId": string_cell(row, "source_id"),
        "requestNo": string_cell(row, "request_no"),
        "idempotencyKey": string_cell(row, "idempotency_key"),
        "occurredAt": string_cell(row, "occurred_at"),
        "createdAt": string_cell(row, "created_at"),
    })
}

fn string_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
    optional_string_cell(row, column).unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn i64_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> i64 {
    row.try_get::<i64, _>(column).unwrap_or(0)
}

fn optional_i64_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<i64> {
    row.try_get::<Option<i64>, _>(column).ok().flatten()
}

fn store_error(message: &str, error: impl std::fmt::Display) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{message}: {error}"))
}

fn current_timestamp_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}
