# CRDT library spike (M0 — Week 4)

**Owners:** `collab` + `coordinator`  
**Status:** **Decided** — see [ADR-0001 — CRDT store](https://github.com/hummingbird-labs/hb-platform/blob/main/docs/ADR/0001-crdt-store.md)

## Candidates (evaluated)

| Library | License | Notes |
|---------|---------|-------|
| [Automerge-rs](https://github.com/automerge/automerge) | MIT | Mature CRDT doc model; WASM + native bindings; heavier payload — **Phase 0.5 primary** |
| [Diamond Types](https://github.com/josephg/diamond-types) | Apache 2.0 / MIT | Text-optimized; lower overhead; less turnkey for structured HNF objects — **fallback candidate** |

## Evaluation criteria

1. **Auth boundary** — server-side validation of CRDT ops scoped by `org_id` / `project_id` ✅ (implemented in `hbp-cloud` collaboration service)
2. **Object graph fit** — map HNF object snapshots vs flat text → deferred to Phase 0.5 Automerge integration
3. **Wire size** — heartbeat + op batch size under collab soak p95 targets → baseline with Redis LWW; re-measure at 0.5
4. **Rust integration** — sidecar in `hbp-cloud` vs pure Python service → Python service + Redis today; Rust sidecar optional at 0.5

## Decision

**Phase 0:** Ship **Redis LWW op log + Postgres durable rows** as documented in ADR-0001, matching `hbp-cloud/api/app/services/crdt_store.py`.

**Phase 0.5:** Integrate **Automerge** (or Diamond for text paths if soak fails) via `envelope.crdt_payload`; replace LWW path merge.

## References

- ADR: `hb-platform/docs/ADR/0001-crdt-store.md`
- Cloud notes: `hbp-cloud/docs/CRDT_SPIKE.md`
- v8 plan M0 / M3 collaboration gates
