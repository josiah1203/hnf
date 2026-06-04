# CRDT library spike (M0 — Week 4)

**Owners:** `collab` + `coordinator`  
**Status:** Open — decision pending

## Candidates

| Library | License | Notes |
|---------|---------|-------|
| [Automerge-rs](https://github.com/automerge/automerge) | MIT | Mature CRDT doc model; WASM + native bindings; heavier payload |
| [Diamond Types](https://github.com/josephg/diamond-types) | Apache 2.0 / MIT | Text-optimized; lower overhead; less turnkey for structured HNF objects |

## Evaluation criteria

1. **Auth boundary** — server-side validation of CRDT ops scoped by `org_id` / `project_id`
2. **Object graph fit** — map HNF object snapshots vs flat text
3. **Wire size** — heartbeat + op batch size under collab soak p95 targets
4. **Rust integration** — sidecar in `hbp-cloud` vs pure Python service

## Decision (TBD)

Record ADR in `hb-platform/docs/ADR/` when spike completes. Until then, collaboration remains the polling stub in `hbp-cloud/api/app/services/collaboration.py`.

## References

- v8 plan M0 / M3 collaboration gates
- `hbp-cloud/docs/CRDT_SPIKE.md` (cloud-side notes)
