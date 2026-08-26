# Nexa Architecture Rebaseline and Program-Integrity Gates

Status: Tactical-pause proposed governance

## Purpose

These gates prevent local implementation correctness from being mistaken for system-level architectural progress. They apply to Nexa immediately and are intended to seed reusable engineering standards.

## Gate model

A gate is blocking. A gate does not pass because code compiles, tests are green, or a local ADR is internally consistent. Evidence must satisfy the whole gate.

## Gate G0 — Project inception

Required before sustained implementation:

- product/system mission and boundaries;
- stakeholder/user outcomes;
- initial system context and architecture;
- authoritative document hierarchy;
- definition of first releasable product;
- initial capability decomposition;
- cross-cutting concerns identified;
- verification and validation strategy;
- decision/ADR process;
- known assumptions and uncertainties.

Exit: there is a finite, testable statement of what the first release is intended to accomplish.

## Gate G1 — Architecture baseline

Required before subsystem implementation expands beyond exploratory spikes:

- system architecture reviewed and at required maturity;
- subsystem ownership and interfaces identified;
- parent-child specification relationships explicit;
- critical quality attributes and constraints identified;
- primary data/state ownership defined;
- security/privacy boundaries identified;
- integration strategy identified;
- thin vertical walking-skeleton path defined.

Exit: lower-level implementation can be judged against a mature parent architecture.

## Gate G2 — Specification readiness

Required before a subsystem can be called implementation-ready:

- governing specification approved for the intended slice;
- observable behavior and errors defined;
- dependencies and ownership defined;
- compatibility/versioning expectations defined;
- acceptance and conformance evidence defined;
- unresolved questions either block implementation or are explicitly bounded as non-blocking;
- ADRs record decisions rather than substitute for missing specification content.

Exit: implementation is downstream of specification rather than creating the specification accidentally.

## Gate G3 — Walking skeleton

Required before deep horizontal hardening:

- one thin end-to-end product path exists;
- concrete dependencies are used at every essential boundary where feasible;
- the path crosses input, orchestration, domain logic, persistence/state, output, and error handling;
- system-level integration tests exist;
- the path is usable enough to validate architecture assumptions.

Exit: the architecture has been exercised as a system, not only as isolated contracts.

## Gate G4 — Periodic architecture rebaseline

Trigger at minimum:

- phase boundary;
- major subsystem activation;
- material architecture change;
- accumulated deferral threshold;
- repeated horizontal increments without vertical progress;
- conflict between roadmap/status/specification/implementation;
- inability to state a finite path to the release definition.

Review questions:

1. Is the current architecture still authoritative and sufficient?
2. Is implementation maturity ahead of parent specification maturity anywhere?
3. What end-to-end user/system capability advanced since the prior review?
4. Which deferrals crossed a boundary and why?
5. Which concrete adapters remain absent?
6. Are cross-cutting concerns now on the critical path?
7. Does the roadmap still represent the shortest credible path to release?
8. Are status terms accurately representing maturity?
9. Are we building capabilities required by the release definition?
10. Should implementation continue, redirect, or pause?

Exit: explicit Continue, Redirect, or Tactical Pause decision with evidence.

## Gate G5 — Vertical capability acceptance

A capability is not complete until the required maturity for its release role is satisfied.

Possible maturity states:

1. Concept identified
2. Architecture defined
3. Specification approved
4. Contract implemented
5. Runtime integrated
6. Concrete adapter implemented
7. System verified
8. User accepted
9. Release ready

Project status MUST report the actual state, not a collapsed generic `Complete` label.

## Gate G6 — Release candidate

Required before a release candidate:

- every v1-required capability meets its required maturity;
- durable data/state behavior is verified;
- security and privacy requirements are verified;
- observability and recovery are verified;
- installation/package/update path is verified;
- supported platform matrix is verified;
- system-level performance budgets are measured;
- accessibility requirements are verified for user-facing surfaces;
- known limitations and deferred post-v1 work are explicit;
- release acceptance tests pass in a production-representative environment.

## Deferral gate

Every architectural deferral must record:

- owning boundary;
- rationale;
- introduced-at milestone/decision;
- earliest required milestone;
- consequence of continued deferral;
- next mandatory review gate;
- disposition.

A deferral that reaches its mandatory review gate may not silently roll forward.

## Chief Systems Architect stop conditions

The Chief Systems Architect must call for review or tactical pause when:

- parent documentation trails child implementation materially;
- locally correct work repeatedly fails to advance vertical capability;
- critical deferrals accumulate across phase boundaries;
- product acceptance remains undefined;
- cross-cutting concerns are being deferred after entering the critical path;
- implementation and governing documentation present materially different system states;
- the roadmap can no longer express a finite, credible route to release.

## Relationship to PR review

PR acceptance asks whether a change is correct and bounded.

Architecture/program acceptance asks whether continuing that class of change is still the right program action.

Neither substitutes for the other.
