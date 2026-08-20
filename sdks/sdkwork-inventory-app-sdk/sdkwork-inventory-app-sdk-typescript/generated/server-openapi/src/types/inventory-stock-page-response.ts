import type { InventoryStock } from './inventory-stock';
import type { PageInfo } from './page-info';

export interface InventoryStockPageResponse {
  code: 0;
  data: { items: InventoryStock[]; pageInfo: PageInfo; };
  traceId: string;
}
