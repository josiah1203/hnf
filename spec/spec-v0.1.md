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

JSON schema stubs: [`schemas/domains/`](../schemas/domains/).

## Phase 1 domains (planned M5-B)

| Domain ID | Description |
|-----------|-------------|
| `bim` | Building information modeling (IFC) |
| `geospatial` | GIS / site context |
| `structural` | Structural analysis models |
| `energy_building` | Building energy (OpenStudio / EnergyPlus) |

## Structural diff

Phase 0: path-level tree diff in HOS (`legacy` merge engine).  
Phase 0 M3: object-graph semantic diff (`HOS_MERGE_ENGINE=semantic`).

## Validation

- **Rust:** `hnf_core::validate(&HnfManifest)` — required manifest fields; `parse_schematic` / `parse_bom` — domain payloads
- **Cloud:** `hbp-cloud/api/app/services/hnf.py` — document body + upload hints (additive warnings)

## CRDT / collaboration

See [`docs/CRDT_SPIKE.md`](../docs/CRDT_SPIKE.md) for real-time edit sync library evaluation.
