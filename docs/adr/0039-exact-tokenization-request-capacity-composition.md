# ADR-0039: Exact tokenization and request-capacity composition

- Status: Accepted
- Date: 2026-08-22
- Scope: Narrow Phase 4 synchronous non-invoking host composition

## Context

ADR-0036 creates exact-model input-tokenization evidence, while ADR-0037 validates an existing ADR-0022 request and that evidence against mandatory conservative byte capacity and exact token capacity. Callers otherwise have to reproduce the required ordering between request preflight, tokenizer consumption, evidence creation, and capacity validation.

## Decision

`nexa-tutor` provides `tokenize_and_validate_model_request_capacity`. The caller supplies the tokenization contract version, exact `ModelDescriptor`, existing `ModelRequest`, and a `ModelInputTokenizer`.

The operation first delegates to unchanged `ModelRequest::validate_for`. Identity, version, capability, output-limit, and conservative UTF-8 byte-capacity failures therefore occur before a tokenizer outcome can be consumed. It then delegates exact counting and evidence creation to unchanged `tokenize_model_input`, which calls the tokenizer exactly once only after its own version and exact-descriptor preflight. Finally it delegates the generated evidence and exact request to unchanged `validate_model_request_token_capacity`. Checked capacity arithmetic, exact-boundary success, evidence association, hashing, serialization, and replay behavior are inherited without duplication. Success returns the exact generated `ModelInputTokenizationEvidence`.

A closed, content-free error distinguishes request preflight, tokenization, and token-capacity failures. The synchronous operation mutates no shared state beyond consuming the one tokenizer outcome. It accepts and invokes no `LanguageModelProvider`.

## Consequences and deferrals

The ADR-0022, ADR-0036, ADR-0037, and ADR-0038 APIs and behavior remain unchanged and independently callable. This composition adds no concrete tokenizer algorithm, provider, inference, selection, authorization, availability, routing, automatic local-first policy, fallback, retry, repair, truncation, rewriting, summarization, provider-usage reconciliation, semantic validation, general privacy policy, networking, endpoint, credential, filesystem, configuration, async/streaming, telemetry, persistence, or new dependency. NEXA-TUTOR-001 remains Baseline Draft, the known ingestion/context-assembly checklist inconsistency is preserved, and Phase 4 remains in progress.
