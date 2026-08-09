-- sdkwork:baseline
-- module: inventory
--
-- Inventory stock facts owned by the inventory capability. The catalog module
-- declares commerce_inventory_* as reference-only; the inventory module is the
-- physical owner of its stock table (column shape mirrors
-- crates/sdkwork-inventory-repository-sqlx/src/postgres_inventory.rs: text
-- ids, text timestamps, quantity counters).
CREATE TABLE IF NOT EXISTS commerce_inventory_stock (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT NOT NULL DEFAULT '0',
    shop_id TEXT,
    sku_id TEXT,
    warehouse_id TEXT,
    fulfillment_node_id TEXT,
    on_hand_quantity BIGINT NOT NULL DEFAULT 0,
    available_quantity BIGINT NOT NULL DEFAULT 0,
    locked_quantity BIGINT NOT NULL DEFAULT 0,
    reserved_quantity BIGINT NOT NULL DEFAULT 0,
    sold_quantity BIGINT NOT NULL DEFAULT 0,
    safety_stock_quantity BIGINT NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_commerce_inventory_stock_sku
    ON commerce_inventory_stock (tenant_id, sku_id, status);
