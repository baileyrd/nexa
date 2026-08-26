# Nexa Tactical-Pause Rebaseline Task Ledger

Status: Active execution control for the owner-authorized tactical pause

Baseline: `e2345a1bb8825451ea079ff5e350b7765075038a` (PR #109)

## Purpose

This ledger is the execution control for the tactical pause. It prevents the assessment itself from becoming an open-ended documentation exercise and records the evidence required before normal implementation resumes.

The ledger's active status authorizes only the pause and this review workflow. It does **not** approve the proposed v1 boundary, system architecture, completion roadmap, rebaseline gates or matrix, R1 plan, or R1 specification drafts. Existing accepted ADR and registry-listed specification authority is preserved until explicit owner decisions are recorded.

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
| Review proposed v1 system architecture | Proposed; owner review pending | `NEXA-V1-SYSTEM-ARCHITECTURE.md`; proposal is not authoritative or complete |
| Review proposed v1 subsystem ownership and concrete adapter boundaries | Proposed; owner review pending | architecture proposal sections 4-10; no approval implied |
| Review proposed primary learner sequence and data/state flows | Proposed; owner review pending | architecture proposal sections 11-13; reconcile during R0 review |
| Reconcile existing ADRs with parent architecture | Pending | after architecture proposal |
| Promote/revise/supersede `NEXA-ARCH-001` through governance | Pending | R0 exit requirement |

## Workstream E — Mature v1 parent specifications

| Specification family | Status | Required action |
|---|---|---|
| Domain / Events | Proposed R1 rebaseline draft | Non-authoritative review input; existing Baseline Draft statuses are unchanged |
| Student / Lessons / Assessment / Pedagogy | Proposed R1 rebaseline draft | Non-authoritative review input; existing specification statuses are unchanged |
| Knowledge / Tutor | Proposed R1 rebaseline draft | Non-authoritative review input; existing specification statuses are unchanged |
| Orchestrator | Proposed R1 rebaseline draft | Non-authoritative review input; `NEXA-ORCH-001` remains Baseline Draft pending reconciliation and approval |
| Speech / Avatar / Labs | Owner decision pending | explicit v1/post-v1 disposition required before implementation resumes; this ledger chooses neither |

## Workstream F — Critical cross-cutting specifications

| Area | Status | v1 disposition |
|---|---|---|
| UX | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| R1 specification plan | Proposed review input | Does not start or close R1 and authorizes no implementation |
| Data / Persistence | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Security | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Privacy | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Observability | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Testing / System Acceptance | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Packaging / Deployment | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Performance | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
| Governed content / release | Proposed R1 draft | Non-authoritative review input; required family remains unapproved |
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
3. the owner has explicitly classified speech, avatar/behavior, and labs/tools as v1 or post-v1;
4. v1-required parent specifications have sufficient maturity;
5. critical cross-cutting R1 specifications are reviewed and approved at the maturity required to govern implementation; draft presence is insufficient;
6. inherited deferrals due at R0/R1 are dispositioned;
7. repository status/registry/roadmap terminology is reconciled;
8. the first R2 walking-skeleton increment can be traced to an explicit release blocker.

## Current next action

Obtain owner review and explicit decisions on the proposed v1 boundary, architecture, conditional-capability disposition, roadmap, gates, matrix, and the scope of the R1 plan/drafts. In parallel, complete the remaining R0 reconciliation of `NEXA-ARCH-001`, existing ADRs, parent specifications, README, registry, roadmap, status, and traceability terminology. The data/persistence, security, privacy, observability, and orchestrator rebaseline drafts are review inputs only; they do not close R1, mature an existing specification, or authorize implementation. Do not add further R1 specification families in this correction, and do not dispatch product implementation while Workstream H is blocked.
