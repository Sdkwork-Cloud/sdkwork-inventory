import type { InventoryStock } from './inventory-stock';

export interface InventoryStockResourceResponse {
  code: 0;
  data: { item: InventoryStock; };
  traceId: string;
}
