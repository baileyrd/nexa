# Nexa Development Lessons Learned

Status: Tactical-pause assessment input; non-authoritative

## Purpose

This document records development lessons from Nexa so that project-specific mistakes become reusable process improvements. The purpose is not blame. It is to identify mechanisms that allowed drift and define controls that prevent recurrence.

The evidence chronology behind these lessons is recorded in [`DEVELOPMENT-DIVERGENCE-ANALYSIS.md`](DEVELOPMENT-DIVERGENCE-ANALYSIS.md). That analysis identifies the first material divergence at the Phase 0-to-Phase 1 transition, the PR #20 missed rebaseline opportunity, the qualified Phase 4 contract-gate closure at PR #74, and the repeated horizontal-depth pattern through PR #109.

## What worked

- Contract-first decomposition produced clear ownership boundaries.
- ADRs captured many implementation decisions and explicit deferrals.
- Deterministic tests, exact-head CI review, and focused PR scope produced strong local correctness.
- Traceability documents usually stated what an increment did not prove.
- Provider-neutral and renderer-neutral boundaries preserved replaceability.
- The ChatGPT/Codex review loop improved correction quality and prevented many merge-level defects.

These strengths should be preserved.

## Lesson 1 — Local correctness is not system progress

### What happened

Individual increments were reviewed for bounded scope, applicable ADR/spec consistency, tests, CI, and traceability. Those increments could be correct while the program as a whole became increasingly horizontal and infrastructure-heavy.

### Missed signal

A growing number of green PRs did not produce a correspondingly complete learner journey.

### Corrective rule

Every architecture/program review must separately answer:

- Is this change locally correct?
- Does it materially advance an approved end-to-end system outcome?

Both answers are required for sustained implementation work.

## Lesson 2 — Parent architecture maturity must lead child implementation maturity

### What happened

The system architecture remained Reconstructed and multiple subsystem specifications remained Baseline Draft while accepted ADRs and code became increasingly detailed.

### Why this mattered

ADRs began resolving ambiguity that should sometimes have been closed through parent-specification maturation. Because accepted ADRs have high authority, the governance hierarchy allowed continued implementation even though the higher-level definition remained incomplete.

### Corrective rule

A lower-level decision or implementation gate cannot close a higher-level architecture maturity gap. Child implementation may not advance beyond a defined threshold when its parent architecture/specification is below the required maturity state.

## Lesson 3 — Do not call contract gates architecture completion

### What happened

Several phases were described as complete for a deterministic headless or contract gate. The qualification was technically present, but shorthand such as "Phase complete" created a misleading program-level signal.

### Corrective rule

Use explicit maturity language:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Never collapse these states into a generic "complete."

## Lesson 4 — Build a thin vertical system early

### What happened

Development became horizontally deep in contracts, validation, model selection, tokenization, evidence, cancellation, and propagation before one real learner interaction existed through concrete UI, model, persistence, and output paths.

### Corrective rule

After foundational contracts, prioritize a deliberately thin walking skeleton that traverses the intended system end to end with concrete dependencies. Harden and generalize around that functioning path.

For Nexa, an earlier vertical milestone should have been approximately:

`learner input -> orchestrator -> lesson/student/pedagogy/knowledge -> one real model -> structured response -> one real output path -> persisted progress`

## Lesson 5 — Deferred work requires active governance

### What happened

Deferrals were often documented correctly, but they accumulated across phases.

### Corrective rule

Every deferral must include:

- owner or owning boundary;
- reason;
- earliest required milestone;
- blocking condition;
- review date/gate;
- disposition: retire, implement, supersede, or explicitly re-approve.

A phase gate must inspect inherited deferrals before approving new architectural depth.

## Lesson 6 — Architecture gates must evaluate the whole system

### What happened

Gates focused heavily on conformance of the increment under review.

### Corrective rule

Architecture gates must examine at minimum:

- parent architecture/spec maturity;
- end-to-end capability progress;
- integration maturity;
- concrete adapter maturity;
- accumulated deferrals;
- unresolved cross-cutting concerns;
- documentation consistency;
- technical-debt trajectory;
- current release criteria;
- whether the roadmap still represents the shortest credible path to the product objective.

## Lesson 7 — The Chief Systems Architect must act as program-integrity gatekeeper

### What happened

The Chief Systems Architect role effectively optimized local architectural correctness and PR governance but did not call a tactical pause when the pattern of system-level drift became material.

### Role failure

The role treated accepted local authorities as sufficient evidence for continued progress rather than periodically evaluating whether those authorities themselves were mature and complete enough to govern the program.

### Corrective rule

The Chief Systems Architect has an affirmative responsibility and authority to stop implementation when any of the following occur:

- implementation maturity overtakes parent specification maturity;
- repeated increments deepen one horizontal concern without advancing vertical capability;
- cross-phase deferrals materially accumulate;
- architecture, roadmap, status, and implementation tell different stories;
- no finite release path can be articulated from current state;
- the current work is locally valid but systemically low leverage.

Failure to surface these conditions is itself an architecture-governance defect.

## Lesson 8 — Roadmaps need product outcomes, not only technical gates

### What happened

The roadmap was effective at describing technical phases but insufficiently explicit about a first releasable Nexa product.

### Corrective rule

Every roadmap must trace technical gates to observable user/system outcomes and to a bounded release definition. A roadmap without a finite definition of done can generate valid work indefinitely.

## Lesson 9 — Specifications, ADRs, traceability, and status serve different purposes

### What happened

ADRs increasingly carried behavioral detail because parent specifications were incomplete. Traceability and status documents then summarized those decisions, creating multiple places from which apparent project truth could be inferred.

### Corrective rule

- Architecture defines the system structure and governing intent.
- Specifications define required behavior and acceptance.
- ADRs record consequential decisions and tradeoffs.
- Traceability maps requirements to evidence.
- Status reports current maturity.

Do not allow one artifact type to become a substitute for another.

## Lesson 10 — Documentation consistency is a release-control concern

### What happened

README, roadmap, registry, project status, traceability, and individual ADRs did not always communicate the same current maturity.

### Corrective rule

At every rebaseline gate, run a documentation consistency review. Conflicting phase/maturity statements are blocking defects because they change how future work is selected.

## Process changes to institutionalize

1. Mandatory project inception artifact set before implementation.
2. Explicit document maturity model and parent/child gate rules.
3. Thin vertical walking-skeleton milestone early in development.
4. Periodic architecture rebaseline independent of PR cadence.
5. Cross-phase deferral register with mandatory dispositions.
6. Capability maturity matrix maintained against release criteria.
7. Separate local PR acceptance from program-progress acceptance.
8. Formal authority for the Chief Systems Architect to call a tactical pause.
9. Product-level definition of done before deep subsystem implementation.
10. Final system verification and user acceptance distinct from unit/contract/conformance testing.

## Core principle

The Chief Systems Architect is responsible not only for ensuring that each change is architecturally correct, but also for continuously determining whether the program as a whole is progressing toward the intended system. Local correctness must never substitute for system-level progress.
