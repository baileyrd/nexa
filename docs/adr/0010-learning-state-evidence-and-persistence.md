# ADR-0010: Learning-state ownership, immutable evidence, and versioned mastery

- **Status:** Accepted
- **Date:** 2026-08-19
- **Related:** NEXA-DOM-001, NEXA-EVT-001, NEXA-STU-001, NEXA-PED-001, NEXA-LESSON-001, NEXA-ASMT-001

## Context

The reconstructed specifications agree that assessment and lesson systems produce observations while
the Student Model interprets them over time. They do not settle a concrete first estimator, durable
transaction technology, async runtime, same-timestamp ordering rule, or whether illustrative semantic
keys replace the UUID identities governed by NEXA-DOM-001. Those ambiguities must not become hidden
infrastructure choices.

## Decision

`nexa-student` owns validated student learning records, the append-only evidence ledger contract, and
derived mastery projections. Entity identity and normalized values remain in `nexa-domain`.
Assessment, lesson, lab, and tutor components may submit evidence but may not directly mutate mastery.

Evidence is immutable after acceptance. Repeating an identical `EvidenceId` is an idempotent no-op;
reusing it for different content is an error. Replay order is `(observed_at, evidence_id)`, making the
otherwise unspecified equal-time order explicit and deterministic. Evidence remains canonical even
when projections are cached or replaced.

The first estimator is the pure `BoundedWeightedV1` policy identified by `ProtocolVersion(1, 0)`.
Its coefficients and status thresholds are part of that policy version. Reprocessing a history must
select its recorded policy version; changing coefficients requires a new version rather than silently
rewriting prior meaning.

Only synchronous repository ports and deterministic in-memory test adapters are introduced. Durable
storage, transactions/outbox behavior, retention, erasure, encryption, authorization, snapshots, and
async execution are deferred. `nexa-events` owns privacy-minimal typed evidence-added and
competency-updated payloads so `nexa-student` can emit those contracts without a dependency cycle.
Event envelope identities, timestamps, causation, and publication remain caller/composition concerns.

## Consequences

- Mastery can be discarded and reconstructed from preserved evidence.
- Duplicate delivery cannot double-apply mastery, and conflicting duplicates are visible.
- Persistence adapters cannot dictate domain APIs or introduce a runtime into the learning core.
- The initial estimator is intentionally modest and is not a claim of psychometric validity.

## Unresolved decisions

- Durable atomic evidence/projection/outbox commits and optimistic concurrency semantics.
- Privacy retention, learner export/erasure, encryption, access control, and audit policy.
- Snapshot schema, migration, policy registry, and historical policy retirement rules.
- Whether authored human-readable competency keys become an additional canonical domain type.
- Regression, retention decay, prerequisite aggregation, batch ingestion, and multi-competency attribution.
- Which composition service supplies learning-event envelope identity and causation.
