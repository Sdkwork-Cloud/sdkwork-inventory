import type { InventoryMovement } from './inventory-movement';
import type { PageInfo } from './page-info';

export interface InventoryMovementPageResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
