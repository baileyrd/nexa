# Nexa Completion Roadmap

Status: Proposed rebaseline roadmap

## Purpose

This roadmap replaces an open-ended sequence of narrow technical increments with a dependency-driven path from the tactical-pause baseline to the Nexa v1 acceptance statement.

The roadmap is organized around product maturity and vertical integration. It does not discard the existing Phase 1-5 implementation; it reclassifies that work as foundation evidence and uses it where it accelerates the shortest credible route to release.

## Governing rule

No work enters implementation merely because it is the next unresolved ADR-sized topic. Every implementation increment must trace to a v1-required capability, an explicit release risk, or a blocking architecture/specification gap.

## Stage R0 — Rebaseline governance and architecture

Objective: restore the documentation hierarchy so that parent architecture and specifications lead implementation.

Required work:

- complete the current-state/documentation-gap assessment;
- approve the v1 definition and release boundary;
- review `NEXA-ARCH-001` against current intent and implementation;
- promote, revise, or supersede the architecture rather than leaving it Reconstructed;
- review all active Baseline Draft subsystem specifications needed by v1;
- reconcile README, registry, roadmap, status, and traceability terminology;
- establish the capability maturity model and architecture rebaseline gates;
- create a deferral register for inherited Phase 1-5 deferrals;
- classify reserved namespaces as v1-required, conditional, or post-v1.

Exit gate:

Every v1-required capability has an authoritative parent architecture/specification, current maturity state, implementation boundary, acceptance criteria, and known deferrals.

## Stage R1 — Critical cross-cutting specifications

Objective: specify the concerns that have entered the v1 critical path.

Minimum required specifications/decisions:

1. **Data and persistence**
   - authoritative state and replay model;
   - storage technology decision boundary;
   - transactions/concurrency;
   - migration/versioning;
   - retention/deletion;
   - backup/recovery;
   - outbox/event publication if required.

2. **Security**
   - threat model and trust boundaries;
   - credential/secrets handling;
   - local file/network privileges;
   - remote provider trust boundary;
   - lab/tool privilege model if labs enter v1.

3. **Privacy**
   - learner-data classification;
   - local/remote disclosure rules;
   - retention/deletion;
   - diagnostic redaction;
   - model-provider disclosure policy.

4. **Observability**
   - event/log/metric ownership;
   - correlation model;
   - content-safe diagnostics;
   - operational health and failure evidence.

5. **Testing and acceptance**
   - maturity-specific evidence expectations;
   - integration/system/E2E test strategy;
   - production-representative adapter requirements;
   - user acceptance criteria.

6. **UX/application**
   - primary learner journey;
   - core screens/states;
   - interaction/error/recovery behavior;
   - accessibility baseline.

7. **Packaging/deployment**
   - first supported platform;
   - install/configure/update/uninstall;
   - persisted-data migration;
   - release artifact provenance.

Exit gate:

No v1-critical cross-cutting concern remains only a reserved namespace or implicit assumption.

## Stage R2 — Thin production walking skeleton

Objective: create the first real end-to-end Nexa path using concrete dependencies.

Scope intentionally minimal:

- learner-facing application shell;
- one authored course and lesson fixture suitable for production-style use;
- durable learner/lesson persistence;
- existing learning-core policies;
- governed knowledge content and retrieval;
- one concrete model provider;
- structured tutor response admission;
- text output in the application;
- durable progress commit and restart/resume;
- correlation-safe logging.

Do not require speech, avatar, labs, dynamic routing, plugins, or advanced authoring to close this stage.

Exit gate:

A learner can complete one primitive real lesson through one composition root with a real model and durable state, then restart and resume correctly.

## Stage R3 — Complete session orchestration

Objective: turn the walking skeleton into a robust learner session.

Required work:

- compose current lifecycle/cancellation foundations into the actual learner workflow;
- define and implement timeout policy;
- define bounded retry/recovery behavior;
- ensure cancellation/interruption cannot corrupt state;
- integrate retrieval/model/persistence failures into explicit user-visible outcomes;
- establish workflow-level observability;
- verify shutdown and restart behavior.

Existing ADR-0051 through ADR-0067 work should be reused only where it directly supports this session path.

Exit gate:

