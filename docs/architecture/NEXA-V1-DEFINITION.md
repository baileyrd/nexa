# Nexa v1 Product Definition and Release Boundary

Status: Approved
Date: 2026-08-26
Governing architecture: `NEXA-ARCH-002`

## Purpose

This document gives Nexa a finite first-release target. Nexa v1 is defined by an observable learner outcome, not by the number of subsystem contracts, crates, ADRs, or technical gates completed.

## v1 mission

A learner can launch Nexa locally, enter or resume a governed course, interact with the tutor, receive grounded adaptive instruction, complete assessment/practice, have competency progress updated durably, exit, restart, and resume without loss or duplication of accepted work.

Release-critical boundaries use real/concrete dependencies rather than scripted test doubles.

## Primary learner journey

1. Install/launch Nexa on the supported desktop target.
2. Select or resume a locally available governed course.
3. Load learner, lesson, and governed knowledge state.
4. Submit text input through the learner application.
5. Establish one orchestrated interaction workflow.
6. Derive learning/pedagogy context and retrieve governed knowledge.
7. Invoke one configured concrete model path.
8. Structurally admit and release-quality-check the tutor response.
9. Present the response through the learner UI.
10. Complete governed assessment/practice.
11. Record evidence, replay/update mastery, and apply the next authored/adaptive route.
12. Commit progress durably.
13. Exit and restart.
14. Resume the accepted state correctly.
15. Complete the bounded lesson acceptance outcome.

## Required v1 capability families

### Learner application

- one supported desktop application/composition root;
- course/lesson start and resume;
- text input and tutor response presentation;
- assessment/practice and progress presentation;
- understandable recoverable/terminal failures;
- supported accessibility baseline.

### Session orchestration

- complete learner interaction composition;
- work ownership/cancellation;
- bounded timeout/failure/recovery policy;
- explicit commit points;
- safe shutdown and restart.

### Learning

- authored curriculum/lesson progression;
- immutable evidence and replayable mastery;
- adaptive pedagogy;
- deterministic assessment/scoring where specified;
- atomic durable learning progress.

### Tutor/model

- one concrete supported model adapter;
- governed prompt/context construction;
- strict structured output admission;
- grounding/citation quality appropriate to the released course;
- bounded failures with no hidden fallback.

Multi-provider dynamic routing is not required for the first release.

### Knowledge

- durable governed content/provenance;
- production-style ingestion for the first course format;
- retrieval/context/citation sufficient for the released course;
- provenance that survives restart.

A dedicated vector database is not required unless measured release evidence justifies it.

### Data/persistence

Persist and recover at minimum:

- learner identity/profile fields required by v1;
- authored course/lesson version identity;
- lesson progress;
- assessment state required for progression;
- immutable learning evidence;
- mastery/replay metadata;
- knowledge provenance;
- schema/recovery metadata needed for safe startup.

### Security/privacy

- least-privilege local operation;
- explicit trust boundaries;
- no secret/raw learner/prompt/model/source content in normal diagnostics;
- remote disclosure only through an explicitly approved path if remote inference is later supported;
- learner-data retention/reset/deletion mechanisms appropriate to v1.

### Observability/recovery

- structured content-safe logging/correlation;
- dependency failure visibility;
- timeout/cancellation/recovery evidence;
- startup/shutdown/restart diagnostics.

### Packaging/release

Before Release Ready:

- reproducible supported build;
- installation/distribution path for at least one desktop target;
- data migration/upgrade strategy;
- version/provenance reporting;
- third-party license/runtime/model/asset disposition;
- release acceptance evidence.

## R2 concrete first path

ADR-0068 establishes the first walking-skeleton implementation baseline:

- `apps/nexa-desktop`;
- `eframe`/`egui` after a bounded suitability spike;
- SQLite/`rusqlite` behind `nexa-storage`;
- local `llama.cpp` server adapter;
- Windows x86_64 acceptance environment;
- Networking Fundamentals / TCP Connection Establishment course;
- text-first learner interaction.

