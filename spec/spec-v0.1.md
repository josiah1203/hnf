# HNF specification v0.1

**HummingBird Native Format** — canonical interchange for hardware and built-environment artifacts.

- **Version:** 0.1 (Phase 0 internal alpha)
- **Encoding:** JSON documents + content-addressed blobs (HNFT)
- **Schemas:** `schemas/domains/<domain>.json`

## Document envelope

Every HNF document includes:

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

Reference implementation: `crates/hnf-core` (`cargo test`). Cloud validation: `hbp-cloud/api/app/services/hnf.py`.

## CRDT / collaboration

See [`docs/CRDT_SPIKE.md`](../docs/CRDT_SPIKE.md) for real-time edit sync library evaluation.
