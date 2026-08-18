# Nexa Reconstructed Baseline

## Decision

The documents added in commit `00cd04d` are the working source of truth for the complete Nexa tutor platform. The earlier conversation is provenance, not an independently enforceable specification.

## Interpretation order

When sources disagree, use this order until a later ADR changes it:

1. Approved ADRs and approved specifications
2. Baseline Draft specifications listed in the registry
3. NEXA-CBS-001 for character identity and semantic behavior principles
4. Canonical visual reference sheets for visual identity
5. Implemented and tested runtime contracts
6. Architecture narrative documents
7. Conversation exports and other provenance

Conflicts between a higher-level design and verified implementation are not silently resolved. They are recorded and reviewed.

## Preservation policy

Reconstructed documents remain intact while they are audited. Formatting corrections may be made independently, but semantic edits require traceability through review, an ADR, or specification history.

## Immediate invariants

- Nexa is a complete adaptive tutor platform, not merely a 3D avatar.
- The avatar is an embodiment adapter behind semantic behavior contracts.
- The LLM produces structured communicative intent and never controls animation primitives directly.
- Core domain types and events are shared contracts rather than redefined per subsystem.
- Student mastery changes only from governed evidence and knowledge-tracing rules.
- The orchestrator coordinates subsystem work but does not absorb subsystem reasoning.
- Renderer, model provider, speech provider, storage, and lab backends remain replaceable.
- Local-first and offline-capable operation remain architectural goals where practical.

See [SPECIFICATION-REGISTRY.md](SPECIFICATION-REGISTRY.md) for the governed inventory.
