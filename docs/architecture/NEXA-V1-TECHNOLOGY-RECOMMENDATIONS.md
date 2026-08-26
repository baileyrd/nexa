# Nexa v1 Technology Recommendations

Status: Architecture recommendation; requires owning ADRs/spec approval before implementation

Research checkpoint: 2026-08-26

## Purpose

Translate the v1 architecture requirements into a deliberately small concrete technology set for the R2 walking skeleton. These recommendations optimize for local-first operation, Rust integration, low operational complexity, existing Nexa architecture, and the shortest path to one real learner journey.

They are recommendations, not silent technology decisions. Each selected technology must be approved by the owning specification/ADR and validated against release requirements.

## Recommended R2/R3 technology spine

```text
Desktop UI        eframe / egui
        |
Rust application composition + Tokio orchestration
        |
        +-- Learning/domain crates (existing)
        +-- SQLite persistence via rusqlite
        +-- Knowledge retrieval over durable SQLite-backed corpus
        +-- Local llama.cpp server adapter over bounded HTTP API
        +-- Content-safe structured logs/tracing
```

## 1. Durable state — SQLite via rusqlite

### Recommendation

Use SQLite as the first v1 embedded durable store, accessed through a new infrastructure adapter using `rusqlite` with bundled SQLite for release-controlled database files.

### Rationale

- Embedded/local-first: no server process is required for learner state.
- Mature ACID transaction semantics fit the existing atomic learning-core UoW requirements.
- A Rust adapter can remain outside domain crates and implement existing persistence ports.
- One local database can support learner state, course/package metadata, knowledge provenance, and bounded lexical indexes without merging domain ownership.
- SQLite gives us a credible backup/export/migration story without introducing a production service.
- For the initial course corpus, vector computation can remain in Rust or use persisted embeddings without requiring a dedicated vector database until measured evidence justifies one.

### Implementation boundary

Create a concrete infrastructure/storage crate only after the data specification and storage ADR are approved. Domain-facing crates do not depend on `rusqlite`.

Likely ownership:

- `crates/nexa-storage` becomes the infrastructure adapter boundary.
- It may implement repository/UoW traits owned by learning, knowledge, and related domains.
- Schema/migrations live with the storage adapter/application release, not inside domain policy crates.

### Important constraints

- Use explicit transactions for the learning-core atomic commit.
- Use version/concurrency checks; do not rely on last-write-wins.
- Define schema migrations before persisted data enters release acceptance.
- Do not let SQLite row IDs replace canonical Nexa IDs.
- Do not store secrets in the normal SQLite domain database.

### Evidence behind recommendation

Current `rusqlite` documentation provides explicit transactions that roll back by default unless committed. The project documentation recommends the bundled SQLite feature for applications that control their own database. `rusqlite_migration` supports atomic ordered migrations. These characteristics align closely with Nexa v1's data requirements.

## 2. Learner desktop UI — eframe / egui

### Recommendation

Use `eframe`/`egui` for the first learner-facing `apps/nexa-desktop` implementation unless a focused UX spike disproves suitability.

### Rationale

- Rust-native application path keeps the first product composition in the existing Rust workspace.
- `eframe` supports native desktop applications and uses `egui-winit`/wgpu by default, conceptually compatible with Nexa's existing winit/wgpu experience.
- Native accessibility integration is available through AccessKit on supported platforms.
- The immediate-mode model is sufficient for the deliberately small v1 course/tutor/progress UX.
- Avoids adding a browser/JavaScript application stack before the learner workflow is proven.
- Can later coexist with or host custom wgpu rendering if the avatar is promoted into the released application.

### Constraints

- The desktop app remains a composition/presentation boundary; it must not absorb learning/tutor/orchestrator logic.
- Long-running async work must run outside the UI frame callback and report state back safely.
- Accessibility must be validated in the actual selected platform/backend rather than assumed from framework support.
- Existing `wgpu` version compatibility must be assessed; do not force a workspace-wide GPU upgrade merely to start the text-first UI if a separate app dependency strategy is safer.

