# Nexa Project Status

Status date: 2026-08-26
Tactical-pause implementation baseline: `e2345a1bb8825451ea079ff5e350b7765075038a` (PR #109)
Rebaseline work: PR #110

This document is the concise current-state authority for resuming Nexa work. Detailed historical increment evidence remains in Git history, accepted ADRs, and phase traceability matrices.

## Current program state

The tactical-pause assessment and architecture convergence are complete to the level required to govern the R2 walking skeleton.

Normal product implementation remains paused **only until PR #110 is green on its exact final head and merged**. After that merge, R2 may begin. The old open-ended Phase 5 increment sequence must not resume.

## Current governing route

Read in this order:

1. `/CHATGPT_WORKFLOW.md`
2. `/AGENTS.md`
3. this file
4. [`BASELINE.md`](BASELINE.md)
5. [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md)
6. [`architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
7. [`architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md`](architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md)
8. [`adr/0068-v1-r2-walking-skeleton-baseline.md`](adr/0068-v1-r2-walking-skeleton-baseline.md)
9. [`architecture/NEXA-COMPLETION-ROADMAP.md`](architecture/NEXA-COMPLETION-ROADMAP.md)
10. the applicable parent specification, accepted ADRs, and traceability evidence for the selected increment.

Assessment/lessons artifacts remain important evidence but are not implementation specifications.

## Rebaseline decisions

### Product boundary

Nexa v1 is a local-first adaptive tutor whose first release is proven by one complete learner journey, not by contract count or phase labels.

The first vertical path is:

`learner text -> desktop app -> orchestrator -> learning state/pedagogy -> governed knowledge -> real local model -> admitted tutor response -> assessment/practice -> atomic durable progress -> restart/resume`

### R2 concrete baseline

ADR-0068 selects:

- learner app: `apps/nexa-desktop`;
- UI: `eframe`/`egui`, subject to a bounded suitability spike;
- durable store: SQLite through `rusqlite` behind `crates/nexa-storage`;
- first model path: local `llama.cpp` server through a narrow Nexa HTTP adapter;
- first acceptance platform: Windows x86_64;
- first governed course: Networking Fundamentals / TCP Connection Establishment;
- text-first learner interaction;
- no R2 requirement for speech, avatar, labs/tools, dynamic multi-provider routing, vector database, or durable event broker.

### Conditional capabilities

Speech and avatar/behavior remain retained architectural capabilities but are deferred from the R2 exit gate. They are reconsidered after the walking skeleton at the later embodiment/speech integration stage.

Labs/tool execution and dynamic multi-provider routing are post-R2 by default.

This is scope control, not abandonment.

## Architecture maturity

### R0 — Rebaseline governance and architecture

**Status: Complete for R2.**

- Current-state/documentation-gap assessment completed.
- Development divergence and lessons learned recorded.
- Reusable lessons captured in the Atlas Engineering Standards Library; Atlas issue #20 tracks normative follow-up.
- `NEXA-ARCH-002` is the approved v1 implementation architecture.
- `NEXA-ARCH-001` is preserved as reconstructed provenance/long-range context and superseded for v1 implementation selection.
- A finite R0–R9 completion roadmap exists.
- Deferrals are centralized and reviewed by release stage.
- Capability maturity terminology is established.

### R1 — Critical specification baseline

**Status: Complete to the maturity required to begin R2.**

`NEXA-R1-IMPLEMENTATION-BASELINE.md` approves the tactical-pause R1 supplements for the R2 path and resolves the blocking decisions for:

- data/persistence;
- security;
- privacy;
- observability;
- complete session orchestration;
- domain/events;
- learning subsystems;
- tutor/knowledge;
- learner UX;
- testing/system acceptance;
- performance measurement;
- packaging/deployment constraints;
- governed first-course content.

The R1 baseline intentionally leaves later-release details open where they do not govern R2.

### R2 — Thin production walking skeleton

**Status: Ready to start after PR #110 is green on its final exact head and merged.**

R2 exit requires one real end-to-end learner lesson using:

- actual desktop UI boundary;
- actual SQLite persistence;
- actual governed TCP lesson content;
- actual local llama.cpp model adapter;
- existing learning/tutor/knowledge contracts where applicable;
- atomic progress commit;
- restart/resume;
- content-safe correlated diagnostics.

Scripted model providers and in-memory persistence do not satisfy the R2 exit gate.

## Reclassification of existing implementation

The existing Phase 1–5 code is retained. Its maturity is interpreted by what the evidence actually proves.

### Contract kernel / embodiment / learning / knowledge / tutor

Substantial contract, deterministic, headless, and conformance foundations exist and remain valuable. Phase traceability matrices document them.

Earlier statements such as “Phase 3 complete” or “Phase 4 complete” are understood as qualified technical-gate statements, not product-completion claims.

### Phase 5 cancellation/control work

ADR-0051 through ADR-0067 provide reusable lifecycle, structured concurrency, cancellation/control, retrieval, speech-control, tool-control, and headless binding foundations.

They do not define the work-selection sequence after the rebaseline. Reuse them only where the R2/R3 learner workflow requires them.

## Capability maturity vocabulary

Use the following status vocabulary for new work:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Do not report an unqualified `Complete` when the demonstrated evidence is one of these narrower states.

## Current non-R2 blockers / later decisions

The following are intentionally unresolved without blocking R2 start:

- exact release GGUF model/quantization;
- final llama.cpp bundling/distribution policy;
- final Windows installer/signing/update mechanism;
- remote provider support and credential mechanism;
- speech/avatar final v1 inclusion decision;
- lab/tool sandbox implementation;
- durable asynchronous event/outbox architecture;
- advanced vector infrastructure;
- multi-provider routing/fallback;
- plugins, public API, analytics, authoring, server/fleet deployment.

Each becomes blocking only at the roadmap stage that owns its release outcome.

## Development selection rule

After PR #110 merges, every implementation increment must trace to:

1. an R2 release-path blocker;
2. an explicit requirement in `NEXA-ARCH-002` / the R1 baseline / ADR-0068; and
3. a concrete maturity advance toward the R2 E2E acceptance scenario.

Do not select work merely because another narrow ADR can be written.

## Chief Systems Architect control

At every major stage boundary, and whenever documentation maturity, deferrals, horizontal depth, or product convergence materially diverge, the Chief Systems Architect must perform a whole-system rebaseline review and record one outcome:

- Continue;
- Redirect;
- Tactical Pause.

Local correctness is necessary but not sufficient for continued program execution.

## Current next action

1. Finalize/review PR #110 exact diff.
2. Require Rust CI green on the exact final PR head.
3. Merge PR #110 unchanged.
4. Begin R2 with the smallest vertical increment that establishes the `apps/nexa-desktop` application shell plus concrete `nexa-storage`/SQLite foundation needed by the walking skeleton, without re-entering unrelated Phase 5 horizontal work.