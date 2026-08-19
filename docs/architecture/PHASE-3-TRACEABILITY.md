# Phase 3 learning-core traceability

Phase 3 is **in progress**. This matrix covers only the first independently reviewable increment.

| Requirement | Authority | Implementation / evidence | Status |
|---|---|---|---|
| Canonical learner identifiers and bounded mastery | NEXA-DOM-001; ADR-0003 | `nexa-domain` UUID newtypes and `MasteryScore` | Implemented slice |
| Student, objective, competency, attempt, evidence, mastery values | NEXA-STU-001; NEXA-LESSON-001; NEXA-ASMT-001 | `crates/nexa-student/src/lib.rs` | Implemented slice |
| Immutable, idempotent evidence history | NEXA-STU-001 §§5, 85–87; ADR-0010 | repository port, deterministic in-memory ledger, duplicate/conflict tests | Implemented slice |
| Replayable, explicitly versioned estimator | NEXA-STU-001 §§23–28, 83–85; ADR-0010 | `BoundedWeightedV1`, deterministic replay and ordering tests | Implemented slice |
| Persistence boundary without database/runtime | ADR-0001; ADR-0002; ADR-0010 | synchronous repository traits and boundary script | Implemented slice |
| Typed learning facts | NEXA-EVT-001; NEXA-STU-001 §§91–94 | `CompetencyEvidenceAdded` and `CompetencyUpdated` in `nexa-events` | Contract only |
| Wire stability and malformed input | ADR-0004 | golden evidence fixture and serde validation tests | Implemented slice |
| Pedagogy policy engine | NEXA-PED-001 | future `nexa-pedagogy` increment | Not started |
| Lesson/curriculum engine | NEXA-LESSON-001 | future `nexa-lessons` increment | Not started |
| Assessment engine | NEXA-ASMT-001 | future `nexa-assessment` increment | Not started |
| Concrete governed persistence | Phase 3 roadmap; ADR-0010 | durable adapter intentionally deferred | Not started |

## Recorded baseline ambiguities

The source material illustrates both UUID identities and semantic competency strings; this slice uses
the governed UUID types and does not invent key semantics. Estimator coefficients, equal-timestamp
ordering, transaction technology, async signatures, retention/regression rules, and event-envelope
construction were also unspecified. ADR-0010 records the narrow choices and deferrals without editing
the reconstructed specifications.

## Exit-gate position

Phase 3 is not complete: no headless adaptive lesson spanning pedagogy, curriculum, and assessment
exists yet. The next recommended increment is a deterministic pedagogy decision policy consuming
read-only mastery projections and producing explainable strategy/routing decisions, with no content,
LLM, persistence, or orchestration implementation.