### Spike gate

Before final ADR approval, implement a short architecture spike proving:

- async tutor operation without blocking UI;
- course/tutor/assessment screens;
- keyboard/accessibility tree basics;
- content-safe error/status display;
- packaging on the selected first OS.

The spike is architecture validation, not the start of feature development.

## 3. First concrete model path — local llama.cpp server

### Recommendation

Use a local `llama.cpp` server as the first concrete model execution path for R2, accessed through a narrow Nexa HTTP adapter implementing the existing provider-neutral model contract.

### Rationale

- Directly aligns with Nexa's local-first/offline-capable architecture goal.
- `llama.cpp` provides a local HTTP server with OpenAI-compatible chat/completion endpoints.
- It supports a broad range of hardware backends and quantized GGUF models.
- Server/process isolation avoids embedding a C/C++ FFI dependency into the first Rust walking skeleton.
- The adapter can be tested independently and later coexist with a remote provider adapter without changing tutor/domain contracts.
- Schema-constrained JSON support is potentially useful for Nexa's structured model-output requirements, while Nexa still retains its own strict admission as authority.

### Critical compatibility rule

Treat llama.cpp as a **specific adapter**, not as proof that all OpenAI-compatible servers behave identically. The adapter must implement only the request/response subset Nexa actually validates and must test server/version behavior directly.

### Model selection

Do not hard-code a model family into architecture. Select the first release GGUF model through the R4 tutor-quality evaluation based on:

- instruction following;
- structured-output reliability;
- context capacity;
- grounding/citation quality;
- latency/resource budgets on target hardware;
- license/distribution requirements.

The walking skeleton may begin with a known-good development model before the release model is selected.

### R8 packaging question

R2 may initially treat `llama-server` and a model file as an explicitly installed/configured local dependency. Before Release Ready, R8 must decide whether Nexa:

- bundles the server binary;
- downloads/installs it through a governed mechanism;
- requires a separately installed supported runtime;
- supports a remote provider as an alternative release configuration.

Do not confuse R2 integration convenience with final distribution policy.

## 4. HTTP/runtime adapter

### Recommendation

Add provider networking only in a concrete adapter/runtime crate or application infrastructure layer. Domain-facing `nexa-tutor` remains free of networking/runtime dependencies.

Use Tokio-backed asynchronous orchestration already established in the runtime layer. Select a mature Rust HTTP client during implementation ADR work and configure:

- strict endpoint policy;
- timeout;
- request/response size bounds;
- connection/TLS behavior appropriate to local versus remote endpoints;
- no automatic redirect/fallback behavior that violates provider/security policy.

For local llama.cpp, bind to loopback by default.

## 5. Knowledge storage/retrieval — start simple

### Recommendation

Use the same SQLite-backed infrastructure for governed source/chunk/provenance metadata and first-course lexical indexing, while retaining the existing in-Rust deterministic retrieval contracts/algorithms where practical.

For vector retrieval:

- persist embeddings in the simplest validated form compatible with the approved data schema;
- perform current deterministic vector ranking in Rust for the initial bounded corpus if it meets R7 budgets;
- adopt a vector extension/database only if R4/R7 evidence shows it is necessary.

This avoids adding a vector service merely because the architecture supports vector retrieval.

## 6. First supported platform — recommended Windows x86_64

### Recommendation

Use Windows x86_64 as the first release-supported desktop target, while preserving portable Rust boundaries and Linux CI where useful.

### Rationale

A first release needs one explicit acceptance environment. The selected platform should match the primary development/acceptance environment and allow focused validation of credential storage, installer behavior, accessibility, wgpu/UI behavior, and local model acceleration before claiming broader support.

Cross-platform support remains an architecture goal; it is not proven by cross-platform crates alone.

### CI consequence

