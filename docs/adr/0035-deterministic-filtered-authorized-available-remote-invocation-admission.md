# ADR-0035: Deterministic filtered authorized available remote invocation and admission

- Status: Accepted
- Date: 2026-08-21
- Scope: `nexa-tutor` Phase 4 composition

## Context

ADR-0033 produces structural whole-layer disclosure evidence and a filtered ADR-0023 compilation. ADR-0034 is the sole filter-evidence-gated, authorized, available remote selection operation, but intentionally does not invoke a provider. ADR-0025 already owns host-side admission preflight, exactly one synchronous invocation of a supplied provider, and strict ADR-0024 admission.

## Decision

Add `select_filtered_authorized_available_remote_model_invoke_and_admit` as a narrow synchronous composition. It requires the complete `RemotePromptFilterResult` and delegates its only selection to unchanged ADR-0034. Consequently ADR-0033 evidence validation, exact singleton target-privacy agreement, authorization bound to the filtered compilation, availability, registry consistency, eligibility, deterministic ordering, and preservation of the original registered provider handle remain authoritative in ADR-0034.

After selection, the existing shared request-construction helper creates the ADR-0022 `ModelRequest` from the caller invocation identity, selected provider/model identities, supported contract version, unchanged requirements, and the exact filtered `ModelInput`. The composition then supplies the original selected provider, exact request, filtered `PromptCompilationResult`, trusted planning authority, governed context, and governed citations to unchanged ADR-0025.

Success is the unchanged ADR-0024 `AdmissionResult`; no duplicate filter, authorization, selection, invocation, response, or admission success evidence is introduced. Its admission evidence therefore binds the filtered compilation replay anchor rather than the source compilation anchor.

No provider is consumed until ADR-0034 selection and ADR-0025 host-side preflight succeed. Once invocation begins there is no second selection, fallback, retry, repair, regeneration, recovery, capability degradation, or alternate-provider attempt.

## Privacy and security consequences

The structural disclosure evidence proves deterministic whole-layer inclusion and omission, association, and replay. It does not prove semantic privacy correctness, policy correctness, sensitivity inference, content-level minimization, redaction, or anonymization. Authorization authenticity and freshness also remain outside this decision.

Errors are closed and content-free, separating nested ADR-0034 selection failure from nested ADR-0025 invocation/admission failure. No new wire-visible contract is added.

## Non-goals

This decision adds no general or automatic routing, automatic local-first policy, fallback, concrete provider or inference, networking, endpoint or credential handling, provider tokenization, partial truncation, semantic validation, tools, async/streaming, telemetry, or persistence. It does not reinterpret or alter ADRs 0021 through 0034; ADR-0034 remains independently non-invoking and ADR-0032 remains independently callable without ADR-0033 evidence.
