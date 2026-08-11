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

-- Order-scoped reservation ledger consumed by the commerce order flow
-- (reserve at checkout, release/consume on cancel/fulfillment). Column shape
-- mirrors the physical-commerce integration tests:
-- crates/sdkwork-order-integration-physical-commerce/src/inventory/tests.rs.
CREATE TABLE IF NOT EXISTS commerce_inventory_reservation (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT NOT NULL DEFAULT '0',
    reservation_no TEXT NOT NULL,
    order_id TEXT NOT NULL,
    reservation_source_type TEXT NOT NULL,
    reservation_source_id TEXT NOT NULL,
    reservation_type TEXT NOT NULL,
    sku_id TEXT NOT NULL,
    warehouse_id TEXT,
    fulfillment_node_id TEXT,
    quantity BIGINT NOT NULL DEFAULT 0,
    reserved_quantity BIGINT NOT NULL DEFAULT 0,
    consumed_quantity BIGINT NOT NULL DEFAULT 0,
    released_quantity BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'reserved',
    release_reason_code TEXT,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    consumed_at TEXT,
    released_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_commerce_inventory_reservation_order
    ON commerce_inventory_reservation (tenant_id, order_id);
