# ADR-0034: Deterministic filtered, authorized, available remote-model selection

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 non-invoking ADR-0033-to-ADR-0031 composition

## Context

ADR-0033 produces intrinsically validating evidence for caller-directed whole-layer disclosure filtering and an exact filtered ADR-0023 compilation. ADR-0031 selects an authorized, available remote model for an exact compilation but does not require ADR-0033 evidence. A narrow composition is required without changing either decision or invoking ADR-0032.

## Decision

`nexa-tutor` provides `select_filtered_authorized_available_remote_model`. It first intrinsically validates the complete ADR-0033 result, including its policy, filtered compilation, source-present inventory, exact included/omitted partition, associations, and replay anchors. Invalid or reassociated evidence fails before selection or provider consumption.

The unchanged selection requirements must contain exactly one privacy preference, it must be `ApprovedRemote` or `RestrictedRemote`, and it must exactly equal the disclosure policy target. Caller requirements are never normalized or rewritten. The operation then passes the exact filtered compilation and unchanged registry, requirements, availability, and authorization to ADR-0031's `select_authorized_available_remote_model`. Consequently, authorization must bind to the filtered compilation replay anchor; authorization for the source compilation cannot authorize the filtered compilation, or vice versa. ADR-0031 remains authoritative for authorization validation, registry association, availability, eligibility, deterministic ordering, privacy matching, and return of the original registered shared provider handle.

The operation is synchronous and non-invoking. It performs no provider call or state consumption, prompt transmission, lookup consumption, networking, filesystem or clock access, randomness, or persistence. Closed content-free failures distinguish filter/privacy requirement mismatch, invalid or unassociated filter evidence, and nested ADR-0031 failure.

## Boundaries and deferrals

Valid structural evidence does not prove that the caller's disclosure decision is semantically correct. This decision adds no general privacy policy, privacy-policy correctness, sensitivity inference, semantic/content minimization, field or substring redaction, anonymization, or partial truncation. It does not invoke ADR-0032 or automatically execute after selection.

Concrete providers and inference; networking, endpoints, credentials, tokenization; dynamic routing, automatic local-first routing, fallback, retry, repair, or recovery; semantic validation; tools; async/streaming; telemetry; and persistence remain deferred. NEXA-TUTOR-001 remains Baseline Draft, the privacy specification namespace remains reserved and unimplemented, and Phase 4 remains in progress.
