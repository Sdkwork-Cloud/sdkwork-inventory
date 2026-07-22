export interface InventoryMovement {
  id: string;
  tenantId: string;
  organizationId?: string | null;
  createdAt?: string;
  updatedAt?: string;
  movementNo?: string;
  skuId: string;
  movementType: string;
  quantity: string;
  direction?: string;
  occurredAt?: string;
}
