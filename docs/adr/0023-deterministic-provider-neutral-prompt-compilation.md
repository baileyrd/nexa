# ADR-0023: Deterministic provider-neutral prompt compilation

- **Status:** Accepted
- **Date:** 2026-08-20
- **Scope:** Narrow Phase 4 prompt-compilation boundary

## Context

NEXA-TUTOR-001 requires stable, versioned prompt layers while leaving their exact wire and framing contracts unresolved. ADR-0022 deliberately deferred compilation. A reviewed contract is required so untrusted learner, retrieval, conversation, lab, and tool material cannot acquire structural authority merely by resembling instructions or delimiters.

## Decision

`nexa-tutor` owns the dependency-light compiler that produces ADR-0022 `ModelInput`; upstream subsystems remain authoritative for learner state, pedagogy, curriculum, knowledge governance, policy, and tool permission.

V1 has this closed canonical order:

1. `platform_contract` — required, authoritative instruction;
2. `nexa_identity` — required, authoritative instruction derived from NEXA-CBS-001;
3. `policy` — required, governed evidence;
4. `pedagogy` — required, governed evidence;
5. `learner_context` — optional, trusted structured context;
6. `curriculum_lesson_context` — optional, trusted structured context;
7. `governed_knowledge_context` — optional, untrusted data after knowledge governance and caller selection;
8. `conversation_context` — optional, untrusted data;
9. `student_input` — required, untrusted data;
10. `permitted_tool_context` — optional, untrusted data describing already-permitted tools, not authorization;
11. `output_contract` — required, authoritative instruction.

Classification is derived from the closed kind and is nevertheless carried on the wire for audit; a mismatch fails. Caller order is ignored, duplicates fail, and missing required layers fail. Present content is non-empty and byte bounded. V1 binds compilation contract `1.0`, prompt-package `1.0`, context-builder `1.0`, output-schema `1.0`, the caller's per-layer/compiled limits, every classification, content length, content byte, and canonical position.

The canonical representation is compact UTF-8 JSON emitted from fixed-order Rust structures with unknown fields rejected. It includes a fixed framing identifier and explicit positions and UTF-8 byte lengths; JSON escaping makes content data rather than framing, preserves the original content bytes after decoding, and cannot be confused by delimiter-like content. Compilation uses checked arithmetic, validates content before cloning it into the envelope, rejects a compiled size over the request limit or ADR-0022 hard limit, and then constructs `ModelInput`. ADR-0022 `ModelRequest::validate_for` remains the final authoritative conservative byte-context and descriptor capability check.

The result carries the `ModelInput`, bound versions and limits, an ordered content-free manifest, exact compiled byte count, and lowercase SHA-256 of the complete canonical input. Because the input covers all governed fields, its digest is the replay/integrity anchor. Standalone result deserialization parses and validates the input, recompiles it, and exactly compares input, manifest, count, and anchor, rejecting tampering, reassociation, unsupported versions, or reordering. Prompt content is redacted from `Debug` and errors contain only closed categories.

This is structural separation and replay evidence, not semantic prompt-injection resistance. Successful compilation proves neither model-output correctness nor acceptance under ADR-0021; raw output remains untrusted.

## Consequences and deferrals

No new dependency or cross-crate identifier is needed. The compiler does not retrieve or assemble context, infer mastery, select pedagogy, authorize tools, choose or invoke a provider, or consume model output.

Deferred are authored prompt packages and filesystem loading; actual inference and provider integration; routing, fallback, cost/availability policy, and provider tokenization; output decoding/admission and connection to ADR-0021 planning; semantic safety, grounding, citation entailment, hallucination or prompt-injection detection; repair/regeneration; tool proposal/execution; async/streaming/cancellation/retry; networking, telemetry, and persistence.
