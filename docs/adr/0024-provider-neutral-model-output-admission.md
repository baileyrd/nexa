# ADR-0024: Provider-neutral model-output admission

- **Status:** Accepted
- **Date:** 2026-08-20
- **Scope:** Narrow Phase 4 bridge from ADR-0022 output to ADR-0021 planning

## Context

ADR-0022 deliberately treats `RawModelOutput` as untrusted bytes, and ADR-0023 stops at a
compiled `ModelInput`. NEXA-TUTOR-001 is still a Baseline Draft and does not approve an exact
output-admission wire contract. A narrow decision is therefore required before generated
candidate content can enter the accepted ADR-0021 planner.

## Decision

`nexa-tutor` owns output admission. A trusted caller supplies `TrustedPlanningAuthority`, which
contains every authoritative `PlanningRequest` field: contract and policy versions, response and
interaction identities, learning scope, knowledge/citation provenance identities, limits,
permitted capabilities, and decision evidence. The model owns only candidate sections. Admission
does not infer student state or pedagogy, authorize capabilities, retrieve knowledge, reinterpret
citations, or invoke a provider.

The closed V1 JSON schema is exactly an object with `candidate_schema_version: "1.0"` and a
non-empty `sections` array of existing closed `SectionRequest` values. Unknown fields at every
exposed structure are rejected. Consequently raw output cannot introduce identities, versions,
limits, evidence, usage, actions, tools, or renderer commands. It is never decoded as a complete
`PlanningRequest`.

Admission proceeds in this order: validate the descriptor; validate the request against that
descriptor; require structured output in both request and descriptor; intrinsically revalidate
the ADR-0023 compilation evidence; require its supported compilation, prompt-package,
context-builder, and output-schema versions; compare the request input byte-for-byte with the
compiled `ModelInput`; validate response identity against the request; reject every non-complete
finish; validate trusted planning authority; strictly decode one candidate object with no trailing
data; combine its sections with the trusted authority; and invoke the unchanged ADR-0021
`plan_response` with the exact `ContextPackage` and `CitationResult`.

`FinishReason::OutputLimit` always fails closed, even when the recorded prefix parses. V1 does not
repair, complete, retry, or regenerate output. The ADR-0022 raw-output bound limits allocation
before JSON decoding; section and response bounds remain those of ADR-0021.

Successful admission returns the validated `TutorResponse` and content-free evidence binding the
admission version; provider, model, and invocation identities; prompt compilation replay anchor;
prompt-package, context-builder, and candidate/output-schema versions; complete finish reason;
SHA-256 of the exact raw bytes; tutor-response replay anchor; and a SHA-256 admission replay anchor
over all those fields. Hashes are lowercase 64-character hexadecimal. Standalone deserialization
rejects unknown fields, invalid versions/hashes, an invalid nested response, response-anchor
disagreement, or an admission-anchor mismatch. A stored raw-output hash binds evidence to bytes
that can be supplied independently; it does not recreate omitted bytes or prove what they mean.

Diagnostics use closed error categories and never copy serde/provider messages or prompt, output,
section, learner, conversation, knowledge, or tool content. `Debug` uses the existing redacted
contracts. Categories distinguish version, descriptor/request, prompt association/replay,
response identity, structured-output capability, incomplete output, syntax, candidate schema,
planning provenance/evidence, policy/pedagogy/safety/capability, citation/reference, and internal
framing failures. Retryability and repairability are not decided.

## Guarantees and limitations

Success proves syntactic, schema, identity, provenance, policy-reference, capability,
citation-reference, and existing ADR-0021 structural consistency. It does **not** prove truth,
factual correctness, semantic safety, prompt-injection resistance, grounding entailment,
hallucination absence, or instructional quality. Tutor output remains semantic intent under
NEXA-CBS-001 and cannot control embodiment primitives.

## Deferred

- actual inference and concrete local/cloud providers;
- provider selection, routing, fallback, availability, cost policy, and tokenization;
- semantic safety, prompt-injection detection, factual correctness, grounding entailment, and
  hallucination control;
- response repair, completion, regeneration, and retry policy;
- tool proposal execution or authorization;
- async, streaming, cancellation, and timeout execution;
- networking, credentials, telemetry export, persistence, and durable adapters.

No dependency is added. NEXA-KNOW-001 retains context, citation, governance, and provenance
ownership; student, pedagogy, lesson, and assessment owners retain their authority. This increment
does not complete generative tutor intelligence, concrete provider integration, semantic safety,
or the Phase 4 exit gate.
