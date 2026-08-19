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

**Status:** Complete for the headless deterministic exit gate. Durable production adapters remain a
later infrastructure increment and are not implied by this status.

Implement:

- [x] governed student model, immutable evidence ledger, versioned replay policy, and persistence ports
- [x] pure, versioned, explainable pedagogy policy over read-only mastery projections
- [x] governed curriculum contracts and pure, versioned headless lesson transitions
- [x] dependency-light assessment contracts, deterministic scoring, lifecycle, and evidence creation
- [x] synchronous learning-core composition and explicit atomic unit-of-work port

Exit: **demonstrated** by the `nexa-learning-core` end-to-end conformance and failure-injection tests: a headless deterministic lesson adapts, assesses, appends evidence, replays mastery, routes governed progress, and commits atomically without an LLM or avatar.

See [ADR-0010](../adr/0010-learning-state-evidence-and-persistence.md),
[ADR-0011](../adr/0011-pedagogy-policy-ownership-and-versioning.md),
[ADR-0012](../adr/0012-governed-curriculum-and-lesson-transitions.md),
[ADR-0013](../adr/0013-assessment-contract-scoring-and-evidence.md),
[ADR-0014](../adr/0014-learning-core-composition-and-atomicity.md), and the
[Phase 3 traceability matrix](PHASE-3-TRACEABILITY.md). Durable adapter semantics remain unresolved and are recorded rather than silently selected.

## Phase 4 — Add knowledge and tutor intelligence

**Status:** In progress. The knowledge slices through deterministic citation resolution (ADRs 0015–0020) and provider-neutral structured response planning (ADR-0021) are implemented. Learned reranking, partial truncation, provider tokenization, semantic citation fidelity and safety, generative tutor intelligence, provider integration, networking, vector databases, and durable adapters remain unimplemented.

- Source ingestion and provenance
- [x] deterministic governed lexical retrieval
- [x] governed embedding contracts and deterministic vector retrieval
- [x] exact hybrid fusion and provider-free policy reranking
- context assembly and token budgeting
- [x] provider-neutral structured tutor response planning contracts
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

### Phase 4 narrow increment: tutor response planning

ADR-0021 adds provider-neutral caller-supplied structured responses, deterministic citation/pedagogy/safety validation, and standalone replay evidence. It does **not** complete Phase 4 or tutor intelligence; semantic safety, semantic entailment, generation, providers, networking, persistence, and durable adapters remain deferred.
