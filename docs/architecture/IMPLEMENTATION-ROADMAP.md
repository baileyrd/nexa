# Nexa Implementation Roadmap

This roadmap converts the reconstructed design baseline into verifiable increments. Phase boundaries are architectural gates, not calendar promises.

## Phase 0 — Govern the baseline

- Maintain the canonical specification registry.
- Audit dependency declarations, identifiers, duplicates, and reconstruction formatting.
- Define ownership, status transitions, and conformance evidence.
- Repair repository navigation and CI documentation.
- Preserve the working 3D runtime while the workspace is established.

Exit: every active specification has an ID, status, authority, dependencies, and intended implementation boundary.

## Phase 1 — Establish the contract kernel

Implement and test:

- `nexa-domain`
- `nexa-events`
- `nexa-nbp`
- shared error, identifier, time, version, and confidence types

Exit: downstream subsystems consume one canonical set of serialized contracts with schema compatibility tests.

## Phase 2 — Migrate embodiment

**Status:** Complete. The ownership migration and governed embodiment acceptance flow are verified.

- [x] Split the former root package into `crates/nexa-3d`, `apps/nexa-3d-viewer`, and `tools/nexa-3d-validate`.
- [x] Add `nexa-avatar` as the renderer-neutral embodiment port.
- [x] Preserve headless validation, animation, skinning, gaze, and viseme behavior.
- [x] Validate runtime assets and manifests through headless CI gates.
- [x] Complete NBP capability negotiation, governed outputs, typed lifecycle events, and deterministic headless conformance.

The ownership move is recorded in [ADR-0008](../adr/0008-controlled-3d-workspace-migration.md); acceptance and lifecycle semantics are recorded in [ADR-0009](../adr/0009-embodiment-acceptance-and-lifecycle.md).

Exit: the 3D implementation consumes NBP/avatar contracts and all existing tests pass from the workspace root.

## Phase 3 — Build the learning core

**Status:** In progress. The governed student-model/evidence-ledger slice is implemented; the Phase 3 exit gate is not satisfied.

Implement:

- [x] governed student model, immutable evidence ledger, versioned replay policy, and persistence ports
- pedagogy policy engine
- curriculum/lesson engine
- assessment engine
- governed persistence ports

Exit: a headless deterministic lesson can adapt, assess, and update student state without an LLM or avatar.

See [ADR-0010](../adr/0010-learning-state-evidence-and-persistence.md) and the
[Phase 3 traceability matrix](PHASE-3-TRACEABILITY.md). The next recommended increment is the
deterministic, explainable pedagogy decision policy over read-only mastery projections.

## Phase 4 — Add knowledge and tutor intelligence

- Source ingestion and provenance
- hybrid retrieval and reranking
- context assembly and token budgeting
- structured tutor response contracts
- model-provider abstraction and safety gates

Exit: grounded responses carry citations, confidence, and machine-validated tutor/behavior output.

## Phase 5 — Orchestrate a complete session

- cancellation-safe session workflow
- speech input/output ports
- behavior synchronization
- tool/lab execution
- interruption, retry, timeout, and recovery policies
- event-driven observability

Exit: a user completes an end-to-end lesson through one composition root.

## Phase 6 — Authoring, packaging, and operations

- course, assessment, lab, and asset compilers
- authoring application
- plugin SDK and capability permissions
- local-first packaging and update strategy
- analytics, privacy, security, and performance gates

Exit: a signed release can be authored, validated, installed, upgraded, observed, and recovered.

## Cross-phase quality gates

Every increment includes:

- unit, contract, integration, and conformance tests
- threat/privacy review proportional to scope
- accessibility checks for user-facing work
- deterministic fixtures where practical
- schema and content versioning
- documentation and traceability updates
