# Phase 3 learning-core traceability

Phase 3 is **in progress**. This matrix covers the student-model and narrow pedagogy-policy increments.

| Requirement | Authority | Implementation / evidence | Status |
|---|---|---|---|
| Canonical learner identifiers and bounded mastery | NEXA-DOM-001; ADR-0003 | `nexa-domain` UUID newtypes and `MasteryScore` | Implemented slice |
| Student, objective, competency, attempt, evidence, mastery values | NEXA-STU-001; NEXA-LESSON-001; NEXA-ASMT-001 | `crates/nexa-student/src/lib.rs` | Implemented slice |
| Immutable, idempotent evidence history | NEXA-STU-001 §§5, 85–87; ADR-0010 | repository port, deterministic in-memory ledger, duplicate/conflict tests | Implemented slice |
| Replayable, explicitly versioned estimator | NEXA-STU-001 §§23–28, 83–85; ADR-0010 | `BoundedWeightedV1`, deterministic replay and ordering tests | Implemented slice |
| Persistence boundary without database/runtime | ADR-0001; ADR-0002; ADR-0010 | synchronous repository traits and boundary script | Implemented slice |
| Typed learning facts | NEXA-EVT-001; NEXA-STU-001 §§91–94 | `CompetencyEvidenceAdded` and `CompetencyUpdated` in `nexa-events` | Contract only |
| Wire stability and malformed input | ADR-0004 | golden evidence fixture and serde validation tests | Implemented slice |
| Read-only deterministic pedagogy policy | NEXA-PED-001 §§4–9, 24–33, 73–75, 90, 117–129; ADR-0011 | `PedagogyPolicyV1`, validated contracts, golden and table tests | Implemented slice |
| Stable explanations and unavailable-option safety | NEXA-PED-001 §§7–8, 124–126; ADR-0011 | closed rationale codes and deterministic availability resolution | Implemented slice |
| Privacy-minimal pedagogy fact | NEXA-EVT-001; NEXA-PED-001 §§100–101; ADR-0011 | `PedagogyDecisionMade` in `nexa-events` | Contract only |
| Lesson/curriculum engine | NEXA-LESSON-001 | future `nexa-lessons` increment | Not started |
| Assessment engine | NEXA-ASMT-001 | future `nexa-assessment` increment | Not started |
| Concrete governed persistence | Phase 3 roadmap; ADR-0010 | durable adapter intentionally deferred | Not started |

## Recorded baseline ambiguities

The source material illustrates both UUID identities and semantic competency strings; this slice uses
the governed UUID types and does not invent key semantics. Estimator coefficients, equal-timestamp
ordering, transaction technology, async signatures, retention/regression rules, and event-envelope
construction were also unspecified. ADR-0010 records the narrow choices and deferrals without editing
the reconstructed specifications. ADR-0011 additionally records unresolved action/strategy mapping,
attempt scope, per-competency threshold authorship, policy hashes, and constraint composition.

## Exit-gate position

Phase 3 is not complete: no headless adaptive lesson spanning pedagogy, curriculum, and assessment
exists yet. The next recommended increment is a headless curriculum/lesson contract that consumes
pedagogy routing while keeping content execution, assessment, LLM, persistence, and orchestration out.
