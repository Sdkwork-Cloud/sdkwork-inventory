export interface InventoryStock {
  id: string;
  tenantId: string;
  organizationId?: string | null;
  createdAt?: string;
  updatedAt?: string;
  skuId: string;
  warehouseId?: string | null;
  fulfillmentNodeId?: string;
  onHandQuantity?: string;
  availableQuantity: string;
  lockedQuantity?: string;
  reservedQuantity: string;
  inboundQuantity?: string;
  damagedQuantity?: string;
  safetyStockQuantity?: string;
  version?: string;
  status: string;
}
