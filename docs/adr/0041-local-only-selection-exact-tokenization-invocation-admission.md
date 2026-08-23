# ADR-0041: Local-only selection, exact tokenization, invocation, and admission

- Status: Accepted
- Date: 2026-08-22
- Scope: Narrow Phase 4 synchronous host composition

## Context

ADR-0028 composes explicit local-only ADR-0027 selection with one ADR-0025 invocation and strict
admission, but does not tokenize the selected model input. ADR-0040 composes exact tokenization,
capacity validation, one supplied-provider invocation, and admission, but deliberately performs no
selection. Callers need one opt-in operation joining these boundaries without reproducing their
ordering or weakening their existing contracts.

## Decision

`nexa-tutor` provides `select_local_model_tokenize_invoke_and_admit`. The caller supplies the
immutable model registry, canonical invocation identity, model-selection requirements,
tokenization contract version, one model-bound tokenizer, exact prompt compilation result, trusted
planning authority, context package, and citation result.

The operation first enforces unchanged ADR-0028 explicit local-only composition requirements. It
then delegates unchanged ADR-0027 deterministic selection over the exact compiled `ModelInput` and
constructs the existing ADR-0022 request from the selected descriptor, caller invocation identity,
exact input, and unchanged requirements. Only then does it delegate exact tokenization, capacity
validation, one provider invocation, and strict admission to unchanged ADR-0040.

Selection remains governed by ADR-0027's conservative UTF-8 byte eligibility; exact token counts
do not influence selection. Because ADR-0040 receives the selected provider, its tokenization
preflight rejects a contract-version or selected-descriptor mismatch before consuming either
dependency. Success returns ADR-0040's exact replayable tokenization evidence and existing
`AdmissionResult`.

A closed, content-free error distinguishes invalid local-only composition requirements, ADR-0027
selection failure, and nested ADR-0040 tokenized invocation/admission failure. There is no second
selection, tokenizer call, provider call, retry, repair, regeneration, or fallback. Non-selected
and remote provider outcomes remain untouched.

## Consequences and deferrals

ADRs 0027, 0028, and 0040 remain unchanged and independently callable. This composition adds no
token-aware selection, dynamic or automatic local-first routing, availability or health policy,
remote authorization or filtering, fallback, concrete tokenizer or provider, inference,
networking, endpoint, credential, persistence, UI, async/streaming, semantic validation, or new
dependency. NEXA-TUTOR-001 remains Baseline Draft, the known ingestion/context-assembly checklist
inconsistency is preserved, and Phase 4 remains in progress.
