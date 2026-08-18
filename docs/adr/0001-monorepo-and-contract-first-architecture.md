# ADR-0001: Monorepo and contract-first architecture

- Status: Accepted
- Date: 2026-08-18
- Decision owners: Nexa project
- Related: NEXA-DOM-001, NEXA-EVT-001, NEXA-NBP-001, NEXA-ORCH-001

## Context

Nexa began with a working 3D validation runtime while the wider tutor architecture existed primarily in design material. The reconstructed baseline defines many cooperating Rust subsystems, applications, tools, content packages, and assets. Leaving the repository organized as a single 3D package misrepresents the product and encourages contracts to drift.

## Decision

Nexa will use a Rust-centered monorepo with these boundaries:

- `apps/`: user-facing applications and composition roots
- `crates/`: reusable domain and subsystem implementations
- `tools/`: compilers, validators, and production tooling
- `content/`: governed courses, lessons, assessments, labs, and knowledge
- `assets/`: runtime avatar, scene, and speech assets
- `docs/`: specifications, ADRs, architecture, governance, and provenance

Implementation is contract-first. `nexa-domain`, `nexa-events`, and `nexa-nbp` form the initial shared contract layer. Higher-level crates depend inward on stable contracts and communicate across subsystem boundaries through typed commands, queries, events, and ports.

The existing root `nexa-3d-runtime` remains intact temporarily. It will move only in a dedicated migration that preserves history, binary behavior, tests, and CI.

## Consequences

Positive:

- The repository represents the whole product.
- Shared types and events gain clear ownership.
- Provider and renderer implementations remain replaceable.
- Specifications can map directly to crates and conformance tests.
- Applications compose capabilities without becoming domain owners.

Costs:

- Workspace dependency direction must be governed.
- Cross-cutting changes require coordinated versioning and tests.
- The initial 3D package needs a careful staged migration.
- Empty planned crates must not be mistaken for implemented capabilities.

## Guardrails

- No crate may directly manipulate another subsystem's persistence.
- Domain crates do not depend on UI, renderer, model-provider, or database implementations.
- The tutor layer emits semantic behavior; avatar adapters resolve it into physical presentation.
- New cross-cutting decisions require an ADR.
- A directory or placeholder does not constitute implementation.
