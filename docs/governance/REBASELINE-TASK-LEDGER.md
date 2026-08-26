# Nexa Tactical-Pause Rebaseline Task Ledger

Status date: 2026-08-26
Baseline: `e2345a1bb8825451ea079ff5e350b7765075038a` (PR #109)
Rebaseline PR: #110

## Purpose

This ledger records completion of the tactical-pause recovery work and the exact remaining condition before R2 implementation begins.

## Workstream A — Current-state assessment

| Task | Status | Evidence |
|---|---|---|
| Establish authoritative baseline | Complete | PR #109 merge baseline |
| Inventory governing docs/maturity | Complete | `NEXA-CURRENT-STATE-ASSESSMENT.md` |
| Distinguish contract/integration/concrete/user maturity | Complete | assessment + testing maturity model |
| Audit historical divergence | Complete | `DEVELOPMENT-DIVERGENCE-ANALYSIS.md` |
| Centralize inherited deferrals | Complete | `DEFERRAL-REGISTER.md` |
| Reconcile current authority/status view | Complete | rewritten `PROJECT-STATUS.md` and registry |

## Workstream B — Lessons learned and reusable process

| Task | Status | Evidence |
|---|---|---|
| Nexa lessons learned | Complete | `DEVELOPMENT-LESSONS-LEARNED.md` |
| Architecture stop/rebaseline gates | Complete | `ARCHITECTURE-REBASELINE-GATES.md` |
| Permanent cross-project home | Complete | Atlas Engineering Standards Library |
| Generalized lessons captured | Complete | Atlas PR #19 merged |
| Normative Atlas follow-up | Tracked, not R2 blocker | Atlas issue #20 |

## Workstream C — Define Nexa v1

| Task | Status | Evidence |
|---|---|---|
| Finite first-release learner outcome | Approved | `NEXA-V1-DEFINITION.md` adopted by `NEXA-ARCH-002` |
| Required/conditional/post-R2 classification | Approved | `NEXA-ARCH-002`, ADR-0068 |
| Product acceptance statement | Approved | v1 definition + R1 testing baseline |
| Speech/avatar/labs R2 disposition | Resolved | speech/avatar retained but not R2 exit criteria; labs/tools post-R2 |

## Workstream D — Rebaseline system architecture

| Task | Status | Evidence |
|---|---|---|
| Review reconstructed NEXA-ARCH-001 | Complete | current-state/divergence/rebaseline analysis |
| Establish v1 system architecture | Approved | `NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md` |
| Define subsystem/adapter ownership | Approved for R2 | NEXA-ARCH-002 |
| Define primary learner/data flow | Approved | NEXA-ARCH-002 primary R2 sequence |
| Reconcile accepted ADR relationship | Complete for R2 | registry + ADR-0068 |
| Supersede reconstructed architecture for v1 selection | Complete | NEXA-ARCH-002/registry |

## Workstream E — Mature v1 parent specifications

| Family | Status for R2 | Evidence |
|---|---|---|
| Domain / Events | Approved supplement | R1 baseline |
| Student / Pedagogy / Lessons / Assessment | Approved supplement | R1 baseline |
| Tutor / Knowledge | Approved supplement | R1 baseline |
| Orchestrator | Approved complete-workflow supplement | R1 baseline |
| Speech / Avatar / Labs | Retained later capability | not R2 blocking |

## Workstream F — Critical cross-cutting specifications

| Area | R2 disposition |
|---|---|
| Data / Persistence | Approved R1 requirements + SQLite decision |
| Security | Approved local-model R2 trust boundary |
| Privacy | Approved local-only R2 disclosure posture |
| Observability | Approved content-safe correlation requirements |
| Testing / Acceptance | Approved maturity model + R2 E2E gate |
| Learner UX | Approved text-first desktop requirements |
| Performance | Approved measurement-first requirements; final release budgets later |
| Packaging / Deployment | Approved constraints; final installer/signing later |
| Governed Content | Approved first-course scope |
| Engineering / Governance | Approved project gates; Atlas normative follow-up tracked |

## Workstream G — Technology and delivery convergence

| Decision | Status | Authority |
|---|---|---|
| Desktop app boundary | Approved | NEXA-ARCH-002 / ADR-0068 |
| UI framework | Approved for bounded R2 spike | ADR-0068 (`eframe`/`egui`) |
| Durable store | Approved | ADR-0068 (SQLite/`rusqlite`) |
| Concrete model path | Approved | ADR-0068 (local `llama.cpp` server) |
| First acceptance platform | Approved | ADR-0068 (Windows x86_64) |
| First course | Approved | ADR-0068 (TCP Connection Establishment) |
| Event broker/outbox | Not required for R2 | R1 baseline |
| Remote provider | Post-R2 | R1 baseline |

## Workstream H — Completion roadmap

| Task | Status | Evidence |
|---|---|---|
| Replace open-ended increment path | Complete | `NEXA-COMPLETION-ROADMAP.md` |
| Define R0–R9 gates | Complete | completion roadmap |
| Prioritize vertical release blockers | Complete | architecture/roadmap selection rule |
| Adopt capability maturity vocabulary | Complete | registry/status/testing baseline |

## R0 gate

**PASS for R2**, subject only to PR #110 being green and merged on its exact final head.

## R1 gate

**PASS for R2**, subject only to PR #110 being green and merged on its exact final head.

R1 is intentionally not “all possible Nexa specifications complete.” It is “all architecture/specification decisions needed to safely govern the R2 walking skeleton are mature enough.”

## R2 readiness

Status: **Conditionally Ready — documentation merge gate only.**

R2 implementation may begin when:

1. PR #110 final exact head has required CI green;
2. the complete final documentation diff is reviewed;
3. PR #110 is merged unchanged;
4. the first R2 increment cites `NEXA-ARCH-002`, `NEXA-R1-IMPLEMENTATION-BASELINE`, ADR-0068, and the relevant owned subsystem specifications.

No additional owner product/technology decision is required before R2 begins.

## First R2 work-selection rule

The first R2 implementation must advance the vertical walking skeleton, not reopen horizontal Phase 5 work.

Preferred first bounded increment:

- activate the real `apps/nexa-desktop` application shell and the concrete `crates/nexa-storage` SQLite infrastructure boundary in a way that proves their architecture/dependency shape without attempting to finish the full learner flow in one PR;
- immediately follow with the smallest integration increments that connect governed content, learning state, real model adapter, and restart/resume into the E2E path.

Every R2 PR must state which E2E step it makes concrete and which maturity state it advances.

## Remaining later-release work

Not R2 blockers:

- release model selection/quality threshold finalization;
- speech/avatar final v1 inclusion;
- labs/tools;
- remote provider/credentials;
- advanced routing/fallback/vector infrastructure;
- final installer/signing/update mechanism;
- plugin/API/analytics/authoring/server capabilities.

These are owned by later roadmap gates and must not silently leak back into R2 scope.
