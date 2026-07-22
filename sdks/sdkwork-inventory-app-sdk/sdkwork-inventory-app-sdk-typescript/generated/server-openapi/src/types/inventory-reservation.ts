export interface InventoryReservation {
  id: string;
  tenantId: string;
  organizationId?: string | null;
  createdAt?: string;
  updatedAt?: string;
  reservationNo?: string;
  orderId?: string | null;
  skuId: string;
  status: string;
  quantity: string;
}
