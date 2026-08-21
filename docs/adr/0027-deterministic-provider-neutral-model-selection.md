# ADR-0027: Deterministic provider-neutral model selection

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 static model-eligibility and single-choice boundary

## Context

ADR-0022 defines validated provider/model descriptors and conservative provider-neutral capacity accounting. ADR-0026 provides an immutable registry but deliberately performs no selection. ADR-0025 continues to accept one explicitly supplied provider. A narrow contract is needed to let a host choose one eligible registered model without turning registry mechanics into invocation, routing, deployment, or privacy-authorization policy.

## Decision

`nexa-tutor` owns a closed V1 `ModelSelectionRequirements` contract and the synchronous `select_model(registry, input, requirements)` operation. Requirements contain only the supported contract version, ADR-0022 required capabilities, a non-zero requested maximum output-token count, and a non-empty ordered list of distinct allowed `PrivacyClass` values. Strict deserialization rejects unknown fields and performs standalone validation. Prompt text is never stored in or serialized with requirements; the validated `ModelInput` is supplied separately.

Selection validates the requirements and each registered descriptor. A descriptor is eligible only if its privacy class appears in the caller's list, it supports every required capability, its maximum output limit admits the request, and the exact compiled-input UTF-8 byte length plus the requested output count fits its context window. Model request validation and selection share this ADR-0022 eligibility check, including the conservative rule that one input byte consumes one provider-neutral context unit. This is checkable preflight evidence, not provider tokenization.

Eligible descriptors are ordered first by their privacy-class position in the caller's list, then by canonical `ModelProviderId`, then canonical `ModelId`. Canonical identity is only a stable final tie-break; it is not a product preference. The operation returns exactly one descriptor and the registry's original shared provider handle, preserving `Arc` identity. An invalid or unsupported requirements contract, no eligible model, or detected registry/descriptor inconsistency fails closed with a small content-free error vocabulary.

The selector reads only immutable registry associations and descriptors. It does not invoke a provider, inspect or consume scripted outcomes, retry, build a fallback chain, mutate state, or perform networking, filesystem access, or persistence. Diagnostics and debug output contain no prompt content.

The privacy list is explicit caller-supplied eligibility and ordering data only. It does not filter context, redact content, authorize a remote provider, establish deployment policy, or implement automatic local-first routing. A caller may explicitly put `LocalOnly` first; this decision does not reinterpret that choice as the deferred automatic local-first policy.

## Consequences and deferrals

This increment implements static, deterministic, single-choice selection only. ADR-0025 single-attempt generation is unchanged and still requires an explicitly supplied provider; selection is not integrated into `generate_once` or the admission/invocation pipeline.

Dynamic availability and health; latency/cost and task-complexity policy; fallback, retry, repair, and regeneration; concrete providers and actual inference; provider tokenization; context privacy filtering, redaction, and remote authorization; semantic validation; tools; streaming and async execution; networking; persistence; and partial truncation remain deferred. NEXA-TUTOR-001 remains Baseline Draft, and this increment does not complete Phase 4.
