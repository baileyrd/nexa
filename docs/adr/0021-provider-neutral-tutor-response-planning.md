# ADR-0021: Provider-neutral tutor response planning

Status: Accepted

## Context

NEXA-TUTOR-001 describes a larger generative tutor while ADRs 0019–0020 establish knowledge-owned governed context and citation results. This Phase 4 slice needs an auditable boundary without prematurely selecting the specification's deferred provider, semantic validation, or repair behavior. The specifications do not yet define a canonical cross-crate pedagogy decision identifier or a deterministic semantic-safety algorithm.

## Decision

`nexa-tutor` owns synchronous response requests, closed section/capability/safety vocabularies, deterministic planning, and standalone response validation. `nexa-knowledge` continues to own context and citation semantics; tutor only verifies and copies their identities and ordered claim/citation positions. Canonical response, section, and interaction identities live in `nexa-domain` because audit, replay, and external addressing require them.

Text is supplied by the caller and remains inert: the planner neither creates, repairs, rewrites, nor interprets it. Student and pedagogy decisions enter through a minimal reference-only `DecisionEvidence` contract using existing evidence and scope identities. It records policy versions, allowed kinds, scaffolding bounds, and assessment restrictions without inferring mastery, intent, emotion, or strategy. Lessons and assessment retain ownership of lesson scope and protected material.

Assessment protection takes precedence over ordinary pedagogy. Contradictory evidence fails closed; protected output is limited to hints, checks, constrained responses, or refusal. Structurally required refusal/constrained classifications must use their matching section kinds. The planner preserves input order and assigns consecutive positions. Results are accepted, constrained, or refused with closed rationale codes; invalid input returns a redacted error and no package.

A SHA-256 replay anchor covers scope, governance/policy versions, exact limits, capabilities, evidence, status, rationale, ordered identities/kinds, content, and citation bindings. Thus standalone deserialization detects coordinated reorder/renumber, reassociation, downgrade, or status tampering. Content-bearing Debug implementations redact text.

## Consequences and deferrals

The contract proves reference consistency, not truth or entailment. Semantic safety classification remains caller evidence because no authoritative deterministic rule resolves it. LLM calls, prose generation, model/provider selection, repair/regeneration, learner-state inference, tools, networking, async, databases, vector stores, persistence, and durable adapters remain deferred. This does not complete tutor intelligence or Phase 4.
