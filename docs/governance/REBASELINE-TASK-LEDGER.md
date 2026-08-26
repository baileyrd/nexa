# Nexa Tactical-Pause Rebaseline Task Ledger

Status: Active

Baseline: `e2345a1bb8825451ea079ff5e350b7765075038a` (PR #109)

## Purpose

This ledger is the execution control for the tactical pause. It prevents the assessment itself from becoming an open-ended documentation exercise and records the evidence required before normal implementation resumes.

## Workstream A — Current-state assessment

| Task | Status | Evidence / next gate |
|---|---|---|
| Establish authoritative baseline | Complete | PR #109 merge baseline recorded |
| Inventory governing documents and maturity | Complete initial pass | `NEXA-CURRENT-STATE-ASSESSMENT.md`, `NEXA-REBASELINE-MATRIX.md` |
| Distinguish contract, integration, concrete adapter and user capability | Complete initial pass | capability maturity matrix in assessment |
| Audit historical divergence | Complete initial pass | `DEVELOPMENT-DIVERGENCE-ANALYSIS.md` |
| Centralize inherited deferrals | Complete initial pass | `DEFERRAL-REGISTER.md` |
| Verify documentation consistency gaps | In progress | README/roadmap/status/registry/traceability reconciliation required after architecture approval |

## Workstream B — Lessons learned and reusable process

| Task | Status | Evidence / next gate |
|---|---|---|
| Nexa-specific lessons learned | Complete working draft | `DEVELOPMENT-LESSONS-LEARNED.md` |
| Architecture rebaseline / stop gates | Complete working draft | `ARCHITECTURE-REBASELINE-GATES.md` |
| Identify permanent cross-project home | Complete | Atlas Engineering Standards Library selected |
| Capture generalized lessons in Atlas | In review | Atlas PR #19 |
| Convert generalized lessons into normative Atlas requirements | Pending | perform only after non-normative lessons review; amend existing owning chapters rather than create parallel hierarchy |

## Workstream C — Define Nexa v1

| Task | Status | Evidence / next gate |
|---|---|---|
| Define finite first-release learner outcome | Complete proposed baseline | `NEXA-V1-DEFINITION.md` |
| Separate required / conditional / post-v1 capabilities | Complete proposed baseline | v1 definition + rebaseline matrix |
| Define product-level acceptance statement | Complete proposed baseline | v1 acceptance statement |
| Human/architecture approval of v1 boundary | Pending | required before v1 definition becomes governing authority |

## Workstream D — Rebaseline system architecture

| Task | Status | Evidence / next gate |
|---|---|---|
| Review reconstructed `NEXA-ARCH-001` against current system | In progress | original architecture and implementation matrix assessed |
| Produce authoritative v1 system architecture proposal | Pending | next architecture artifact |
| Define v1 subsystem ownership and concrete adapter boundaries | Pending | system architecture proposal |
| Define primary learner sequence and data/state flows | Pending | system architecture proposal |
| Reconcile existing ADRs with parent architecture | Pending | after architecture proposal |
| Promote/revise/supersede `NEXA-ARCH-001` through governance | Pending | R0 exit requirement |

## Workstream E — Mature v1 parent specifications

| Specification family | Status | Required action |
|---|---|---|
| Domain / Events | Pending rebaseline | close v1 domain/event scope and durable runtime requirements |
| Student / Lessons / Assessment / Pedagogy | Pending rebaseline | reconcile deterministic slices with persistent product behavior |
| Knowledge / Tutor | Pending rebaseline | consolidate existing ADR decisions and define concrete v1 execution/quality requirements |
| Orchestrator | Pending rebaseline | define complete learner workflow, timeout, retry, recovery and dependency composition |
| Speech / Avatar / Labs | Decision pending | include only if explicitly promoted into v1 |

## Workstream F — Critical cross-cutting specifications

| Area | Status | v1 disposition |
|---|---|---|
| UX | Missing | Required |
| Data / Persistence | Missing | Required |
| Security | Missing | Required |
| Privacy | Missing | Required |
| Observability | Missing | Required |
| Testing / System Acceptance | Missing | Required |
| Packaging | Missing | Required |
| Deployment | Missing | Required |
| Performance | Missing | Required |
| Engineering / Governance | Working controls exist | Required; project controls should reference Atlas once normative standards mature |
| Authoring | Missing | Post-v1 by default |
| Analytics | Missing | Post-v1 by default |
| Plugins | Missing | Post-v1 |
| Public API | Missing | Post-v1 by default |

## Workstream G — Completion roadmap

| Task | Status | Evidence / next gate |
|---|---|---|
| Replace open-ended narrow-increment path with release path | Complete proposed baseline | `NEXA-COMPLETION-ROADMAP.md` |
| Define R0–R9 exit gates | Complete proposed baseline | completion roadmap |
| Prioritize primary learner-journey blockers | Complete proposed rule | roadmap work-selection priority |
| Bind every implementation task to a release blocker | Pending enforcement | begins only after R0/R1 authority work |

## Workstream H — Resume implementation

Status: **Blocked intentionally.**

Implementation does not resume until at minimum:

1. v1 boundary is approved;
2. v1 system architecture is authoritative enough to govern implementation;
3. v1-required parent specifications have sufficient maturity;
4. critical cross-cutting R1 specifications exist;
5. inherited deferrals due at R0/R1 are dispositioned;
6. repository status/registry/roadmap terminology is reconciled;
7. the first R2 walking-skeleton increment can be traced to an explicit release blocker.

## Current next action

Complete the v1 system architecture proposal and use it to drive the parent-specification and cross-cutting specification rebaseline. Do not dispatch product implementation while this ledger reports Workstream H as blocked.
