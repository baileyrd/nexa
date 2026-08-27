# Nexa Learning Subsystems v1 Rebaseline Requirements — Draft

Status: R1 proposal; intended to mature `NEXA-STU-001`, `NEXA-PED-001`, `NEXA-LESSON-001`, and `NEXA-ASMT-001`

## 1. Purpose

Reconcile the strong deterministic Phase 3 learning-core slices with the requirements of a persistent learner-facing v1 product. The final governance action should mature the existing parent specifications rather than create competing subsystem authorities.

## 2. Existing foundation retained

Phase 3 already proves:

- canonical learner/competency/attempt/evidence values;
- append-only/idempotent learning evidence;
- replayable versioned mastery estimation;
- pure explainable pedagogy policy;
- validated authored curriculum/lesson graphs;
- deterministic lesson transitions;
- validated assessment/scoring/attempt lifecycle;
- atomic learning-core composition with failure injection through an abstract UoW.

Those are valuable foundations and should not be redesigned without evidence.

## 3. v1 learning outcome

A learner can start/resume the released lesson, complete governed instructional/assessment actions, have accepted evidence committed durably exactly once, derive/update mastery according to the approved policy, receive an approved pedagogy/lesson route, restart the application, and continue from the same authoritative state.

## 4. Student model v1 additions

`NEXA-STU-001` must define/reconcile:

- which learner/profile fields are actually required by v1;
- durable evidence repository semantics through the approved data spec;
- projection persistence/replay/caching rules;
- concurrency/conflict behavior for the supported v1 process/session model;
- deletion/retention behavior delegated to privacy/data policy;
- policy-version migration/replay behavior;
- whether and how prior course/lesson state affects tutor context;
- explicit separation between persistent learner model and transient conversation context.

Do not persist free-form tutor conversation as learner memory by implication.

## 5. Evidence authority

Accepted learning evidence remains the authoritative basis for mastery changes.

v1 requirements:

- evidence identity is stable across retries/restart;
- duplicate-identical submission remains idempotent;
- conflicting identity reuse fails closed;
- evidence commit is atomic with any progress/projection state required by the learning operation;
- evidence provenance identifies the assessment/practice/policy context required for replay;
- learner-facing progress is not reported as saved until the durable operation commits.

## 6. Mastery projection

The approved estimator remains versioned and deterministic for its governed inputs.

v1 must specify:

- when projections are persisted versus recomputed;
- replay horizon/version association;
- behavior when the estimator/policy version changes;
- whether historical evidence is re-evaluated under new policies or frozen to prior policy semantics;
- how migration/rebuild failure is surfaced;
- how mastery confidence/precision is presented, if at all, in UX.

The UX must not imply more statistical certainty than the estimator semantics justify.

## 7. Pedagogy v1 additions

`NEXA-PED-001` must define the bounded v1 instructional decisions used by the first released course.

Requirements:

- every route/action used by the released lesson has an authored/approved semantic mapping;
- thresholds/policy version and rationale are explicit;
- pedagogy consumes read-only learner/mastery context;
- policy cannot select a route unavailable in the current authored lesson graph;
- fallback/recovery behavior when no authored route is valid is defined;
- tutor output generation does not override the pedagogy decision authority;
- v1 evaluation checks whether tutor responses adhere to the intended pedagogy mode where applicable.

Advanced/generalized pedagogy strategies may remain post-v1.

## 8. Curriculum and lesson v1 additions

`NEXA-LESSON-001` must define the exact authored features supported by the first released course.

Required v1 scope should be intentionally small but complete:

- curriculum/course/module/lesson hierarchy actually used;
- lesson steps/activities used by the released content;
- prerequisite behavior actually required;
- authored routing from pedagogy outcomes;
- assessment/practice attachments;
- content/version identity;
- progress/resume semantics;
- completion criteria;
- behavior when authored content is updated after learner progress exists.

Do not implement every reconstructed branch/content feature solely to claim specification completeness.

## 9. Course/content versioning

Learner progress must be bound to the authored content version/fingerprint that produced it.

The lesson/content specification must define:

- compatible non-semantic edits, if any;
- semantic content changes requiring migration/restart/new version;
- behavior when the current application opens progress bound to an unsupported content version;
- how released content packages declare compatibility;
- whether completed historical lessons remain viewable after content update.

