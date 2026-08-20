# ADR-0025: Single-attempt provider-neutral invocation-to-admission composition

- **Status:** Accepted
- **Date:** 2026-08-20
- **Scope:** Narrow Phase 4 synchronous composition boundary

## Context

ADRs 0021–0024 separately define deterministic response planning, provider-neutral invocation,
prompt compilation, and strict model-output admission. They intentionally do not define an
operation that verifies all available host evidence before consuming a provider outcome and then
joins one invocation to admission.

## Decision

`nexa-tutor` owns a dependency-light, synchronous composition operation. The caller supplies the
provider, exact request, compilation evidence, trusted planning authority, context package, and
citation result. Provider selection, routing, fallback, and privacy authorization remain the
caller's or composition root's responsibilities.

Before provider consumption, the operation uses the shared ADR-0024 preflight to validate the
descriptor; structured-output requirement and support; request/descriptor association and limits;
supported invocation, compilation, prompt-package, context-builder, and output-schema contracts;
intrinsic compilation evidence and exact compiled input; intrinsic authority, context, and
citation evidence; and all scope, context, citation, hybrid-result, retrieval-query, governance,
and citation-policy associations. Diagnostics are closed and content-free.

Successful preflight permits exactly one call to the supplied `LanguageModelProvider`. The exact
returned `ModelResponse` proceeds through the remaining ADR-0024 admission path. Success returns
the existing `AdmissionResult`. Closed errors distinguish preflight failure, provider invocation
failure with its normalized `ModelErrorKind`, and post-invocation `AdmissionError`.

There is no retry, repair, regeneration, fallback, partial success, or second provider call.
`FinishReason::OutputLimit` remains a failed admission.

## Consequences and deferrals

This composition proves no inference quality, truth, entailment, semantic safety, prompt-injection
resistance, instructional quality, or remote-provider privacy authorization. It adds no concrete
provider, network transport, credentials, async runtime, streaming, persistence, telemetry export,
router, registry, or model-selection policy. Provider tokenization, actual inference, privacy
filtering/authorization, semantic validation, repair/regeneration, and partial truncation remain
deferred. ADRs 0021–0024 retain their meanings and ownership boundaries.
