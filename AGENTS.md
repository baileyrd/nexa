# Nexa agent guide

## Authority and preservation

Always begin with [`docs/PROJECT-STATUS.md`](docs/PROJECT-STATUS.md), then follow the authority route in [`docs/BASELINE.md`](docs/BASELINE.md) and [`docs/SPECIFICATION-REGISTRY.md`](docs/SPECIFICATION-REGISTRY.md).

For v1 work, the current parent authorities are:

- [`docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
- [`docs/architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md`](docs/architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md)
- [`docs/adr/0068-v1-r2-walking-skeleton-baseline.md`](docs/adr/0068-v1-r2-walking-skeleton-baseline.md)
- [`docs/architecture/IMPLEMENTATION-ROADMAP.md`](docs/architecture/IMPLEMENTATION-ROADMAP.md)
- the applicable subsystem specifications and accepted ADRs.

`NEXA-ARCH-001` and reconstructed documents are preserved provenance/long-range design context. They do not override `NEXA-ARCH-002` for v1 implementation selection.

Do not silently reconcile a conflict between documents or between documentation and implementation. Stop at the boundary, report the conflict, and resolve it through the owning architecture/specification/ADR before implementation continues.

## Program-integrity rule

Local correctness is necessary but not sufficient.

Every implementation increment must state:

1. the current release/E2E blocker it addresses;
2. the governing parent architecture/specification/ADR;
3. the E2E step it makes concrete;
4. the capability maturity state before/after;
5. the evidence required to support that maturity change.

Use the maturity vocabulary:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Do not use an unqualified `Complete` when it would hide the actual maturity demonstrated.

The Chief Systems Architect must call Continue, Redirect, or Tactical Pause when parent documentation trails implementation, inherited deferrals cross required gates, repeated horizontal work does not advance the vertical release path, authority/status documents disagree, or the roadmap no longer presents a finite credible route to release.

## Current R2 scope

After the rebaseline PR is green and merged, R2 is the only normal product-development stage.

The R2 walking skeleton is text-first:

```text
learner text
 -> apps/nexa-desktop
 -> orchestrator
 -> SQLite durable state
 -> learning/pedagogy
 -> governed TCP course knowledge
 -> local llama.cpp model adapter
 -> admitted tutor response
 -> assessment/practice
 -> atomic durable progress
 -> restart/resume
```

R2 concrete baseline:

- `apps/nexa-desktop` learner composition root;
- `eframe`/`egui` after bounded suitability spike;
- SQLite/`rusqlite` behind `crates/nexa-storage`;
- local `llama.cpp` server through a narrow provider adapter;
- Windows x86_64 first acceptance environment;
- Networking Fundamentals / TCP Connection Establishment first course.

Do not restart the superseded open-ended Phase 5 sequence.

Speech, avatar/behavior embodiment, labs/tools, dynamic multi-provider routing/fallback, dedicated vector infrastructure, and durable event broker work are not R2 exit criteria unless an actual R2 blocker proves otherwise.

## ChatGPT–Codex coordination

When development uses the human-coordinated or automated ChatGPT/Codex workflow, follow [`CHATGPT_WORKFLOW.md`](CHATGPT_WORKFLOW.md).

ChatGPT/Chief Systems Architect selects work from current repository authority and the release path, reviews exact PR heads, requests corrections on the existing PR branch, and merges only the exact reviewed green head.

Codex implements and validates bounded tasks; it does not independently redefine architecture, product scope, or authority status.

## Repository map and boundaries

- `crates/nexa-domain`, `nexa-events`, `nexa-nbp`: shared contract kernel. `nexa-domain` remains the dependency-light leaf.
- `crates/nexa-student`, `nexa-pedagogy`, `nexa-lessons`, `nexa-assessment`: owned learning policies.
- `crates/nexa-learning-core`: atomic learning composition; it does not absorb owned subsystem reasoning.
- `crates/nexa-knowledge`: governed ingestion/retrieval/context/citation contracts.
- `crates/nexa-knowledge-runtime`: runtime-owned async knowledge service boundaries.
- `crates/nexa-tutor`: provider-neutral prompt/model/admission/tutor contracts.
- `crates/nexa-orchestrator`: dependency-light session/workflow contracts.
- `crates/nexa-orchestrator-runtime`: Tokio-backed workflow task/cancellation ownership.
- `crates/nexa-storage`: approved R2 concrete persistence adapter boundary; database/runtime dependencies belong here, not in domain crates.
- `crates/nexa-avatar`: renderer-neutral embodiment ports.
- `crates/nexa-3d`: 3D runtime/adapter; GPU/window composition belongs in `apps/nexa-3d-viewer`.
- `crates/nexa-speech`, `crates/nexa-labs`: retained later-capability foundations; not R2 critical path by default.
- `apps/nexa-desktop`: approved R2 learner-facing composition root.
- `apps/nexa-headless`: test/integration composition, not the released learner application.
- `tools/`: compilers/validators.
- `content/` and `assets/`: governed inputs/assets.
- `docs/`: specifications, ADRs, architecture, governance, traceability, and provenance.

A `.gitkeep`-only boundary is planned, not implemented.

Keep domain-facing crates independent of UI, renderer, OS, async runtime, storage/database, networking, and concrete model-provider implementations. Concrete integrations implement ports owned by domain-facing layers.

Tutor/model output is untrusted semantic content until admission; it must not select renderer primitives or host authority.

## Required validation

Run from the repository root for ordinary Rust implementation PRs:

```text
cargo fmt --all --check
cargo check --workspace --all-targets
cargo check -p nexa-3d --no-default-features
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-contract-boundaries.sh
git diff --check
```

As R2 activates Windows/UI/storage/model adapters, add focused concrete-adapter and Windows validation required by the owning R1 specifications. Passing the existing Linux/headless suite does not prove higher maturity.

## Change and review rules

- Make bounded, independently reviewable changes; do not bundle unrelated cleanup or speculative architecture.
- Prefer a vertical maturity advance over another horizontal abstraction when both are possible.
- Contract/architecture changes require explicit authority/impact notes; consequential cross-cutting decisions require an ADR; authority/status changes require a registry update.
- Review wire changes for stable names, validation, compatibility, replay, and deterministic fixtures.
- Review dependency changes against the inward DAG and renderer/provider/storage boundaries.
- Review evidence/state changes for ownership, immutability/idempotency, policy versions, concurrency, and atomic failure behavior.
- A PR is done only when implementation, tests, governing docs, status/registry/traceability, and applicable ADRs agree; all required checks pass on the exact reviewed head; the diff is focused; and deferrals/conflicts are explicit.
- User-facing work requires proportionate accessibility checks.
- Security/privacy review must match the actual changed trust/data boundary.
- Do not claim `System Verified`, `User Accepted`, or `Release Ready` from unit/contract/headless evidence.