## 10. Assessment v1 scope

`NEXA-ASMT-001` must define the assessment/question types actually shipped in the first course.

v1 requirements:

- authored assessment/version validation;
- scoring policy/version;
- exact attempt lifecycle;
- one accepted outcome/evidence path;
- duplicate/conflict behavior;
- assessment-protected content handling;
- learner-facing feedback allowed by pedagogy/assessment policy;
- durable attempt/evidence semantics;
- restart behavior for supported in-progress attempts;
- explicit disposition of unsupported advanced weighting/randomization/manual review/evaluator types.

## 11. Assessment security/privacy

The assessment parent must integrate with security/privacy:

- protected answer/rubric material must not enter learner-facing retrieval/tutor context without explicit governed need;
- raw learner answers are retained only as required for replay/review/product scope;
- normal logs exclude learner answers;
- remote model use for evaluation, if any, requires a separate explicit governed flow and disclosure policy;
- v1 deterministic scoring should remain local where the released assessment type permits it.

## 12. Learning-core v1 composition

The existing `nexa-learning-core` remains the atomic policy-composition boundary.

The concrete v1 operation must integrate it with the durable UoW while preserving:

- all preflight/validation before commit;
- exact expected-snapshot/version conflict detection;
- no partial evidence/progress/mastery update;
- idempotent operation receipt/retry semantics;
- safe failure injection/restart behavior;
- no persistence-policy leakage into domain policy crates.

## 13. Orchestrator relationship

The orchestrator:

- loads exact persisted learning state;
- invokes learning policies through their owning boundaries;
- coordinates tutor/retrieval/model interaction;
- invokes the learning-core commit after a learner action requires state change;
- maps bounded outcomes to UX.

The orchestrator does not calculate mastery, score assessments, invent lesson transitions, or rewrite pedagogy decisions.

## 14. Tutor relationship

Tutor context may consume bounded/reference learning information required for instruction, such as:

- current lesson objective/activity;
- mastery/competency summary relevant to the lesson;
- pedagogy decision/strategy;
- recent governed mistake/evidence summary only if explicitly part of the approved tutor context model.

Tutor generation cannot directly mutate learner state. Only governed learner actions/evidence accepted through the learning path can change mastery/progress.

## 15. Authoring boundary

A full authoring application is not required for v1.

The first course may be hand-authored as governed versioned content, provided:

- schema/content validation exists;
- content is reproducible/versioned;
- invalid content cannot enter the release package;
- the release process can validate the course/assessment/knowledge package.

General course/assessment compilers or authoring UI may be post-v1 unless required to make the release content safely maintainable.

## 16. Verification

Before System Verified maturity, prove using concrete durable storage and the actual learner application path:

- new learner lesson start;
- existing learner exact resume;
- assessment start/submit/score;
- atomic evidence/mastery/progress commit;
- duplicate-identical retry;
- conflicting retry/concurrency failure;
- pedagogy route constrained to authored availability;
- completion criteria;
- restart after each durable boundary;
- migration/content-version incompatibility behavior;
- protected assessment data exclusion from tutor/retrieval path;
- failure injection with no partial learner state;
- primary v1 lesson E2E through tutor and UI.

## 17. v1 deliberate simplifications

Unless the first released course needs them, defer:

- rich arbitrary branch expressions;
- cross-course adaptive prerequisite graphs;
- generalized free-form pedagogy action vocabulary;
- complex weighted/randomized assessments;
- manual/instructor review workflows;
- remote AI assessment grading;
- enterprise learner management;
- analytics dashboards;
- general authoring application.

## 18. Approval decisions

- exact first-course lesson/activity feature subset;
- exact assessment/question type subset;
- supported learner/profile fields;
- projection persistence/recompute policy;
- content-version migration behavior;
- learner evidence deletion/retention policy integration;
- completion criteria for the first released course;
- whether any authoring compiler is required for release maintainability.

## 2026-08-26 ADR-0069 reconciliation

One local learner completes the same governed learning workflow from either identical client. Accepted evidence/progress remains atomic and restartable in SQLite. Speech and avatar presentation cannot independently alter mastery; labs/tools and cloud sync remain deferred.
