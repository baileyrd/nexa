# Phase 5 traceability

Phase 5 is **in progress**. This matrix records the ADR-0051 through ADR-0056 lifecycle, planning, port, owned-task, target-association, and atomic exact-plan execution foundations; it does not claim a complete session runtime.

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
| Closed-target owned-task association | `WorkflowTaskGroup::spawn_for_target` and bounded per-target counts; direct all-five, multiple/simultaneous, exact return, rejection, root cancellation, natural drain, identity, panic, and abort-on-drop tests | Implemented foundation |
| Atomic exact-plan runtime execution | Side-effect-free exact coverage preflight; global spawn closure; selective target and unclassified cancellation/joining; canonical stopped or target/count non-cancellable evidence; repeat/conflict and abort-on-drop tests | Implemented foundation (ADR-0056) |
| Five-subsystem cancellation binding and subsystem-specific non-cancellable reporting | ADR-0056 supplies generic runtime execution/evidence only; concrete retrieval, tutor, speech, behavior, and tool adapters and composition wiring remain absent | Not implemented |

NEXA-ORCH-001 remains Baseline Draft. Tokio ownership and generic exact-plan execution are the only async runtime capabilities implied; no concrete subsystem I/O, provider, speech, renderer, tool, persistence, networking, clock, health, or recovery integration is implied.
