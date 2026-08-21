# ADR-0031: Deterministic caller-authorized available remote-model selection

- **Status:** Accepted
- **Date:** 2026-08-21
- **Scope:** Narrow Phase 4 non-invoking remote-authorization gate

## Context

ADR-0027 defines static eligibility and ordering, and ADR-0029 independently gates selection with
caller-supplied availability. Neither authorizes a remote model. A host needs a bounded,
provider-neutral way to prove its explicit authorization decision for the exact prompt that would
be sent, without yet transmitting that prompt or invoking a provider.

## Decision

`nexa-tutor` owns a closed V1 caller-supplied remote-authorization contract. It records supported
contract and policy versions, the exact ADR-0023 prompt-compilation replay anchor, and a bounded,
canonical allowlist of exact provider/model identities and their expected `ApprovedRemote` or
`RestrictedRemote` privacy class. Omission means unauthorized; an empty snapshot is valid and
authorizes nothing. Standalone decoding rejects unknown fields, unsupported versions, invalid or
non-lowercase SHA-256 anchors, local entries, duplicates, non-canonical order, and excessive
entries. The ordinary constructor canonicalizes input.

`select_authorized_available_remote_model` requires explicit, distinct, remote-only privacy
requirements. It intrinsically validates the exact prompt compilation and all its supported
versions, requires replay-anchor equality, validates every authorization identity and privacy
class against the registry, validates ADR-0029 availability, and selects from their intersection
using unchanged ADR-0027 capability, output, conservative byte-context, privacy-position, and
canonical provider/model rules. Authorization and availability are independent gates. The
original registered shared handle and descriptor are returned.

The operation is non-invoking. It never calls a provider, consumes scripted state, or transmits
data. Authorization is caller-supplied policy evidence, not proof of authenticity or freshness.
It does not filter, redact, minimize, or semantically inspect context and does not claim that any
such processing occurred. It does not authorize tools or other provider capabilities.

## Preserved boundaries and deferrals

This decision does not implement local-first routing, fallback, recovery, retry, repair,
regeneration, or remote execution. ADR-0025 and ADRs 0028–0030 retain their existing APIs and
meanings. No provider, endpoint, credential, networking, inference, tokenizer, health probe,
clock, monitoring, semantic-safety system, persistence, async/streaming execution, telemetry, or
tool execution is introduced.

Context privacy filtering/redaction/minimization and sensitivity inference remain deferred, as do
concrete remote invocation, authenticity/freshness policy, automatic routing, fallback, and
`partial truncation`. NEXA-TUTOR-001 remains Baseline Draft and Phase 4 remains in progress.
