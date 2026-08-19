# ADR-0013: Assessment ownership, deterministic scoring, and evidence creation

- **Status:** Accepted
- **Date:** 2026-08-19
- **Governing specifications:** NEXA-ASMT-001, NEXA-STU-001, NEXA-DOM-001, NEXA-EVT-001

## Context

NEXA-ASMT-001 reconstructs a broad target spanning deterministic and provider-backed evaluators,
protected packages, timers, pools, adaptive selection, review, regrading, authoring, analytics, and
persistence. It does not fix a first scoring formula, rubric normalization rule, identifier format,
wire ordering rule, evidence identifier derivation, or lifecycle transition table. The learning core
needs a small reproducible assessment seam without silently selecting infrastructure or psychometric
semantics.

## Decision

`nexa-assessment` owns immutable validated authored assessments, questions, rubrics, frozen item
instances, mutable attempt values, pure lifecycle transitions, and performance outcomes. Shared UUID
identities live in `nexa-domain`; `QuestionId`, `ResponseId`, `AssessmentItemInstanceId`, `RubricId`,
and `RubricCriterionId` are added because they cross authored, attempt, event, and persistence seams.
Authored collections are canonicalized by UUID. Meaningful ordering answer keys retain authored order
while rejecting duplicate members.

`ScoringPolicyV1` is protocol version `1.0`. Exact, choice, boolean, and ordering evaluators award zero
or one. Rubric scores are the weighted sum of caller-supplied normalized criterion levels; positive
criterion weights must total one. Attempt score is the equal-weight arithmetic mean of frozen item
scores, and the authored inclusive threshold determines pass/fail. Any formula, normalization,
rounding, weighting, threshold comparison, or rationale meaning change requires another policy
version. Stable rationale codes are `assessment.correct`, `assessment.partial`, and
`assessment.incorrect`.

Attempts freeze question identities and versions separately from authored content. The pure engine
accepts caller-supplied identity and time. Operations validate student, assessment, version, policy,
state, timestamp, and complete evidence mappings before returning a new value. Terminal attempts do
not reactivate. An identical response-ID replay is a no-op and emits no evidence; conflicting reuse is
rejected. Failed operations cannot mutate their input.

Each accepted item evaluation returns one immutable `nexa-student::LearningEvidence` per mapped
competency. The caller supplies unique evidence identities. Evidence carries no response, prompt,
answer key, or rubric and is not written by the assessment engine. Existing student-ledger duplicate
and conflict semantics remain authoritative. `nexa-events` owns minimal evaluated and completed fact
payloads so event contracts do not depend on assessment. Envelope construction and publication remain
composition concerns.

V1 does not infer measurement conditions it did not observe: generated evidence uses `Unknown`
difficulty and independence and no confidence value. The student policy treats unknown difficulty as
neutral and unknown independence conservatively. For rubric questions, each competency's evidence
outcome is derived only from criteria mapped to that competency rather than from the aggregate item
score.

No repository port is added: the pure value-in/value-out slice does not require one. Durable atomic
answer/result/evidence/outbox commits remain an adapter and orchestration concern.

## Reconstructed ambiguities and deferred decisions

- Semantic assessment keys versus canonical UUID identities; semantic aliases are not invented.
- Question weighting, section weighting, per-competency scoring, critical items, penalties, hint
  multipliers, confidence aggregation, rounding, and measurement strength.
- Pause authorization, authoritative elapsed-time calculation, timeouts, retries, navigation, answer
  changes, random selection and seeds, adaptive testing, and maximum attempts.
- Numeric/unit, semantic/AI, code, command, lab, composite, human-review, uncertain, and invalid
  evaluation behavior.
- Protected packages, feedback/reveal policy, accommodations, security, regrading, faulty questions,
  invalidation reasons, analytics, authoring, and compiler behavior.
- Durable repositories, concurrency tokens, transactions/outbox, async APIs, retention, authorization,
  and event-envelope construction.

## Consequences

The increment is deterministic, synchronous, dependency-light, and headless. It establishes a
student-model-compatible evidence boundary without equating assessment score with mastery or directly
mutating learner state. It is deliberately only an implemented Phase 3 slice, not complete conformance
to the full reconstructed assessment architecture or the Phase 3 exit gate.
