import type { InventoryReservation } from './inventory-reservation';
import type { PageInfo } from './page-info';

export interface InventoryReservationPageResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
