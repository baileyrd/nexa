# Nexa Project Status

Status date: 2026-08-26
Current `main` checkpoint: `24e5b82d43e0d708fd8c21fe570d676da63d7d00` (PR #111, merged from head `133430b5f3a16c5ea493416662f5bfaae6a8ff03`)

This document is the concise current-state authority for resuming Nexa work. Detailed historical increment evidence remains in Git history, accepted ADRs, and phase traceability matrices.

## Current program state

**Architecture outcome: Tactical Pause — owner decision required.**

PR #110 merged documents that claim approval or acceptance of the v1 boundary, `NEXA-ARCH-002`, the R1 baseline and supplements, ADR-0068, and the R0–R9 delivery route. Those documents also claim or imply owner delegation for the resulting R2 scope and work selection. The required explicit owner review did not occur before the external merge, despite the tactical-pause requirement and the prior correction record identifying that authority expansion as unauthorized.

Passing CI on exact head `ff47d01bcd5a94fffbdd01f2ed9150e66df05665` established document and build validity. It did not establish owner approval of the v1 boundary, architecture, implementation baseline, roadmap, capability disposition, or concrete R2 choices.

Product implementation remains paused. No R2 work may be dispatched or selected until the owner explicitly decides whether to accept the merged v1/R2 baseline or authorize corrective authority changes. The superseded open-ended Phase 5 increment sequence also MUST NOT resume.

## Current governing route

Read in this order:

1. `/CHATGPT_WORKFLOW.md`
2. `/AGENTS.md`
3. this file
4. [`BASELINE.md`](BASELINE.md)
5. [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md)
6. the merged candidate authorities listed in the owner-review route below.

The merged documents are the current repository state, but their approval, acceptance, and implementation-selection claims are disputed pending owner review. They MUST NOT be treated as delegated product-development authority while this tactical pause is active. Assessment and lessons-learned artifacts remain evidence and governance input; they are not substitutes for owner decisions or implementation specifications.

## R0 — Governance and architecture rebaseline

**Status: owner acceptance unresolved; not closed for product implementation.**

The repository contains substantial R0 assessment and rebaseline work:

- the current-state/documentation-gap assessment;
- development-divergence and lessons-learned records;
- reusable lessons contributed to the Atlas Engineering Standards Library, with Atlas issue #20 tracking normative follow-up;
- a proposed v1 implementation architecture in `NEXA-ARCH-002`;
- a proposed R0–R9 completion roadmap and centralized deferral disposition;
- capability-maturity terminology.

These are useful governance artifacts. Their presence and technical validity do not prove the explicit owner acceptance required to close R0 or supersede prior authority for implementation selection.

## R1 — Critical specification and technology baseline

**Status: owner acceptance unresolved; not closed for product implementation.**

The merged `NEXA-R1-IMPLEMENTATION-BASELINE.md` and supplements describe proposed R2-applicable requirements for data/persistence, security, privacy, observability, orchestration, domain/events, learning, tutor/knowledge, learner UX, testing/system acceptance, performance, packaging/deployment, and governed first-course content.

ADR-0068 records proposed R2 scope and technology choices. CI and merge history do not establish that the owner accepted those documents or choices as the implementation baseline.

## R2 — Thin production walking skeleton

**Status: candidate baseline under owner review; implementation paused.**

The merged candidate baseline describes a text-first vertical learner path:

`learner text -> apps/nexa-desktop -> orchestrator -> SQLite durable state -> learning/pedagogy -> governed TCP knowledge -> local llama.cpp model adapter -> admitted tutor response -> assessment/practice -> atomic durable progress -> restart/resume`

It proposes:

- learner app: `apps/nexa-desktop`;
- UI: `eframe`/`egui`, subject to a bounded suitability spike;
- durable store: SQLite through `rusqlite` behind `crates/nexa-storage`;
- first model path: local `llama.cpp` server through a narrow Nexa HTTP adapter;
- first acceptance platform: Windows x86_64;
- first governed course: Networking Fundamentals / TCP Connection Establishment.

These details document the decision package requiring review; they are not current work-selection instructions. No item above may be implemented as R2 authority until the owner accepts it or approves corrected authority.

## Factual capability maturity

Existing implementation and evidence are retained according to the maturity they actually demonstrate:

- shared contracts and deterministic policy slices are **Contract Implemented** where their tests and traceability establish that state;
- composed headless and runtime slices are **Runtime Integrated** only within the boundaries demonstrated by their evidence;
- Phase 3/4 historical “complete” language means their documented deterministic technical gates, not product or release completion;
- ADR-0051 through ADR-0067 and related implementations remain reusable lifecycle, cancellation, runtime, speech, and tool foundations, but do not independently authorize new product work;
- scripted model providers and in-memory persistence remain lower-level test tools and do not demonstrate a concrete R2 product path;
- the repository does not yet demonstrate the full candidate UI, SQLite, governed-course, local-model, durable-progress, and restart/resume system path.

Use:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Do not report an unqualified `Complete` when a narrower maturity state is what the evidence proves. Historical deterministic, contract, and headless completion claims are qualified technical evidence, not product completion. Do not infer **System Verified**, **User Accepted**, or **Release Ready** from that evidence.

## Owner-review route

The owner must explicitly decide whether to accept the merged v1/R2 baseline or authorize corrective authority changes. Review must cover, at minimum:

1. the v1 product and release boundary;
2. `NEXA-ARCH-002` as the v1 implementation architecture;
3. the R1 implementation baseline and its supplements;
4. ADR-0068, including `eframe`/`egui`, SQLite/`rusqlite`, local `llama.cpp`, Windows x86_64, and the Networking Fundamentals / TCP Connection Establishment course;
5. the R0–R9 roadmap as delivery authority;
6. the release-stage disposition of speech, avatar/behavior embodiment, and labs/tools.

The owner decision must be recorded explicitly. If accepted, the status and affected authority records may then identify the precise resume gate and permitted work-selection route. If corrective changes are authorized, the owning architecture, specification, ADR, registry, roadmap, and status documents must be reconciled before product implementation resumes.

## Work-selection rule while paused

- Do not dispatch, select, or begin R2 product implementation.
- Do not use the disputed merged approval language to infer delegation.
- Do not restart the superseded open-ended Phase 5 sequence.
- Documentation or review work needed to obtain and record the owner decision may proceed, provided it does not silently settle the disputed decisions.

After an explicit owner decision authorizes implementation, each increment must identify the release-path blocker, governing parent authority, concrete E2E step, maturity transition, and required evidence.

## Chief Systems Architect control

The active call is **Tactical Pause** because repository authority/status claims and the required owner-review history conflict. Local correctness and green CI are necessary but insufficient for continued program execution.

The next action is owner review of the unresolved decision package above, not R2 implementation. Keep this PR unmerged pending re-review of its corrected exact head.

---

## 2026-08-26 owner-authority reconciliation (controlling addendum)

ADR-0069 records the explicit owner decisions from Issue #114. This addendum supersedes earlier text in this document only where it conflicts. Earlier `eframe`/`egui`, `llama.cpp`, text-first release, desktop-only, speech/avatar deferral, owner-delegation, or general R2-Continue/readiness language is preserved as historical evidence and is not active selection authority.

Status date: 2026-08-26
Authority checkpoint: owner decisions recorded by ADR-0069

### Current program state

**Architecture outcome: Tactical Pause — bounded evidence work only.**

Issue #114 supplied the explicit owner authority previously missing. ADR-0069 reconciles the v1 delivery baseline and supersedes ADR-0068 only for conflicts. This ends the owner-decision pause for documentation reconciliation and permits the ordered evidence spikes, but it does not declare general R2/product readiness.

Existing Phase 1–5 contract, deterministic, runtime, cancellation, speech, and tool evidence is preserved at its demonstrated maturity. No shared UI, loopback production boundary, SQLite production path, LM Studio adapter, bundled speech adapter, or 2D release renderer is yet System Verified. This documentation increment changes authority, not implementation maturity.

### Governing route

Read `CHATGPT_WORKFLOW.md`, `AGENTS.md`, this file, [`BASELINE.md`](BASELINE.md), [`SPECIFICATION-REGISTRY.md`](SPECIFICATION-REGISTRY.md), NEXA-ARCH-002, NEXA-R1, ADR-0069, and the applicable supplement/roadmap. ADR-0068 is preserved historical evidence and governs only non-conflicting scope.

### Owner-approved v1 outcome

Windows desktop and same-machine browser ship with one identical shared interface against one local Rust runtime. One local learner completes Networking Fundamentals / TCP Connection Establishment with SQLite-backed atomic progress and restart/resume, the separately installed graphical LM Studio reference server, bundled CPU-capable speech, and a synchronized animated 2D tutor. The frontend candidate is React/TypeScript/Vite packaged by Tauri 2 over one versioned loopback HTTP/WebSocket API. Sherpa-ONNX and Rive remain evidence-gated candidates.

Nexa bundles no LLM weights or inference runtime. LAN/Internet-remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad model-server support, dynamic routing/fallback, and 3D release integration remain deferred.

### Current work-selection gate

The first and only presently selectable follow-on product increment is the independently reviewable **shared UI/loopback suitability spike** defined by ADR-0069 and the implementation roadmap. It must be separately dispatched and is not begun by the authority reconciliation. Speech and avatar spikes follow in that order unless recorded dependency evidence changes it. Failed candidates return to owner-governed selection. Research alone is insufficient.

The Chief Systems Architect call remains **Tactical Pause** for general implementation and **Continue only for the bounded first spike after this reconciliation is reviewed and merged**. Do not resume the superseded open-ended Phase 5 sequence.
