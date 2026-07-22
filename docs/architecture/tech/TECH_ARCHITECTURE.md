# Inventory Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-22
Specs: ARCHITECTURE_DECISION_SPEC.md, RUST_CODE_SPEC.md, API_SPEC.md, SDK_SPEC.md, PAGINATION_SPEC.md, SECURITY_SPEC.md,
APPLICATION_GATEWAY_SPEC.md

## 1. Architecture Overview

`sdkwork-inventory` is a composable Rust/Axum capability. Domain and SQLx layers are independent of
HTTP; route crates expose app/backend routers; the assembly creates shared service state and merges
those routers; the standalone gateway adds process concerns. The platform cloud gateway embeds the
same assembly rather than rebuilding inventory state.

## 2. Technology Choices

- Rust domain services and ports.
- SQLx repositories for PostgreSQL and SQLite with store-level pagination.
- Axum route composition through `sdkwork-web-framework`.
- `sdkwork-utils-rust` HTTP helpers for validated list parameters and standard envelopes.
- `@sdkwork/sdk-generator` through `sdkgen` for generated transports.

## 3. System Boundaries And Modules

| Layer | Owner |
| --- | --- |
| Domain behavior | `sdkwork-inventory-service` |
| SQL persistence | `sdkwork-inventory-repository-sqlx` |
| Database lifecycle adapter | `sdkwork-inventory-database-host` |
| Shared runtime state | `sdkwork-inventory-service-host` |
| App routes | `sdkwork-routes-inventory-app-api` |
| Backend routes | `sdkwork-routes-inventory-backend-api` |
| Router composition | `sdkwork-api-inventory-assembly` |
| Independent process | `sdkwork-api-inventory-standalone-gateway` |

## 4. Runtime Composition

```text
database pool -> inventory service host -> inventory assembly
                                      -> app router (2 operations)
                                      -> backend router (4 operations)
inventory assembly -> standalone gateway or platform cloud gateway
```

Health, readiness, liveness, and metrics endpoints belong to the host gateway and are not exported
by the capability assembly.

## 5. API, SDK, And Data Ownership

| Surface | Route owner | API authority | SDK family |
| --- | --- | --- | --- |
| app-api | `sdkwork-routes-inventory-app-api` | `sdkwork-inventory-app-api` | `sdkwork-inventory-app-sdk` |
| backend-api | `sdkwork-routes-inventory-backend-api` | `sdkwork-inventory-backend-api` | `sdkwork-inventory-backend-sdk` |

The app family contains 2 operations and the backend family contains 4. No inventory open-api
surface is declared. SQL ownership remains in inventory repository/database contracts; this change
does not alter schemas or migrations.

## 6. Security, Privacy, And Observability

IAM middleware supplies authenticated tenant context. Route permissions are
`commerce.inventory.read` and `commerce.inventory.manage`. Responses follow the current SDKWork API
envelope and ProblemDetail contracts; identifiers outside the public resource contract remain
internal. SQL errors are mapped without leaking database details.

## 7. Deployment And Runtime Topology

The standalone binary supports independent validation. Production composition uses the cloud
gateway feature/dependency/runtime contract and the same assembly export. Both use the process-shared
database pool rather than creating route-local pools.

## 8. Architecture Decision Index

No repository-local ADR is required for the current authority mapping; machine evidence is in the
assembly manifest, component specs, route manifests, and SDK manifests.

## 9. Verification

```powershell
pnpm check
cargo test --workspace
cargo clippy --workspace --tests -- -D warnings
```
