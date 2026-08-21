# ADR-0029: Deterministic provider-neutral availability-gated selection

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 caller-supplied availability eligibility boundary

## Context

ADR-0026 provides an immutable registry and ADR-0027 selects one statically eligible model without invocation. Neither accepts explicit evidence that a registered model is available for a particular selection. NEXA-TUTOR-001 discusses broader routing and availability behavior, but remains Baseline Draft; health infrastructure, recovery policy, and dynamic routing are not approved by this slice.

## Decision

`nexa-tutor` owns a closed V1 `ModelAvailabilitySnapshot`. It is caller-supplied deterministic eligibility evidence containing only the supported version and a bounded, canonically ordered list of exact existing `(ModelProviderId, ModelId)` identities with an `available` or `unavailable` state. Standalone decoding validates the version, rejects unknown fields, duplicates, non-canonical provider-then-model ordering, and excessive entry counts. An empty snapshot is valid. A constructor may canonicalize input before validation and deterministic serialization.

`select_available_model` validates the snapshot and every snapshot identity against the immutable ADR-0026 registry. Unknown identities and descriptor/registry disagreement fail closed. A registered model omitted from the snapshot, or explicitly marked unavailable, is unavailable for this selection.

Availability gates ADR-0027 eligibility; it does not replace ADR-0027 ordering. The operation reuses requirements validation, privacy eligibility and ordering, capability checks, maximum-output limits, conservative byte-context capacity, and canonical identity tie-breaking. It returns the original registered shared provider handle and descriptor. Selection is synchronous and non-invoking: it does not call `generate`, consume scripted outcomes, retry, or construct a fallback chain. Closed, content-free errors distinguish invalid availability, unsupported availability version, snapshot/registry inconsistency, and nested ADR-0027 selection failure (including no available eligible model).

The snapshot is eligibility evidence only. It does not establish freshness, authenticity, continuous monitoring, health probing, or recovery policy, and it does not prove that a state is still current. It contains no prompt, output, learner, knowledge, diagnostic, endpoint, credential, cost, or latency data.

## Privacy and composition boundaries

The snapshot does not authorize remote execution. ADR-0027 privacy preferences remain eligibility and ordering input rather than privacy filtering or authorization. ADR-0028 is unchanged and does not consume this snapshot; its explicit local-only selection/invocation/admission composition retains its existing contract.

## Consequences and deferrals

Freshness and authenticity policy, health probing and monitoring, recovery, fallback, retry, dynamic latency/cost/task routing, automatic local-first routing, remote authorization and privacy filtering, provider tokenization, concrete inference/providers, networking, async/streaming, telemetry, persistence, repair/regeneration, and capability degradation remain deferred. No clock, endpoint, credential, configuration loader, or mutable health service is introduced. `partial truncation` remains deferred.

NEXA-TUTOR-001 remains Baseline Draft. Phase 4 remains in progress. ADRs 0021–0028 retain their meaning, APIs, and execution boundaries.
