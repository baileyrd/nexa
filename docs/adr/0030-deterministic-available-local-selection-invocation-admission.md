# ADR-0030: Deterministic available local selection-to-invocation-to-admission composition

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 caller-availability-gated explicit local-only synchronous composition

## Context

ADR-0029 gates ADR-0027 selection with caller-supplied availability evidence but never invokes a
provider. ADR-0028 composes explicit local-only static selection with ADR-0025's single-attempt
invocation and admission, but deliberately does not acquire or consume availability. Hosts need a
narrow operation that composes these boundaries without creating general routing, automatic
local-first behavior, remote authorization, health monitoring, or recovery policy.

## Decision

`nexa-tutor` adds `select_available_local_model_invoke_and_admit`. The caller supplies an
immutable `ModelRegistry`, invocation identity, unchanged `ModelSelectionRequirements`, an
existing `ModelAvailabilitySnapshot`, exact `PromptCompilationResult`, trusted planning
authority, context package, and citation result.

The operation first requires the supported selection version, non-zero maximum output count,
structured output, and exactly one privacy preference equal to `LocalOnly`. Empty, duplicate,
remote-only, multiple, malformed, or fallback-like privacy lists fail before selection or provider
consumption. It passes the exact compiled `ModelInput`, unchanged requirements, and exact snapshot
to ADR-0029 `select_available_model`. ADR-0029's snapshot validation, registry association,
missing-is-unavailable rule, and ADR-0027 eligibility and canonical ordering remain authoritative.
Available remote entries do not become eligible: the snapshot is evidence, not authorization.

The selected descriptor supplies the provider and model identities for an exact ADR-0022 V1
`ModelRequest`; the caller supplies its invocation identity, while the compiled input, required
capabilities, and maximum output count are copied unchanged. The original registered provider
handle and request pass to unchanged ADR-0025 `invoke_and_admit_model_output`. At most that one
provider is invoked exactly once. Success is the existing ADR-0024 `AdmissionResult` and evidence;
no competing success evidence is introduced.

An unavailable or omitted model may be excluded during the initial deterministic eligibility
pass. Choosing another model already marked available is not fallback. After selection and an
invocation attempt, there is no second selection, fallback, retry, repair, regeneration, recovery
chain, or capability degradation. Non-selected local providers and every remote provider remain
untouched.

A closed, content-free error distinguishes invalid explicit-local requirements, nested ADR-0029
availability/selection failure, and nested ADR-0025 invocation/admission failure. It contains no
prompt, output, learner, knowledge, endpoint, credential, or provider-private content.

## Preserved boundaries and deferrals

Availability is caller-supplied evidence, not health probing, freshness/authenticity proof,
monitoring, a clock, or a mutable health service. Execution is explicitly `LocalOnly`; the
snapshot never authorizes `ApprovedRemote` or `RestrictedRemote` execution. This is not general
routing or automatic local-first routing.

ADR-0028 remains unchanged and does not implicitly acquire availability. ADR-0029 remains an
independently usable non-invoking selector. ADR-0025 remains available with an explicitly supplied
provider. ADRs 0021–0027 and their ownership and execution boundaries are unchanged.

Remote authorization/privacy filtering; fallback/retry/repair/regeneration; concrete providers or
inference; provider tokenization; latency/cost/task policy; endpoints, credentials, networking,
async/streaming/cancellation/timeouts; semantic safety, truth, entailment, hallucination or
instructional-quality evaluation; tools; telemetry; persistence; and `partial truncation` remain
deferred. No dependency is added. NEXA-TUTOR-001 remains Baseline Draft, and Phase 4 remains in
progress.
