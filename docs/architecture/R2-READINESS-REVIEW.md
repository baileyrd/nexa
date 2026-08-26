# Nexa R2 Readiness Review

Review date: 2026-08-26
Review authority: Chief Systems Architect under owner-delegated tactical-pause convergence direction
Baseline implementation: PR #109 / `e2345a1bb8825451ea079ff5e350b7765075038a`

## Decision

**R2 architecture/specification readiness: PASS.**

**R2 implementation start: CONDITIONAL PASS pending only the exact-final-head review, required CI success, and merge of PR #110.**

No additional product/technology decision is required before the first R2 implementation increment.

## Gate review

| Gate | Result | Evidence |
|---|---|---|
| Finite v1 product boundary | PASS | `NEXA-V1-DEFINITION.md` |
| Governing parent architecture | PASS | `NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md` |
| R1 implementation requirements | PASS | `NEXA-R1-IMPLEMENTATION-BASELINE.md` |
| R2 technology/scope decisions | PASS | ADR-0068 |
| Data/persistence policy | PASS for R2 | R1 data supplement + SQLite decision |
| Security/privacy trust boundary | PASS for R2 | R1 supplements + local-model-only R2 posture |
| Complete workflow ownership | PASS for R2 | NEXA-ARCH-002 + R1 orchestrator supplement |
| UX boundary | PASS for R2 | `apps/nexa-desktop` + egui suitability spike gate |
| Testing/maturity model | PASS | R1 testing/acceptance supplement |
| First governed course | PASS for R2 start | TCP course scope approved; authored package is R2 implementation work |
| Observability requirements | PASS for R2 | content-safe correlation baseline |
| Performance policy | PASS for R2 | measurement-first; exact release budgets measured later |
| Packaging constraints | PASS for R2 | Windows acceptance target chosen; final installer is later gate |
| Deferral ownership | PASS | deferral register + R1 later-release classification |
| Documentation authority consistency | PASS after PR #110 merge | baseline/status/registry/README/roadmap reconciled in PR #110 |
| CI on exact final documentation head | PENDING | must be green before merge |

## Scope decisions

### Required in R2

- text learner interaction;
- learner desktop app boundary;
- SQLite persistence;
- one governed TCP lesson package;
- one concrete local llama.cpp provider path;
- governed knowledge/context/citation path;
- assessment/evidence/mastery/progress commit;
- restart/resume;
- content-safe observability.

### Explicitly not R2 exit criteria

- speech input/output;
- animated avatar/behavior integration;
- labs/tools;
- remote provider support;
- dynamic multi-provider routing/fallback;
- dedicated vector database;
- durable event broker/outbox;
- final signed installer/update implementation.

## Inherited Phase 1–5 disposition

Existing implementation is retained according to demonstrated maturity.

- Contract/headless/conformance foundations: reusable.
- Runtime/cancellation foundations: reused only when required by actual learner workflow.
- Provider-neutral abstractions: retained; not expanded merely for completeness.
- Speech/avatar/lab foundations: retained but outside R2 priority.
- No existing accepted ADR is invalidated by default.

## First implementation increment recommendation

The first R2 implementation should be bounded but vertical-enabling:

**Activate `apps/nexa-desktop` and `crates/nexa-storage` as real workspace boundaries, prove the egui application shell and SQLite transaction/migration adapter architecture, and add Windows release-critical CI/check coverage proportional to those new boundaries.**

This first increment should not attempt the entire E2E lesson.

It should establish:

- real learner app compilation/startup shell;
- async-safe UI orchestration seam without tutor logic in the UI;
- real SQLite open/migration/transaction infrastructure behind storage-owned interfaces;
- enforced dependency boundaries preventing `rusqlite`/UI leakage into domain crates;
- concrete tests for store initialization, transaction rollback, schema version handling, and application/storage composition startup;
- Windows CI/build evidence for the new release-critical surfaces.

The next increments should immediately continue the vertical path: durable learning state -> governed content/retrieval -> local model adapter -> tutor/UI composition -> assessment/progress/restart E2E.

## Stop conditions during R2

Return to Redirect/Tactical Pause if:

- a required parent specification proves ambiguous/contradictory at implementation time;
- the egui spike disproves the approved UI path;
- SQLite cannot satisfy the existing learning atomicity/recovery requirements without architectural distortion;
- llama.cpp adapter compatibility cannot satisfy bounded request/output requirements;
- multiple consecutive PRs add horizontal abstraction without making the R2 E2E path more concrete;
- documentation/status begins overstating demonstrated maturity again.

## Final R2 start condition

Once PR #110 is green on its exact final head, reviewed, and merged unchanged, the architecture gate outcome becomes **Continue — begin R2**.

---

## 2026-08-26 owner-authority reconciliation (controlling addendum)

ADR-0069 records the explicit owner decisions from Issue #114. This addendum supersedes earlier text in this document only where it conflicts. Earlier `eframe`/`egui`, `llama.cpp`, text-first release, desktop-only, speech/avatar deferral, owner-delegation, or general R2-Continue/readiness language is preserved as historical evidence and is not active selection authority.

Status: Superseded readiness conclusion; reconciled 2026-08-26

The earlier review incorrectly cited owner-delegated convergence and concluded general R2 Continue. No such delegation occurred. ADR-0069 records the later explicit owner decisions and supersedes the conflicting desktop-only, `egui`, local `llama.cpp`, text-first release, and speech/avatar-deferral conclusions.

Existing contract and runtime evidence remains valid at its demonstrated maturity. General product implementation is **not ready**. The Chief Systems Architect call is **Tactical Pause**, with a narrowly scoped **Continue only for the shared UI/loopback suitability spike after authority reconciliation merges and that spike is separately dispatched**. The spike must meet ADR-0069 and roadmap evidence and may not become production architecture implicitly.