The primary learner journey survives expected dependency failures, cancellation, shutdown, and restart without state corruption.

## Stage R4 — Grounding and tutor quality gate

Objective: ensure a structurally valid tutor response is also acceptable for the released educational experience.

Required work:

- define semantic grounding/entailment expectations for v1;
- establish citation fidelity checks appropriate to the released corpus;
- define instructional-quality acceptance criteria;
- define prompt-injection/content-boundary handling relevant to governed knowledge;
- create evaluation fixtures and a repeatable model-quality test suite;
- separate deterministic structural tests from probabilistic/evaluation evidence.

Exit gate:

The configured v1 model path meets explicit grounding, citation, and instructional-quality acceptance criteria on the release evaluation set.

## Stage R5 — Required UX and optional embodiment decision

Objective: make the primary learner journey usable and decide which richer interfaces are release-critical.

Required:

- course/lesson selection and resume;
- conversation/input presentation;
- lesson/progress feedback;
- assessment interaction;
- errors/recovery states;
- accessibility checks.

Conditional release candidates:

- speech input;
- TTS/speech output;
- animated avatar integration;
- labs/tool execution.

Decision rule:

A conditional capability enters v1 only if it can reach concrete adapter + runtime integration + system verification + user acceptance without jeopardizing the core release path.

Exit gate:

The primary UX is user-testable, and every conditional capability has an explicit v1/post-v1 disposition.

## Stage R6 — Security, privacy, and data-integrity verification

Objective: verify the cross-cutting specifications against the assembled system.

Required work:

- threat-model review of actual data flows;
- credential/secret storage verification;
- remote disclosure testing;
- retention/deletion tests;
- persistence corruption/failure injection;
- migration tests;
- diagnostic redaction review;
- least-privilege verification;
- supply-chain/dependency review proportional to release.

Exit gate:

No unresolved critical security, privacy, or data-integrity finding remains.

## Stage R7 — Operational and performance hardening

Objective: make Nexa diagnosable and sufficiently performant on the supported target.

Required work:

- explicit startup/interaction/model/retrieval/persistence latency budgets;
- memory/resource budget;
- representative corpus/course benchmarks;
- structured logging/correlation verification;
- failure/recovery diagnostics;
- soak/restart tests where appropriate.

Exit gate:

Measured release candidate meets approved budgets and can be diagnosed without unsafe content exposure.

## Stage R8 — Packaging and release candidate

Objective: produce a distributable release candidate.

Required work:

- supported platform declaration;
- reproducible build;
- installer/package;
- configuration workflow;
- data migration on upgrade;
- version/reporting;
- third-party license and asset provenance;
- release notes and known limitations;
- clean install / upgrade / uninstall test matrix.

Exit gate:

A fresh supported machine can install, configure, run, upgrade, and remove Nexa according to the release specification.

## Stage R9 — System verification and user acceptance

Objective: prove the system rather than its individual contracts.

Required evidence:

- primary learner journey E2E pass;
- restart/resume pass;
- model/storage/retrieval failure paths;
- interruption/cancellation pass;
- security/privacy/data-integrity gates;
- performance gates;
- accessibility gate;
- user acceptance session(s);
- all v1-required capability maturity states at their release thresholds;
- all remaining deferrals explicitly classified post-v1.

Exit gate:

The Nexa v1 acceptance statement is satisfied and a release decision can be made from evidence.

## Work-selection priority after rebaseline

When multiple tasks are available, select in this order:

1. blocks the primary learner journey;
2. blocks durable correctness/state integrity;
3. blocks security/privacy of the primary journey;
4. blocks observability/recovery of the primary journey;
5. blocks packaging/release;
6. improves accepted v1 user experience;
7. optional v1 capability;
8. post-v1 architectural generalization.

This ordering intentionally prevents another drift into technically valid but release-low-leverage horizontal work.

## Definition of roadmap health

At every architecture rebaseline, the Chief Systems Architect must be able to answer:

- What stage are we in?
- What exact exit evidence is missing?
- What is the next blocker to the primary learner journey?
- Which inherited deferrals have reached their review point?
- Can the remaining path to v1 be stated finitely?

If any answer is unclear, implementation pauses until the roadmap is revalidated.
