# ADR-0014: Learning-core composition and atomicity boundary

- **Status:** Accepted
- **Date:** 2026-08-19
- **Related:** ADR-0010, ADR-0011, ADR-0012, ADR-0013; NEXA-STU-001; NEXA-PED-001; NEXA-LESSON-001; NEXA-ASMT-001

## Context

The Phase 3 policies were independently deterministic but no boundary proved that assessment state,
immutable evidence, replayed mastery, pedagogy, and lesson progress could be coordinated without
partial persistence. This is learning-operation composition, not the Phase 5 session orchestrator:
it does not own interaction loops, cancellation, providers, tools, scheduling, or event publication.

## Decision

`nexa-learning-core` is the narrow synchronous composition boundary. It depends inward on the
assessment, student, pedagogy, lesson, event, and domain contracts and delegates every domain rule to
their existing versioned policies. A request targets exactly one canonical student, lesson,
assessment attempt, and response, supplies the complete competency/evidence mapping, and explicitly
selects one of those competencies as the pedagogy and authored-routing scope. The service
starts or loads governed state, invokes `ScoringPolicyV1`, appends its immutable evidence with the
student ledger's duplicate/conflict semantics, replays `BoundedWeightedV1`, constructs validated
read-only pedagogy input, invokes `PedagogyPolicyV1`, and asks `LessonPolicyV1` to apply only an
authored route. It never assigns mastery directly.

`LearningUnitOfWork` loads one validated snapshot and commits a complete replacement against that
expected snapshot. Commit must be atomic and stale snapshots must not be merged. An operation receipt
is part of the atomic state: identical operation-ID replay returns its prior result without writing;
different content under that ID is rejected. The deterministic in-memory adapter publishes its
replacement only after all injected commit stages succeed. This defines required semantics without
choosing locks, isolation levels, a database, or an async runtime.

Response identity is independently idempotent across operation IDs. A response ID already committed
with an otherwise identical operation and identical authored assessment and curriculum returns its
prior result as a replay before lesson lifecycle or scoring work; any mismatch is rejected. Receipt
v1 retains those exact authored contracts as its auditable semantic fingerprint rather than relying
on a lossy or implementation-dependent hash. For a multi-competency response,
all evidence and affected mastery projections are updated in deterministic competency order while
only the explicitly selected, validated competency drives pedagogy.

Pedagogy history rule v1 is derived from canonical evidence ordered by `(observed_at, evidence_id)`
and scoped to the selected student and competency. `attempt_count` is that stream's evidence count,
`recent_outcome` is its final evidence outcome (`ambiguous` maps to partial success), and
`consecutive_failures` is only the trailing run of failures. Evidence from another competency never
contributes to these fields. Event-fact semantic keys use exhaustive mappings to the governed
snake-case wire vocabulary rather than debug formatting.

Loaded persistence is untrusted input. Before lookup or replay, v1 rejects duplicate or noncanonical
lesson scopes, attempt IDs, evidence IDs/order, and mastery scopes, as well as receipt map-key,
authored-assessment, request, and result scope inconsistencies. Successful replacements sort every
state vector by its governed identity/order so a durable adapter cannot make first-match behavior
ambiguous.

The result contains policy outputs and privacy-minimal typed facts only. It deliberately does not
construct event envelopes or own event IDs, causation, correlation, sequencing, publication, or a
durable outbox. A durable adapter may atomically stage an outbox with the state, but that design is not
implied by the current port.

## Consequences

- The Phase 3 headless exit path is demonstrated with deterministic conformance and failure-injection
  tests, without an LLM, avatar, UI, database, networking, or async runtime.
- Serialization validates stored values and new request/result/snapshot contracts; canonical ordered
  collections make replay and output stable.
- A single assessment response may update every competency mapped to its question; the boundary is
  not a general arbitrary batch or complete lesson/session workflow.

## Deferred durable-adapter decisions

- transaction technology, isolation level, optimistic tokens, lock scope, retry/backoff, and crash recovery;
- transactional outbox schema, ordering, dispatch, deduplication, and retention;
- evidence/state retention, erasure, encryption, backup, migration, and policy-version retirement;
- authentication, authorization, tenant isolation, auditing, and administrator repair;
- snapshot/cache validation, durable receipt lifetime, and conflict resolution;
- whether durable adapters expose finer repositories behind this unit of work.
