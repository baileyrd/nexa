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

## Owner-approved concrete v1 path

The concrete release path is:

- one shared React/TypeScript/Vite frontend candidate used identically by the same-machine browser and a Tauri 2 Windows desktop shell candidate;
- one local Rust runtime exposing one versioned loopback HTTP/WebSocket business API; Tauri commands do not form a second business API;
- SQLite through `nexa-storage` as authoritative state;
- the separately installed graphical LM Studio server as the sole v1 reference model server, with no bundled LLM weights or inference runtime;
- bundled CPU-capable speech and synchronized animated 2D embodiment as release requirements; Sherpa-ONNX and Rive remain evidence-gated candidates;
- Networking Fundamentals / TCP Connection Establishment as the governed first content package.

## Explicitly deferred from v1

LAN/remote access, hosted deployment, cloud sync, accounts and multi-user administration, labs/tools, broad model-server support, dynamic routing/fallback, dedicated vector infrastructure unless evidence proves it necessary, durable event brokerage, and 3D release integration do not block v1. Text remains an accessibility and recovery path, but does not replace required speech or animated 2D behavior.

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
