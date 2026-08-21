# ADR-0028: Deterministic local-only selection-to-invocation-to-admission composition

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 explicit local-only synchronous composition boundary

## Context

ADR-0027 selects one eligible registered model without invoking it. ADR-0025 validates trusted host inputs, invokes one explicitly supplied provider at most once, and admits its exact output. Hosts need one narrow operation that joins these existing contracts for an explicitly requested local-only execution without creating a general router or treating a broader privacy preference as remote authorization.

## Decision

`nexa-tutor` adds `select_local_model_invoke_and_admit` to its existing generation surface. The caller supplies an immutable ADR-0026 `ModelRegistry`, a canonical `ModelInvocationId`, ADR-0027 `ModelSelectionRequirements`, the exact ADR-0023 `PromptCompilationResult`, ADR-0024 `TrustedPlanningAuthority`, and governed `ContextPackage` and `CitationResult`. Provider and model identities are not caller inputs.

Before selection or invocation, the operation requires the supported selection version, a non-zero maximum output count, required structured output, and exactly one privacy preference equal to `LocalOnly`. Empty, duplicate, remote-only, multiple, or fallback-like privacy lists fail closed. The operation neither adds a capability nor rewrites caller requirements.

The exact compiled `model_input` and unchanged validated requirements are passed to ADR-0027 `select_model`. Selection therefore retains descriptor validation, capability and conservative byte-context eligibility, output limits, canonical provider/model ordering, registry consistency checking, and non-invoking failure behavior. The operation constructs ADR-0022's exact supported `ModelRequest` using the caller invocation identity; selected descriptor provider/model identities; exact compiled input; and exact required capabilities and maximum output tokens.

The selected shared provider and constructed request are passed to unchanged ADR-0025 `invoke_and_admit_model_output`. Its shared preflight, prompt replay checks, governed association checks, exactly-once invocation, and strict output admission remain authoritative. Success is the existing `AdmissionResult` and `AdmissionEvidence`; no competing selection or identity evidence is added.

A closed content-free error distinguishes invalid local-only composition requirements, nested ADR-0027 selection failure, and nested ADR-0025 invocation/admission failure. Invalid requirements, selection failure, or preflight failure invoke no provider. Invocation or post-invocation admission failure causes no retry, repair, regeneration, or fallback. Only the selected provider can consume one normal scripted outcome; non-selected local and all remote providers remain untouched.

## Privacy and execution boundary

This operation can invoke only a descriptor whose privacy class is exactly `LocalOnly`. `ApprovedRemote` and `RestrictedRemote` are never authorized or invoked. The caller privacy list remains selection input, not authorization. This is explicit local-only execution with no fallback, not automatic local-first routing.

Context privacy filtering, redaction-policy implementation, and remote-provider authorization remain deferred. Existing prompt redaction and content-free diagnostic boundaries are unchanged.

## Consequences and deferrals

ADR-0025's original explicitly supplied-provider API remains available and unchanged. ADR-0027 remains a standalone non-invoking selector. Selection is not made implicit in either existing API, and prompt wire contracts and output-admission semantics do not change.

Dynamic routing and health/availability policy; latency, cost, and task-complexity routing; fallback and capability degradation; retry, repair, and regeneration; concrete providers and inference; provider tokenization; semantic validation, safety, prompt-injection detection, truth, entailment, and hallucination control; tools; async/streaming execution; networking; telemetry export; persistence and durable adapters; and partial truncation remain deferred. No dependency is added. NEXA-TUTOR-001 remains Baseline Draft, and Phase 4 remains in progress.
