# ADR-0032: Deterministic authorized available remote selection-to-invocation-to-admission composition

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 explicitly authorized remote synchronous composition boundary

## Context

ADR-0031 deterministically selects a statically eligible remote model only when caller-supplied authorization for the exact ADR-0023 prompt and caller-supplied availability independently admit it, but deliberately never invokes a provider. ADR-0025 invokes one explicitly supplied provider at most once and strictly admits its exact response. A narrow composition is needed without adding general routing, fallback, concrete provider integration, or a second permission mechanism.

## Decision

`nexa-tutor` adds `select_authorized_available_remote_model_invoke_and_admit`. The caller supplies an immutable registry, invocation identity, unchanged selection requirements, availability, remote authorization, exact prompt compilation, trusted planning authority, governed context, and governed citations. Explicit caller authorization bound to the compilation replay anchor is the permission boundary for sending that exact compiled prompt to the selected remote provider/model.

The operation passes the exact registry, unchanged requirements, availability, authorization, and compilation to ADR-0031. Its remote-only privacy validation, prompt association, independent authorization/availability/static-eligibility/registry gates, privacy-order preference, and canonical provider/model tie-break remain authoritative. `LocalOnly`, mixed local/remote, empty, or duplicate privacy requirements fail closed.

After selection, the operation constructs exactly ADR-0022's supported `ModelRequest`: the caller invocation identity; selected provider and model identities; exact compiled `ModelInput`; unchanged required capabilities and maximum output count; and supported invocation contract version. It passes the registry's original selected provider handle and exact request to unchanged ADR-0025 `invoke_and_admit_model_output`.

Only the selected provider may be invoked, exactly once and only after selection succeeds. Once invocation begins there is no second selection, retry, fallback, repair, regeneration, recovery, or capability degradation. Success is unchanged ADR-0024 `AdmissionResult`; no parallel identity, authorization, availability, request, response, admission, or success evidence is introduced. A closed content-free error distinguishes nested ADR-0031 failure from nested ADR-0025 failure.

## Permission and privacy boundary

Caller authorization permits this composition to transmit the exact compiled prompt only to an authorized, available selected remote identity. The operation does not verify authorization authenticity or freshness. It does not perform or prove context filtering, redaction, minimization, or sensitivity inference; those responsibilities remain upstream and deferred.

No concrete provider, inference implementation, network transport, endpoint, credential, configuration loader, health probe, clock, or monitoring facility is added.

## Consequences and deferrals

ADRs 0021–0031 and their public APIs retain their meanings. This is not automatic local-first or general routing. Authenticity/freshness proof; context privacy filtering/redaction/minimization and sensitivity inference; fallback/retry/repair/regeneration/recovery; concrete providers and inference; provider tokenization; networking; async/streaming/cancellation; semantic validation; tools; telemetry; persistence; and `partial truncation` remain deferred. NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress.
