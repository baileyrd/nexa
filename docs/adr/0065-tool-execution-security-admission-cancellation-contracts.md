# ADR-0065: Tool Execution security admission and cancellation contracts

- Status: Accepted
- Date: 2026-08-25
- Scope: Provider-neutral Tool Execution contract foundation

## Context

Tool Execution is the remaining Phase 5 cancellation target, but binding it in the headless application before defining its security prerequisites would permit orchestration to imply that an unsafe or insufficiently associated action was executable. NEXA-LAB-001 requires isolation, bounded resources, authorization, risk, confirmation, and cancellation semantics without assigning concrete execution or sandbox mechanisms to the contract layer.

## Decision

Activate dependency-light `nexa-labs`, depending normally only on `nexa-domain`, Serde, and content-free error support. Canonical non-nil lab-session, tool-request, tool-execution, and environment-instance identifiers live in `nexa-domain`; validated `SemanticKey` identifies tools and operations. An opaque fixed-size request-content digest completes the exact association without retaining commands, arguments, paths, environment variables, credentials, tokens, or output.

V1 defines structural sandbox preflight: implicit host filesystem and network access, root or privileged declarations fail closed; CPU, memory, storage, process, execution-time, and output bounds must each be nonzero; network policy, mounts, and capabilities are explicit. Deny-all structurally carries no network targets, while allow-listed networking requires a nonempty, strictly ordered, duplicate-free list of provider-neutral transport and endpoint keys. This validates only the declaration and does not enforce or prove real isolation.

External security and assessment policy authorities supply strict V1 evidence containing the exact association, classified risk, and closed deny/allow/confirmation-required result. The risk classification is likewise strict V1 evidence for the exact association. No policy evaluation is implemented. Every association, version, or risk mismatch fails closed; denial precedes dependency use; exact confirmation is required when directed; and confirmation binds the association, risk, and both policy consequences. As a deliberate stronger rule, destructive and privileged actions always require exact confirmation even when generic allow is supplied. Tutor preference cannot weaken security or assessment restrictions and is never authorization or confirmation.

A side-effect-free exact capability declares cancellable or non-cancellable semantics. The object-safe control port returns an erased standard-library future. Cancellable admission invokes the exact dependency once, owns its future through terminalization or caller drop, and accepts only an exact acknowledgement. Non-cancellable handling returns immutable associated evidence without invoking the dependency. Acknowledgement and joined control-future evidence mean only dependency acceptance and terminalization at this boundary; they do not prove that any process, command, database commit, destructive action, hardware operation, provider, external side effect, or tool execution stopped.

Caller-supplied authorization, confirmation, isolation, capability, acknowledgement, and evidence establish structural association only. They do not prove authenticity, freshness, identity, runtime enforcement, completion, safety, or external behavior.

## Consequences and deferrals

The boundary includes deterministic scripted cancellation participants and content-free closed diagnostics for direct contract testing. It adds no Tokio, async-trait, process, filesystem, network, storage, database, container/VM, provider, UI, renderer, OS, persistence, event, telemetry, timeout, retry, or recovery implementation.

Authentication, user-identity proof, expiry clocks, policy authenticity/freshness and evaluation, confirmation UI, secrets, sandbox provisioning/enforcement, actual tool execution and registry behavior, concrete providers, and the headless `CancellationTarget::ToolExecution` binding are deferred. Orchestrator and runtime semantics are unchanged. Phase 5 and the five-subsystem cancellation gate remain incomplete; headless Tool Execution binding is the next separately reviewed increment.
