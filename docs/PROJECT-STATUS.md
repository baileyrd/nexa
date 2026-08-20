# Nexa project status

Verified against repository `main` at merge commit `624ca78251064b1061de7aac805851625c12f08b` (PR #19). This is a checkpoint, not an additional specification or ADR.

## Verified checkpoint

- **Phase 0:** the reconstructed baseline, specification registry, roadmap, accepted ADR series, and CI quality gates exist. The registry still records baseline audit work; documentation is therefore not claimed complete.
- **Phase 1:** the contract kernel is implemented in `nexa-domain`, `nexa-events`, and `nexa-nbp`, with canonical values, event envelopes, NBP messages, wire fixtures, and boundary enforcement.
- **Phase 2:** complete. `nexa-avatar` owns renderer-neutral ports; `nexa-3d` supplies the headless runtime/adapter; `nexa-3d-validate` is the headless validator; and `nexa-3d-viewer` is the GPU/window composition root.
- **Phase 3:** complete for the deterministic headless exit gate. Student evidence/replay, pedagogy policy, curriculum transitions, assessment/scoring, and atomic learning-core composition are implemented. Durable production persistence and outbox behavior are not implied.
- **Phase 4:** in progress. `nexa-knowledge` implements governed ingestion, lexical and vector retrieval, hybrid fusion, whole-chunk context assembly, and deterministic citation resolution. PR #19 added `nexa-tutor` provider-neutral deterministic response planning; its latest merged milestone is merge commit `624ca78251064b1061de7aac805851625c12f08b`.

The current Cargo workspace contains the contract kernel; avatar/3D; student, pedagogy, lessons, assessment, and learning-core; knowledge and tutor crates; plus the 3D viewer and validator. Reserved `.gitkeep`-only directories are not implemented capabilities. Dependency direction and exact allowed normal edges are enforced by [`scripts/check-contract-boundaries.sh`](../scripts/check-contract-boundaries.sh).

## Authorities and evidence

- Governance and precedence: [`BASELINE.md`](BASELINE.md) and the [`specification registry`](SPECIFICATION-REGISTRY.md).
- Delivery gates: [`IMPLEMENTATION-ROADMAP.md`](architecture/IMPLEMENTATION-ROADMAP.md).
- Implemented-slice evidence: [Phase 1](architecture/PHASE-1-TRACEABILITY.md), [Phase 2](architecture/PHASE-2-TRACEABILITY.md), [Phase 3](architecture/PHASE-3-TRACEABILITY.md), and [Phase 4](architecture/PHASE-4-TRACEABILITY.md) traceability.
- Cross-cutting decisions: the [accepted ADRs](adr/), including [ADR-0021](adr/0021-provider-neutral-tutor-response-planning.md) for the PR #19 boundary.
- Reconstructed design sources remain governed source evidence; their authority/status is stated in the registry.

## Recorded gaps and unresolved decisions (verified)

- Phase 1 defers the remaining domain inventory, durable event replay/store semantics, privacy retention, async backpressure, broader payloads, NBP arbitration/update/canvas behavior, and formal schema compatibility automation.
- Phase 3 defers durable adapter choices and transaction, concurrency, authorization, retention, recovery, migration, and outbox semantics; its traceability matrix records further policy/content ambiguities.
- Phase 4 defers learned/semantic reranking, partial truncation, provider tokenization, semantic citation fidelity/entailment and safety, generative tutor intelligence, provider integration, networking, vector databases, and durable adapters.
- Phases 5 and 6 (complete session orchestration; authoring, packaging, and operations) have not begun according to the roadmap.
- Registry baseline work remains open: reconstruction formatting normalization, duplicate/relocated 3D material reconciliation, dependency-declaration audit, fuller ownership/acceptance/conformance links, and explicit specification promotion review.
- **Documentation inconsistency:** Phase 4 traceability verifies ingestion and context assembly as implemented, while their roadmap bullets remain unchecked. Treat the traceability evidence and phase status text as the implementation checkpoint; do not silently edit the checklist without focused review.

## Next incomplete increment

**Verified roadmap fact:** Phase 4 is the active phase, and its next incomplete capability area after the implemented response-planning slice is model-provider abstraction and safety gates; ADR-0021 explicitly leaves semantic safety, generation, providers, and durable adapters deferred.

**Recommendation (not a decision):** open a bounded Phase 4 task that selects one of those explicitly deferred capabilities, first resolving its authoritative requirements and ADR needs. Do not bundle provider selection, semantic validation, generation, networking, or persistence into one increment.

## How to resume work

1. Start from the current `main`; read `AGENTS.md`, the baseline, registry, roadmap, Phase 4 traceability, and ADR-0021.
2. Confirm PR #19 / `624ca78251064b1061de7aac805851625c12f08b` is present and inspect intervening merges.
3. Choose one evidence-backed incomplete Phase 4 increment, state its exclusions, and identify specification/ADR/traceability impact before coding.
4. Keep the PR independently reviewable, report documentation/implementation conflicts, and run every validation command in `AGENTS.md`.
