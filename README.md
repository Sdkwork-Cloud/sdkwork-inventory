# sdkwork-inventory

repository-kind: application

SDKWork commerce inventory capability. It owns inventory domain rules, tenant-scoped SQLx
repositories, app/backend route crates, a reusable API assembly, and a standalone gateway.

## Active Surfaces

| Surface | API authority | SDK family | Operations |
| --- | --- | --- | ---: |
| app-api | `sdkwork-inventory-app-api` | `sdkwork-inventory-app-sdk` | 2 |
| backend-api | `sdkwork-inventory-backend-api` | `sdkwork-inventory-backend-sdk` | 4 |

The OpenAPI contracts in `apis/` are owner-authored review inputs. SDK families in `sdks/` contain
materialized authority contracts, `sdkgen` inputs, composed facades, and generated transports.

## Repository Layout

- `apis/`: owner-only app/backend OpenAPI contracts.
- `crates/`: domain, repository, route, host, assembly, and standalone runtime crates.
- `database/`: inventory-owned database contracts and migrations.
- `sdks/`: app and backend SDK family workspaces.
- `specs/`: application-wide component and IAM contracts.
- `tools/`: deterministic API/SDK materialization.
- `apps/`: application-root index; no inventory UI is implemented here.

Directory rules come from `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`. API and SDK authority comes
from route manifests, OpenAPI, `sdk-manifest.json`, and component specs rather than this README.

## Verification

```powershell
pnpm check
cargo test --workspace
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)
- [apps directory index](apps/README.md)