The current CI runs only on `ubuntu-latest`. Before v1 concrete UI/storage/model work can reach System Verified maturity, add Windows CI for release-critical build/test paths. Keep Linux CI for headless portability/boundary evidence where applicable.

## 7. Release packaging — cargo-dist candidate, MSI decision later

### Recommendation

Evaluate `cargo-dist` as the release-artifact orchestration candidate after the learner app exists. It currently supports building cross-platform Rust release artifacts/installers and artifact attestations.

Do not adopt an MSI-specific tool until the repo Rust-version/tooling compatibility is checked. Current `cargo-wix` documentation requires a newer Rust toolchain than Nexa's current `rust-version = 1.85`, so using it directly today would introduce a toolchain decision that belongs in R8 rather than R2.

A signed/attested portable Windows package may be a valid earlier R2/R3 delivery artifact; the final v1 installer format remains an R8 decision.

## 8. Event/runtime recommendation

Do **not** introduce a durable event broker for R2.

Use:

- direct orchestrator calls/returns for the primary synchronous/async workflow;
- SQLite atomic transactions for authoritative local state;
- existing typed events where they provide useful domain facts;
- content-safe observability for operations.

Promote durable event/outbox infrastructure only if a concrete v1 asynchronous consumer proves it is required for correctness.

## 9. Conditional capability recommendation

For the shortest credible v1 path:

- **Text tutor:** required.
- **Durable learning state:** required.
- **Real model:** required.
- **Grounded knowledge:** required.
- **Speech:** defer from the R2 walking skeleton; reassess at R5.
- **Animated avatar:** preserve existing foundation; defer from R2; reassess at R5 as a product-identity enhancement.
- **Labs/tools:** post-v1 by default unless the first released course specifically requires a lab for its acceptance outcome.
- **Dynamic multi-provider routing:** post-v1.

This is scope control, not abandonment of those capabilities.

## 10. First course recommendation

Use a very small Networking Fundamentals acceptance course centered on TCP connection establishment as the first governed content package.

Rationale:

- the reconstructed Nexa architecture already uses Networking Fundamentals/TCP examples to illustrate learner context, pedagogy, competency, and tutor behavior;
- the topic supports clear source-grounded factual questions, ordered concepts, common misconceptions, and deterministic assessment items;
- it can demonstrate adaptation without requiring labs, speech, or advanced multimedia;
- it provides a meaningful but bounded first end-to-end lesson.

Proposed first-course shape:

- Course: `Networking Fundamentals`
- Module: `Transport Layer Basics`
- Lesson: `TCP Connection Establishment`
- Core objectives: purpose of TCP connection establishment; SYN/SYN-ACK/ACK ordering; contrast with UDP at a basic level
- Assessment: bounded deterministic question types already supported by the assessment slice
- Knowledge corpus: small governed set sufficient to answer/evaluate the lesson

Exact authored content remains a separate content-design/release artifact and must pass provenance/quality review.

## 11. Decisions to formalize through ADRs

If these recommendations survive architecture review, create focused ADRs for:

1. v1 embedded persistence technology and migration approach;
2. v1 learner UI framework/application composition;
3. v1 concrete local model adapter/runtime boundary;
4. v1 first supported platform and release packaging approach;
5. v1 first-course/content-package scope if this is treated as an architectural/product decision rather than only release planning.

Do not implement from this recommendation document alone.

## 12. Evidence sources consulted

Current upstream project documentation was reviewed for:

- `rusqlite` transaction and bundled SQLite behavior;
- `rusqlite_migration` atomic migrations;
- `eframe`/`egui` native wgpu and AccessKit capabilities;
- `llama.cpp` local server/OpenAI-compatible API and hardware support;
- `cargo-dist` current release-artifact/attestation capabilities;
- `cargo-wix` current toolchain requirements.

These upstream capabilities can change; owning ADRs should record exact versions tested when implementation begins.
