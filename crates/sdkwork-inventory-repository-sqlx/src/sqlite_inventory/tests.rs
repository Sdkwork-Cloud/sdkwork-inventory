use super::*;

#[test]
fn merchant_list_query_rejects_invalid_offset_pagination() {
    assert!(MerchantInventoryListQuery::new("tenant-1", None, 0, 20).is_err());
    assert!(MerchantInventoryListQuery::new("tenant-1", None, 1, 0).is_err());
    assert!(MerchantInventoryListQuery::new("tenant-1", None, 1, 201).is_err());

    let query = MerchantInventoryListQuery::new("tenant-1", None, 2, 20).expect("valid pagination");
    assert_eq!(query.page, 2);
    assert_eq!(query.page_size, 20);
}

#[tokio::test]
async fn merchant_stock_list_applies_limit_offset_and_reports_total() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("sqlite pool");
    sqlx::query(
        r#"
        CREATE TABLE commerce_inventory_stock (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            organization_id TEXT,
            sku_id TEXT NOT NULL,
            warehouse_id TEXT,
            fulfillment_node_id TEXT,
            available_quantity INTEGER NOT NULL,
            reserved_quantity INTEGER NOT NULL,
            inbound_quantity INTEGER NOT NULL,
            damaged_quantity INTEGER NOT NULL,
            status TEXT NOT NULL,
            version INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("inventory fixture table");

    for index in 1..=5 {
        sqlx::query(
            r#"
            INSERT INTO commerce_inventory_stock (
                id, tenant_id, organization_id, sku_id, warehouse_id,
                fulfillment_node_id, available_quantity, reserved_quantity,
                inbound_quantity, damaged_quantity, status, version, created_at, updated_at
            ) VALUES (?, 'tenant-1', NULL, ?, NULL, NULL, 10, 0, 0, 0, 'active', 1, ?, ?)
            "#,
        )
        .bind(format!("stock-{index}"))
        .bind(format!("sku-{index}"))
        .bind(format!("2026-07-22T00:00:0{index}Z"))
        .bind(format!("2026-07-22T00:00:0{index}Z"))
        .execute(&pool)
        .await
        .expect("inventory fixture row");
    }

    let page = SqliteCommerceInventoryStore::new(pool)
        .list_merchant_stocks(
            MerchantInventoryListQuery::new("tenant-1", None, 2, 2).expect("list query"),
        )
        .await
        .expect("inventory page");

    assert_eq!(page.items.len(), 2);
    assert_eq!(page.total, 5);
    assert_eq!(page.page, 2);
    assert_eq!(page.page_size, 2);
    assert_eq!(page.items[0]["id"], "stock-3");
    assert_eq!(page.items[1]["id"], "stock-2");
}
