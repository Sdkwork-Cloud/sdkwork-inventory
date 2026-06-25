# Inventory PRD

Status: active
Owner: SDKWork maintainers
Application: inventory
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Platform split alignment (commerce T0): `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`

## 1. Background And Problem

Stock levels, reservations, and inventory adjustments require an isolated capability with clear tenant boundaries and auditable mutations.

This repository is a **T1 commerce capability building block**. `sdkwork-commerce` remains the T0 composition layer (gateway, IAM wrappers, composed SDK). This repository owns domain logic, persistence, and HTTP route builders for the **inventory** capability.

## 2. Target Users

Warehouse operators, merchant admins, and order fulfillment integrators.

## 3. Goals And Non-Goals

### Goals

- Own inventory domain service, repository SQL, backend admin HTTP, and merchant app inventory routes.
- Expose merchant stock list/adjustment at `/app/v3/api/shops/current/inventory/*` from inventory app router.

### Non-Goals

- Order payment or catalog master ownership.

## 4. Scope

- Inventory service domain.
- Backend inventory SQL + HTTP (stocks, reservations, movements list/update).
- Merchant app inventory SQL + HTTP (current shop stock list and adjustments).

Primary API prefixes:

- App: `/app/v3/api/shops/current/inventory`
- Backend: `/backend/v3/api/inventory`

Migration status: **complete**.

## 5. User Scenarios

- Fulfillment reserves stock when an order moves to allocated status.
- A merchant operator lists current shop stock and posts quantity adjustments.
- An admin operator adjusts on-hand quantity from backend inventory routes.

## 6. Success Metrics

- Backend and merchant inventory routes return real data instead of manifest 501 stubs.
- Repository crate is the sole inventory SQL owner (shop repo no longer queries inventory tables).

## 7. Phases

- Phase 1 (complete): domain service moved to sibling repo.
- Phase 2 (complete): backend + merchant app SQL/HTTP in sibling repo.

## 8. Linked Requirements

- Commerce capability split alignment: `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions

- Whether merchant inventory routes should move from `/shops/current/inventory/*` to `/inventory/*` before production launch.
