# Phase 5 traceability

Phase 5 is **in progress**. This matrix records the ADR-0051 lifecycle, ADR-0052 propagation-planning, ADR-0053 propagation-port, and ADR-0054 Tokio owned-task foundations; it does not claim a complete session runtime.

| Requirement | Evidence | Status |
|---|---|---|
| Canonical workflow identity | `nexa-domain::WorkflowId`; direct nil construction/wire tests | Implemented foundation |
| Closed session lifecycle | `RuntimeSessionState::transition_to`; exhaustive all-pairs and terminal/failure tests | Implemented foundation |
| Closed interaction workflow lifecycle | `InteractionWorkflow::advance`; exhaustive all-pairs legal/illegal tests | Implemented foundation |
| Lifecycle cancellation | `InteractionWorkflow::cancel`; every nonterminal, idempotency, completed/failed rejection tests | Implemented foundation |
| Exact identity association | Reference-only aggregate; pure four-identity trusted-association validation; direct reassociation, operation-preservation, and wire-preservation tests | Implemented foundation |
| V1 deterministic strict wire | validating round trips and unknown version/field/variant/nil rejection | Implemented foundation |
| Content-free diagnostics | closed errors and direct `Debug`/`Display` tests | Implemented foundation |
| Dependency-light synchronous boundary | workspace metadata boundary script and forbidden-capability scans | Implemented foundation |
| Deterministic propagation planning | `plan_workflow_cancellation`; private validated contract construction; direct all-target/semantics, five-target bound, permutation, duplicate, empty, lifecycle, association, strict-wire matrix, identity, and operation-diagnostic tests | Implemented foundation |
| Exact-plan propagation port | `propagate_workflow_cancellation`, `WorkflowCancellationPropagationPort`, and `WorkflowCancellationAcknowledgement`; direct exact-JSON/round-trip and strict negative wire matrices, target × directive semantics, all operation failures, identity and acknowledgement mismatch, FIFO, exhaustion, exact consumption, failure-normalization, and returned-error diagnostics tests | Implemented foundation |
| Deterministic scripted propagation adapter | `ScriptedWorkflowCancellationPropagationPort`; direct FIFO, zero/one outcome consumption, exact received-plan, dependency failure, and exhaustion tests | Implemented foundation |
| Tokio owned-task structured concurrency | `nexa-orchestrator-runtime::WorkflowTaskGroup`; private root token and `JoinSet`, child token per spawn, no handle escape, closed inspection, spawn rejection, and exact four-identity completion evidence | Implemented foundation |
| Directly owned task cancellation and completion | Deterministic cooperative single/multiple-task, idempotency, natural-drain, normalized panic, empty-after-return, and abort-on-drop tests | Implemented foundation |
| Five-subsystem cancellation binding and non-cancellable reporting | Explicitly deferred by ADR-0054; no ADR-0052 directive or ADR-0053 acknowledgement is bound to a live task group | Not implemented |

NEXA-ORCH-001 remains Baseline Draft. Tokio task ownership is the only async runtime capability implied; no subsystem I/O, provider, speech, renderer, tool, persistence, networking, clock, health, recovery, or orchestration capability is implied.
