# Nexa Project Status

Status date: 2026-08-26
Current `main` checkpoint: `4369a85410c3dcb5ffcb619ef1c2a04bed08978f` (PR #110)

This document is the concise current-state authority for resuming Nexa work. Detailed historical increment evidence remains in Git history, accepted ADRs, and phase traceability matrices.

## Current program state

**Architecture outcome: Continue — begin R2.**

The tactical-pause assessment, R0 governance/architecture rebaseline, and R1 implementation-specification convergence are complete to the maturity required for the R2 walking skeleton.

PR #110 merged the reviewed rebaseline after required CI passed on exact head `ff47d01bcd5a94fffbdd01f2ed9150e66df05665`. Product implementation is no longer blocked by the tactical pause.

The superseded open-ended Phase 5 increment sequence MUST NOT resume. New work is selected from the R2 vertical release path.

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

Assessment and lessons-learned artifacts remain evidence and governance input; they are not substitutes for implementation specifications.

## R0 — Governance and architecture rebaseline

**Status: PASS / closed for R2.**

- Current-state/documentation-gap assessment completed.
- Development divergence and lessons learned recorded.
- Reusable lessons merged into the Atlas Engineering Standards Library; Atlas issue #20 tracks normative follow-up.
- `NEXA-ARCH-002` is the approved v1 implementation architecture.
- `NEXA-ARCH-001` is preserved as reconstructed provenance/long-range context and superseded for v1 implementation selection.
- R0–R9 completion roadmap is the delivery authority.
- Deferrals are centralized and stage-dispositioned.
- Capability maturity terminology is established.

## R1 — Critical specification and technology baseline

**Status: PASS / closed for R2.**

`NEXA-R1-IMPLEMENTATION-BASELINE.md` approves the R2-applicable parent/cross-cutting requirements for:

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

ADR-0068 resolves the R2 scope and technology baseline.

R1 intentionally does not finish every long-term Nexa specification; later-release requirements mature at their owning R3–R9 gates.

## R2 — Thin production walking skeleton

**Status: READY / next implementation stage.**

R2 is text-first and builds one real vertical learner path:

`learner text -> apps/nexa-desktop -> orchestrator -> SQLite durable state -> learning/pedagogy -> governed TCP knowledge -> local llama.cpp model adapter -> admitted tutor response -> assessment/practice -> atomic durable progress -> restart/resume`

ADR-0068 selects:

- learner app: `apps/nexa-desktop`;
- UI: `eframe`/`egui`, subject to a bounded suitability spike;
- durable store: SQLite through `rusqlite` behind `crates/nexa-storage`;
- first model path: local `llama.cpp` server through a narrow Nexa HTTP adapter;
- first acceptance platform: Windows x86_64;
- first governed course: Networking Fundamentals / TCP Connection Establishment.

R2 exit requires actual UI, actual SQLite, actual governed course content, actual local model adapter, admitted tutor response, durable assessment/evidence/mastery/progress, restart/resume, and content-safe correlation.

Scripted model providers and in-memory persistence remain lower-level test tools and cannot close R2.

## Not R2 blockers

Unless a concrete R2 failure promotes them:

- speech input/output;
- animated avatar/behavior integration;
- labs/tool execution;
- remote provider support/credentials;
- dynamic multi-provider routing/fallback;
- dedicated vector database;
- durable event broker/outbox;
- exact release GGUF model/quantization;
- final Windows installer/signing/update mechanism;
- plugins/public API/analytics/authoring/server/fleet deployment.

These are owned by later roadmap stages.

## Existing Phase 1–5 implementation

Existing code is retained according to the maturity it actually demonstrates.

- Contract/headless/conformance foundations remain reusable.
- Phase 3/4 historical “complete” language means their documented deterministic technical gates, not product/release completion.
- ADR-0051 through ADR-0067 remain reusable lifecycle/cancellation/runtime/speech/tool foundations, but only enter implementation work when the actual R2/R3 learner path needs them.

## Capability maturity vocabulary

Use:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Do not report an unqualified `Complete` when a narrower maturity state is what the evidence proves.

## R2 work-selection rule

Every implementation increment must identify:

1. the R2 release-path blocker it addresses;
2. its governing parent architecture/specification/ADR;
3. the E2E step it makes concrete;
4. the maturity transition it advances;
5. the evidence required to substantiate that transition.

Do not select work merely because another narrow ADR can be written.

## Chief Systems Architect control

At every major stage boundary, and whenever parent maturity, deferrals, horizontal depth, status consistency, or product convergence materially diverge, the Chief Systems Architect performs a whole-system review and records:

- Continue;
- Redirect;
- Tactical Pause.

Local correctness is necessary but not sufficient for continued program execution.

## Current next action

Begin R2 with the smallest vertical-enabling increment:

- activate the real `apps/nexa-desktop` application shell;
- activate the real `crates/nexa-storage` SQLite/`rusqlite` infrastructure boundary;
- prove the egui async-safe application seam and SQLite open/migration/transaction architecture;
- enforce UI/storage dependency boundaries;
- add focused store/application startup tests;
- add Windows release-critical validation proportional to the new surfaces.

Then continue immediately down the vertical path: durable learning state -> governed content/retrieval -> local model adapter -> tutor/UI composition -> assessment/progress/restart E2E.