# Contributing to Nexa

Nexa is developed as a contract-first platform. A change is complete only when code, tests, specifications, and traceability agree.

## Workflow

1. Start from an issue, specification requirement, ADR, or clearly stated maintenance objective.
2. Use a focused branch and pull request.
3. Identify affected specifications and subsystem boundaries.
4. Add or update tests before promoting a contract.
5. Record cross-cutting architectural decisions as ADRs.
6. Update the specification registry when authority or status changes.

## Change categories

- Editorial: formatting or clarity with no semantic change
- Contract: serialized types, events, protocols, schemas, or public ports
- Implementation: behavior behind an existing contract
- Architecture: boundaries, dependency direction, deployment, or cross-cutting policy
- Content/asset: governed learning content or runtime media

Contract and architecture changes require explicit impact notes in the pull request.

## Rust quality gates

The intended workspace gates are:

```text
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Asset validation remains headless in CI. GPU presentation tests supplement but do not replace deterministic validation.

## Dependency direction

Domain and protocol crates remain independent of applications, UI frameworks, renderers, databases, model providers, and operating-system integrations. Integrations implement ports owned by the domain-facing layer.

## Documentation

Use stable specification IDs. Do not silently rewrite reconstructed source material or resolve conflicts by deletion. Mark superseded documents and point to their replacements.
