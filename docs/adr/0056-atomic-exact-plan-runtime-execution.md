# ADR-0056: Atomic exact-plan runtime cancellation execution

- Status: Accepted
- Date: 2026-08-24
- Scope: Narrow Phase 5 exact-plan runtime execution foundation

## Context

ADRs 0051–0053 define lifecycle cancellation, a canonical plan, and a synchronous acceptance port. ADRs 0054–0055 establish private Tokio ownership and target association, but deliberately do not execute a plan. The repository owner approved the first binding from one existing validated plan to its exact live workflow task group, without approving concrete subsystem adapters.

## Decision

`WorkflowTaskGroup::execute_cancellation_plan` consumes one existing `WorkflowCancellationPlan` associated with the group's exact workflow, session, correlation, and trace identities. Before any runtime mutation it validates the supported version, exact identity association, canonical directive invariants, and exact set equality between plan targets and targets with currently owned target-aware work. A live target cannot be omitted, an empty target cannot be added, and an empty plan is valid exactly when no target-aware work is owned. Every failed preflight is side-effect free.

Successful preflight globally closes both spawn paths. The group records the accepted plan, accepted counts, and canonical outcomes before cancellation. `RequestCancellation` cancels only the corresponding private target token and joins every task associated with that target. `ReportNonCancellable` does not cancel its token; its work remains privately owned and the result records its target and exact accepted owned-task count. Existing unclassified `spawn` work is internally placed beneath one dedicated private unclassified token; execution cancels that token and joins every unclassified task. This refactor preserves the public spawn API and legacy workflow-root cancellation behavior.

Execution returns only after all request-cancellation targets and unclassified tasks stop. Its immutable identity-preserving evidence contains canonical target outcomes, accepted non-cancellable counts, the accepted unclassified count, and proof that no unclassified tasks remain. It does not claim all work stopped. Reported non-cancellable tasks remain owned and may still run; owner drop aborts them.

An identical repeat returns the same terminal result without new mutation. A different plan fails with a closed conflict. Legacy `cancel_and_wait` and `drain` retain their prior behavior only when plan execution has not begun. Plan execution and either legacy completion path fail closed against one another. Spawn closure and accepted execution state survive cancellation of the caller's future.

Private task-ID association permits selected and unclassified join failures to be normalized while unrelated non-cancellable tasks remain owned. Public errors are closed and content-free; task IDs, tokens, handles, mutable collections, task output, and panic text never escape.

## Compatibility

ADRs 0051 through 0055 remain unchanged except for the approved internal unclassified-token refactor and this opt-in operation. The synchronous ADR-0052 planner and ADR-0053 port APIs are not changed. No runtime evidence wire format is required because this evidence is neither persisted nor transmitted.

## Consequences and deferrals

This is the first exact-plan runtime execution foundation, not the Phase 5 exit gate. Explicitly deferred are concrete retrieval, tutor, speech, behavior, and tool adapters and composition-root wiring; network/provider cancellation; subsystem-specific non-cancellable payloads; timeouts, retry, fallback, repair, compensation, recovery, and shutdown orchestration; persistence, networking, providers, secrets, authentication or authorization changes, destructive actions, and migrations. NEXA-ORCH-001 remains Baseline Draft and the broad cancellation-safe execution/propagation checklist remains incomplete.
