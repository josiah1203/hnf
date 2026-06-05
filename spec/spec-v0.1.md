# HNF specification v0.1

**HummingBird Native Format** — canonical interchange for hardware and built-environment artifacts (HummingBird v8, Phase 0).

- **Version:** 0.1 (Phase 0 internal alpha; aligns with v8 manifest `hnf_version`)
- **Encoding:** JSON documents + content-addressed blobs (HNFT)
- **Schemas:** `schemas/domains/<domain>.json`
- **Reference validation:** `crates/hnf-core` (`validate`, `cargo test`)

## Layer model (v8 core)

HNF documents are layered so bridges, HOS, and merge can share one envelope:

| Layer | Purpose | Required in v0.1 |
|-------|---------|-------------------|
| **Manifest** | Identity, schema version, active disciplines | `hnf_version`, `doc_id`, `disciplines` |
| **Objects** | Typed graph nodes (components, nets, solids, …) | Per-domain payloads under `objects` |
| **Refs** | Cross-object and cross-domain edges | On each object envelope |
| **Blobs** | Content-addressed geometry, netlists, GDS | HNFT sidecar refs (`content_hash`) |

```
┌─────────────────────────────────────────┐
│ manifest (hnf_version, doc_id, …)       │
├─────────────────────────────────────────┤
│ objects[]  ──refs──►  other objects     │
│     │                                   │
│     └──► blob refs (content_hash)       │
└─────────────────────────────────────────┘
```

Cloud persistence maps manifest + object snapshots to HOS commits; see `hbp-cloud/api/app/services/hnf.py`.

## Manifest (required)

Every top-level HNF package **must** include a `manifest` object. `hnf-core::validate` rejects documents missing or invalid manifest fields.

```json
{
  "manifest": {
    "hnf_version": "0.1",
    "doc_id": "550e8400-e29b-41d4-a716-446655440000",
    "disciplines": ["schematic", "layout", "bom"],
    "created_at": "2026-06-04T12:00:00Z",
    "schema_revision": "0.1.0"
  },
  "document_uri": "hbp://projects/demo/board",
  "objects": []
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `hnf_version` | string | yes | HNF spec version; v0.1 documents use `"0.1"` |
| `doc_id` | string | yes | Stable document identifier (UUID recommended) |
| `disciplines` | string[] | yes | Non-empty list of active domain IDs (subset of Phase 0/1 tables) |
| `created_at` | string (ISO-8601) | no | Creation timestamp |
| `schema_revision` | string | no | Semver of tooling-specific schema extensions |

`disciplines` entries must be known domain IDs for strict tooling; Phase 0 validators accept any non-empty string and recommend the tables below.

## Document envelope (object-level)

Domain payloads use a shared object envelope (unchanged from early v0.1 stubs):

```json
{
  "domain": "<domain_id>",
  "version": "0.1",
  "hnf_type": "hardware.object",
  "object_id": "<uuid>",
  "content_hash": "<sha256>",
  "refs": [],
  "properties": {}
}
```

## Phase 0 domains (v0.1)

| Domain ID | Description | Primary tools |
|-----------|-------------|---------------|
| `schematic` | Schematic capture | KiCad, Xschem |
| `layout` | PCB layout | KiCad |
| `ic_layout` | IC layout | KLayout, Magic, OpenROAD |
| `mechanical` | MCAD / mechanical | FreeCAD |
| `simulation` | SPICE, EM, FEA netlists | ngspice, OpenEMS, Elmer, Qucs-S |
| `bom` | Bill of materials | derived / Bridge |
| `firmware` | Embedded artifacts | PlatformIO, Verilator, Yosys |

Full JSON schemas: [`schemas/domains/`](../schemas/domains/). Rust validators and fixtures: `crates/hnf-core`.

### Domain payloads (implemented in `hnf-core`)

| Domain | `properties` (v0.1) | HNFT blob refs |
|--------|----------------------|----------------|
| `schematic` | `symbols`, `nets`, `pins`, `power_domains` | optional `content_hash` on envelope |
| `layout` | `footprints`, `tracks` (≥1 required) | envelope `content_hash` → PCB geometry/copper blobs |
| `ic_layout` | `layers` (≥1), `shapes` with `bbox` | envelope → GDS/OASIS HNFT |
| `mechanical` | `solids` (≥1), `constraints` | envelope → STEP/BREP mesh HNFT |
| `simulation` | `models` (≥1), `probes` | per-model `netlist_hash` → SPICE/EM/FEA deck HNFT |
| `bom` | `lines` (≥1) | optional envelope hash |
| `firmware` | `targets` (≥1), `sources` and/or `artifacts` | `artifacts[].content_hash` → firmware binary HNFT |

HNFT (HummingBird Native Format **T**ransport) sidecars are content-addressed blobs referenced by 64-character SHA-256 `content_hash` fields. Object envelopes may also carry top-level `content_hash` when the whole domain body is stored out-of-line.

## Phase 1 domains (planned M5-B)

| Domain ID | Description |
|-----------|-------------|
| `bim` | Building information modeling (IFC) |
| `geospatial` | GIS / site context |
| `structural` | Structural analysis models |
| `energy_building` | Building energy (OpenStudio / EnergyPlus) |

## Structural diff

`hnf-core::diff` (Phase 0 M0):

| Domain | Strategy | Notes |
|--------|----------|-------|
| `schematic` | `object_graph` | Entities keyed by `id` (symbols, nets, power_domains) or `symbol_id:pin_number` (pins) |
| `bom` | `object_graph` | Lines keyed by `line_id` |
| `layout`, `ic_layout`, `mechanical`, `simulation`, `firmware` | `path_fallback` | JSON path tree diff until domain-specific graph diff lands |

HOS merge: path-level tree diff in `legacy` engine; object-graph semantic merge for schematic/bom bodies when `HOS_MERGE_ENGINE=semantic` (M3).

## Validation

- **Rust:** `hnf_core::validate(&HnfManifest)` — manifest; `parse_domain` / `parse_<domain>` — all seven Phase 0 domains; `diff_domain` — structural compare
- **Cloud:** `hbp-cloud/api/app/services/hnf.py` — document body + upload hints (additive warnings)

## CRDT / collaboration

See [`docs/CRDT_SPIKE.md`](../docs/CRDT_SPIKE.md) for real-time edit sync library evaluation.
