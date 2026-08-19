# Phase 3 learning-core traceability

Phase 3 is **in progress**. This matrix covers the student-model, pedagogy-policy,
curriculum/lesson-transition, and narrow deterministic assessment increments.

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
| Immutable authored curriculum hierarchy and mappings | NEXA-LESSON-001 §§2–11, 28–31; ADR-0012 | validated curriculum/course/module/lesson/step contracts | Implemented slice |
| Deterministic prerequisite graph | NEXA-LESSON-001 §§26, 28; ADR-0012 | dangling/self/cycle rejection and stable topological order | Implemented slice |
| Separate validated progress and transitions | NEXA-LESSON-001 §§33–36; ADR-0012 | serde-validated `LessonProgress` and pure `LessonPolicyV1` lifecycle table | Implemented slice |
| Curriculum-constrained pedagogy routing | NEXA-LESSON-001 §§27–30; NEXA-PED-001; ADR-0012 | read-only decision, authored route lookup, structured rejection | Implemented slice |
| Privacy-minimal lesson facts | NEXA-EVT-001; ADR-0012 | lifecycle and transition payloads in `nexa-events` | Contract only |
| Validated authored assessment/question/rubric contracts | NEXA-ASMT-001 §§5, 14–39, 104–105, 111, 186; ADR-0013 | `crates/nexa-assessment/src/lib.rs`, golden and malformed-wire tests | Implemented slice |
| Versioned deterministic scoring and aggregation | NEXA-ASMT-001 §§25–39, 78–82, 95, 174–176; ADR-0013 | `ScoringPolicyV1`, boundary and rubric aggregation tests | Implemented slice |
| Frozen attempt lifecycle and replay safety | NEXA-ASMT-001 §§55–67, 105, 136–143, 186; ADR-0013 | pure transitions, scope/policy/time/conflict and immutability tests | Implemented slice |
| Privacy-minimal mastery evidence creation | NEXA-ASMT-001 §§82–88, 135–137; ADR-0010; ADR-0013 | `LearningEvidence` output and existing ledger compatibility tests | Implemented slice |
| Typed assessment facts | NEXA-EVT-001; NEXA-ASMT-001 §§131–135; ADR-0013 | evaluated/completed payloads in `nexa-events` | Contract only |
| Concrete governed persistence | Phase 3 roadmap; ADR-0010 | durable adapter intentionally deferred | Not started |

## Recorded baseline ambiguities

The source material illustrates both UUID identities and semantic competency strings; this slice uses
the governed UUID types and does not invent key semantics. Estimator coefficients, equal-timestamp
ordering, transaction technology, async signatures, retention/regression rules, and event-envelope
construction were also unspecified. ADR-0010 records the narrow choices and deferrals without editing
the reconstructed specifications. ADR-0011 additionally records unresolved action/strategy mapping,
attempt scope, per-competency threshold authorship, policy hashes, and constraint composition. ADR-0012
records unresolved rich branch conditions, completion evidence, cross-course and competency
prerequisites, freeform routing, content/version migration, invalidation, and blocked recovery. ADR-0013
records unresolved assessment weighting, timing, selection, evaluator, security, review, persistence,
and orchestration semantics.

## Exit-gate position

Phase 3 is not complete: its individual learning-core policies exist, but no headless composition
atomically connects lesson routing, assessment evidence ingestion, replayed mastery, and governed
durable persistence. The next increment should define that composition boundary without adding an LLM,
avatar, UI, or ungoverned database semantics.
