# hnf-adapter-sdk

Apache-2.0 HNF document model and `ToolAdapter` traits for Hardware Cloud Platform sidecars.

**Version pin (monorepo):** `0.1.0` (workspace). Future publish target: `hcp-adapters/hnf-adapter-sdk` on crates.io.

## Dependencies

- `sidecar-protocol` (path from HCP monorepo until protocol crate is published separately)

## Host OSS flags

Re-exported via `host_env`:

- `HCP_USE_HOST_OSS` — KiCad/FreeCAD subprocess bindings
- `HCP_SIM_USE_HOST` — simulation runners resolve solvers on `PATH`

See [OSS_HOST_DEPENDENCIES.md](../../../docs/OSS_HOST_DEPENDENCIES.md).
