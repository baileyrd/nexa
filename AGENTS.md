# Nexa agent guide

## Authority and preservation

Always begin with [`docs/PROJECT-STATUS.md`](docs/PROJECT-STATUS.md), then follow [`docs/BASELINE.md`](docs/BASELINE.md) and [`docs/SPECIFICATION-REGISTRY.md`](docs/SPECIFICATION-REGISTRY.md).

The current v1 parents are:

- [`docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
- [`docs/architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md`](docs/architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md)
- [`docs/adr/0069-owner-approved-v1-delivery-baseline.md`](docs/adr/0069-owner-approved-v1-delivery-baseline.md)
- [`docs/architecture/IMPLEMENTATION-ROADMAP.md`](docs/architecture/IMPLEMENTATION-ROADMAP.md)
- the applicable subsystem specifications and accepted ADRs.

ADR-0068 is preserved historical authority and remains applicable only where ADR-0069 does not conflict. `NEXA-ARCH-001` and reconstructed documents are provenance/long-range context, not v1 selection authority. Report conflicts; never silently reconcile them.

## Program integrity and current gate

Use `Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`. Existing contract/headless evidence retains its factual maturity; do not infer system, user, or release maturity.

The owner-approved route is recorded in ADR-0069. General product implementation is under **Tactical Pause**. After this reconciliation is reviewed and merged, only the separately dispatched G1 shared React/TypeScript/Vite + Tauri 2 and versioned loopback HTTP/WebSocket suitability spike may receive **Continue**. These technologies, Sherpa-ONNX, and Rive remain candidates until their gates pass; spike evidence cannot silently select production architecture.

Every implementation increment must state its release blocker, governing authority, concrete E2E step, maturity before/after, and required evidence. The Chief Systems Architect records Continue, Redirect, or Tactical Pause whenever authority, deferrals, maturity, or the finite release route diverges.

## Owner-approved v1 route

One local learner uses identical Windows desktop and same-machine browser clients, backed by one local Rust runtime and authoritative SQLite state, to complete Networking Fundamentals / TCP Connection Establishment through LM Studio, bundled speech, and a synchronized animated 2D tutor. Both clients use one shared frontend and one versioned loopback HTTP/WebSocket business API; Tauri commands never form a second API.

Nexa bundles no LLM weights or inference runtime. LAN/remote access, hosted deployment, cloud sync, accounts/multi-user administration, labs/tools, broad model-server support, dynamic routing/fallback, dedicated vector infrastructure unless proven necessary, durable event brokerage, and 3D release integration are deferred.

## ChatGPT–Codex coordination

When development uses the human-coordinated or automated ChatGPT/Codex workflow, follow [`CHATGPT_WORKFLOW.md`](CHATGPT_WORKFLOW.md).

ChatGPT/Chief Systems Architect selects work from current repository authority and the release path, reviews exact PR heads, requests corrections on the existing PR branch, and merges only the exact reviewed green head.

Codex implements and validates bounded tasks; it does not independently redefine architecture, product scope, or authority status.

## Repository map and boundaries

- `crates/nexa-domain`, `nexa-events`, `nexa-nbp`: dependency-light shared contract kernel.
- `crates/nexa-student`, `nexa-pedagogy`, `nexa-lessons`, `nexa-assessment`: owned learning policies.
- `crates/nexa-learning-core`: atomic learning composition; it does not absorb owned reasoning.
- `crates/nexa-knowledge` and `nexa-knowledge-runtime`: governed knowledge contracts and async runtime boundary.
- `crates/nexa-tutor`: provider-neutral prompt/model/admission contracts; concrete LM Studio integration remains a narrow adapter.
- `crates/nexa-orchestrator` and `nexa-orchestrator-runtime`: workflow contracts and runtime task/cancellation ownership.
- `crates/nexa-storage`: the SQLite adapter boundary; database dependencies do not enter domain crates.
- `crates/nexa-speech`: provider-neutral speech foundations; the required bundled v1 adapter remains evidence-gated.
- `crates/nexa-avatar`: renderer-neutral semantic embodiment ports; required 2D integration remains evidence-gated.
- `crates/nexa-3d`: retained non-v1 renderer/runtime foundation.
- `crates/nexa-labs`: retained later-capability foundation, not a v1 gate.
- `apps/nexa-headless`: test/integration composition, not either released learner client.
- the shared frontend, desktop shell, and local runtime composition locations are selected only after G1 evidence and an authority update.
- `tools/`, `content/`, `assets/`, and `docs/`: validators, governed inputs, assets, and authority/evidence.

A `.gitkeep`-only boundary is planned, not implemented. Domain-facing crates remain independent of UI, OS, async runtime, databases, networking, renderers, and concrete providers. Tutor/model output is untrusted until admission and never selects renderer primitives or host authority.

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

As G4–G8 activate Windows/UI/storage/model, speech, and embodiment adapters, add focused concrete-adapter and Windows validation required by the owning R1 specifications. Passing the existing Linux/headless suite does not prove higher maturity.

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

---
