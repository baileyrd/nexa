# Nexa v1 Technology Recommendations — Research Record

Status: Reviewed research basis; decisions adopted where stated by ADR-0068
Research checkpoint: 2026-08-26

The tactical-pause technology review evaluated a deliberately small R2 technology spine for a local-first Rust desktop tutor.

The governing decisions are now recorded in:

[`../adr/0068-v1-r2-walking-skeleton-baseline.md`](../adr/0068-v1-r2-walking-skeleton-baseline.md)

ADR-0068 adopts for R2:

- SQLite via `rusqlite` behind `nexa-storage`;
- `eframe`/`egui` for the learner desktop path subject to a bounded suitability spike;
- a local `llama.cpp` server behind a narrow Nexa model adapter;
- Windows x86_64 as the first acceptance environment;
- Networking Fundamentals / TCP Connection Establishment as the first governed course;
- text-first scope, with speech/avatar/labs outside the R2 exit gate.

The original full research/rationale is preserved in Git history. Future implementation must cite ADR-0068 and the owning R1 specifications rather than treating this research record as independent authority.

Technology capabilities and upstream projects evolve. Concrete implementation PRs must record the exact versions/configurations they validate when those dependencies enter the workspace or release path.
