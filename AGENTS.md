# Nexa agent guide

## Authority and preservation

Work from a requirement, accepted ADR, approved/baseline specification, or focused maintenance objective. For repository design questions, follow the interpretation order in [`docs/BASELINE.md`](docs/BASELINE.md): accepted ADRs and approved specifications; registry-listed Baseline Draft specifications; NEXA-CBS-001; canonical visual references; verified runtime contracts; architecture narratives; then provenance. The [`specification registry`](docs/SPECIFICATION-REGISTRY.md) is the navigation/status authority, and the [`implementation roadmap`](docs/architecture/IMPLEMENTATION-ROADMAP.md) and phase traceability matrices record delivery gates and evidence.

Do not silently reconcile a conflict between documents or between documentation and implementation: report it and obtain review. Preserve reconstructed documents as source evidence. Do not rewrite, delete, or supersede their meaning without traceability through review, an ADR, or specification history.

## ChatGPT–Codex coordination

When development uses the human-coordinated ChatGPT and Codex workflow, follow [`CHATGPT_WORKFLOW.md`](CHATGPT_WORKFLOW.md).

`CHATGPT_WORKFLOW.md` governs task handoffs, pull-request review, correction cycles, and workflow trigger messages. It does not supersede the repository authorities, engineering constraints, validation requirements, or completion rules in this file.

Do not duplicate the complete workflow elsewhere in `AGENTS.md`.

## Repository map and boundaries

- `crates/nexa-domain`, `nexa-events`, and `nexa-nbp` are the shared contract kernel. `nexa-domain` is the dependency-light leaf; events and NBP may depend on it, never on each other.
- `crates/nexa-avatar` owns renderer-neutral embodiment ports; `crates/nexa-3d` implements the headless 3D runtime/adapter. GPU, window, and OS-input composition belongs in `apps/nexa-3d-viewer`; `tools/nexa-3d-validate` stays headless.
- `crates/nexa-student`, `nexa-pedagogy`, `nexa-lessons`, and `nexa-assessment` own their Phase 3 policies; `nexa-learning-core` composes them atomically without absorbing their reasoning.
- `crates/nexa-knowledge` owns governed knowledge/retrieval/context/citation contracts. `crates/nexa-tutor` owns provider-neutral response planning and consumes knowledge by reference.
- `apps/` contains composition roots; `tools/` contains compilers/validators; `content/` and `assets/` contain governed inputs; `docs/` contains specifications, ADRs, architecture, governance, and provenance. A `.gitkeep`-only boundary is planned, not implemented.

Keep domain-facing crates independent of UI, renderer, OS, async runtime, storage/database, networking, and model-provider implementations. Integrations implement ports owned by the domain-facing layer; no subsystem manipulates another subsystem's persistence. Tutor output is semantic intent: LLM/tutor code must not select clips, bones, blendshape weights, or renderer commands. See [ADR-0001](docs/adr/0001-monorepo-and-contract-first-architecture.md), [ADR-0002](docs/adr/0002-contract-kernel-dependency-boundaries.md), and the accepted ADRs indexed by the registry rather than duplicating their decisions here.

## Required validation

Run from the repository root:

```text
cargo fmt --all --check
cargo check --workspace --all-targets
cargo check -p nexa-3d --no-default-features
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-contract-boundaries.sh
git diff --check
```

## Change and review rules

- Make bounded, independently reviewable changes; do not combine unrelated cleanup or speculative architecture.
- Identify affected specifications and boundaries. Contract/architecture changes require explicit PR impact notes; cross-cutting decisions require an ADR; authority/status changes require a registry update.
- Review wire changes for stable names, validation, compatibility, replay, and deterministic fixtures. Review dependencies against the enforced inward DAG and renderer/provider/storage exclusions. Review evidence/state changes for subsystem ownership, immutability/idempotency, explicit policy versions, and atomic failure behavior where the accepted ADRs require them.
- A PR is done only when implementation, tests, specifications, registry/traceability, and affected ADRs agree; all applicable commands above pass; the diff is focused; and deferrals or conflicts are explicit. User-facing work also requires proportionate accessibility checks, and threat/privacy review must match the scope.