## Not R2 exit criteria

The following do not block the R2 walking skeleton:

- speech input/output;
- animated avatar/behavior integration;
- labs/tool execution;
- dynamic multi-provider routing/fallback;
- remote-provider credentials/disclosure;
- dedicated vector database;
- final signed installer/update mechanism.

They are reconsidered at their owning later roadmap gates.

## System verification requirements

Before v1 Release Ready, evidence must include:

- automated primary learner E2E path;
- durable restart/resume;
- storage/model/retrieval/cancellation/recovery failure tests;
- security/privacy review of actual data flows;
- measured performance on the accepted release environment;
- supported packaging/install/upgrade validation;
- representative user acceptance.

## Explicit non-definition

Nexa v1 is not complete merely because:

- all current unit tests pass;
- an old phase traceability matrix is green;
- provider-neutral contracts exist;
- every planned crate/directory exists;
- every ADR is accepted;
- a headless/scripted composition demonstrates deterministic behavior.

Those are evidence inputs at specific maturity levels, not product acceptance.

## v1 acceptance statement

Nexa v1 is Release Ready only when a new user can use the supported build to complete the primary learner journey with real configured dependencies, exit, restart, resume correct durable state, and pass the release acceptance suite with no unresolved critical architecture, security, privacy, data-integrity, or system-quality finding.

---

## 2026-08-26 owner-authority reconciliation (controlling addendum)

ADR-0069 records the explicit owner decisions from Issue #114. This addendum supersedes earlier text in this document only where it conflicts. Earlier `eframe`/`egui`, `llama.cpp`, text-first release, desktop-only, speech/avatar deferral, owner-delegation, or general R2-Continue/readiness language is preserved as historical evidence and is not active selection authority.

Status: Approved, reconciled 2026-08-26 by ADR-0069

### Mission and learner journey

One local learner launches either the Windows desktop client or a browser on the same Windows PC. Both render the identical shared interface. The learner completes Networking Fundamentals / TCP Connection Establishment using governed knowledge, the separately installed graphical LM Studio server, admitted tutor output, assessment/practice, atomic SQLite progress, and restart/resume. Required bundled CPU-capable speech enables conversation; an animated 2D tutor provides synchronized lip-sync and semantic states. System verification, representative user acceptance, and packaging gates close v1.

### Required boundaries

- One shared React/TypeScript/Vite candidate frontend, packaged for Windows by candidate Tauri 2, subject to evidence.
- One versioned loopback HTTP/WebSocket Rust business API for both clients; no second Tauri command API.
- Same-PC loopback access only; no LAN or Internet-remote client.
- One learner with no login, account, multi-user administration, or cloud sync.
- SQLite behind `crates/nexa-storage`, canonical IDs, migrations, atomic evidence/progress, backup/recovery, and restart/resume.
- LM Studio as the sole validated v1 reference model server through a narrow provider-neutral adapter. Nexa bundles no LLM weights or inference runtime.
- Bundled/managed speech runtime and models behind provider-neutral ports; Sherpa-ONNX is only a candidate.
- Animated 2D embodiment through semantic behavior ports; Rive is only a candidate and model output never selects primitives.
- Local Rust runtime plus cloud-ready ports/contracts; no working hosted deployment.

### Explicit deferrals

Labs/tool execution, LAN/remote access, hosted deployment, cloud sync, broad/multiple model-server compatibility, dynamic routing/fallback, and 3D release integration are post-v1. Text remains an accessible equivalent, but speech and 2D avatar are required release capabilities.

### Acceptance boundary

Acceptance must distinguish same-machine browser evidence from hosted-web/remote evidence. Scripted providers, in-memory stores, technical research, or spike demonstrations cannot substitute for production adapter and full E2E evidence. Candidate spikes establish only suitability; failed candidates require authority revision.
