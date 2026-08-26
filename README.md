# Nexa

Nexa is a local-first adaptive AI training tutor platform. Its architecture combines governed tutor intelligence, pedagogy, student modeling, curriculum, assessment, knowledge retrieval, persistence, observability, and optional speech/avatar/lab adapters.

Nexa is not an avatar attached to a chatbot. The first v1 release is defined by a complete learner journey with durable progress and grounded instruction.

## Current development state

Nexa completed a tactical architecture/documentation rebaseline in August 2026 after identifying that strong local contract/conformance progress had outpaced parent architecture maturity and vertical product integration.

The current delivery path is **R0–R9**, not the older open-ended Phase 5 increment loop.

- R0 — governance/architecture rebaseline: complete for R2 once the rebaseline PR is merged.
- R1 — release-critical specification/technology baseline: complete for R2 once the rebaseline PR is merged.
- R2 — thin real production walking skeleton: next implementation stage.

See [`docs/PROJECT-STATUS.md`](docs/PROJECT-STATUS.md) for the exact current gate.

## Governing architecture

The v1 implementation authority is [`NEXA-ARCH-002`](docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md).

The reconstructed Tutor System Architecture remains preserved as historical/long-range design provenance, but it is superseded by NEXA-ARCH-002 for v1 implementation selection.

### Central rule

Tutor intelligence produces semantic instructional/communicative intent. It does not directly manipulate animation clips, bones, blendshape weights, renderer operations, host authorization, or other infrastructure authority.

```text
Learner
   |
   v
Nexa Desktop App
   |
   v
Session Orchestrator
   |-------------------|-------------------|
   v                   v                   v
Learning Core      Tutor / Knowledge   Observability
   |                   |
   |                   v
   |              Model Adapter
   |                   |
   |                   v
   |             local model runtime
   |
   +-------------> Durable Data
```

Speech, avatar/behavior embodiment, and labs/tools remain optional/later adapters outside the R2 critical path.

## R2 walking skeleton

ADR-0068 establishes the first concrete vertical path:

- learner app: `apps/nexa-desktop`;
- UI: `eframe`/`egui`, subject to a bounded suitability spike;
- persistence: SQLite through `rusqlite` behind `crates/nexa-storage`;
- model: local `llama.cpp` server behind a narrow Nexa adapter;
- first acceptance target: Windows x86_64;
- first course: Networking Fundamentals / TCP Connection Establishment;
- text-first interaction.

R2 must prove:

```text
learner text
 -> desktop UI
 -> orchestrator
 -> SQLite state
 -> learning/pedagogy
 -> governed knowledge
 -> real local model
 -> admitted tutor response
 -> assessment/practice
 -> atomic durable progress
 -> restart/resume
```

Scripted providers and in-memory storage remain useful test tools but cannot close the R2 system gate.

## Repository layout

| Path | Purpose |
|---|---|
| `apps/` | Application composition roots |
| `crates/` | Reusable domain, subsystem, runtime, and adapter boundaries |
| `tools/` | Validators/compilers and engineering tools |
| `content/` | Governed course, knowledge, assessment, and lab content |
| `assets/` | Avatar, scene, and speech assets |
| `docs/` | Architecture, specifications, ADRs, governance, traceability, and provenance |

Notable implemented foundations include:

- `nexa-domain`, `nexa-events`, `nexa-nbp` — canonical contract kernel;
- `nexa-student`, `nexa-pedagogy`, `nexa-lessons`, `nexa-assessment`, `nexa-learning-core` — deterministic learning policies/composition;
- `nexa-knowledge`, `nexa-knowledge-runtime` — governed ingestion/retrieval/context/citation foundations;
- `nexa-tutor` — provider-neutral prompt/model/admission foundations;
- `nexa-orchestrator`, `nexa-orchestrator-runtime` — lifecycle/structured-concurrency/cancellation foundations;
- `nexa-avatar`, `nexa-3d`, `nexa-3d-viewer` — renderer-neutral embodiment and concrete 3D/viewer foundations;
- `nexa-speech`, `nexa-labs` — retained contract/control foundations for later gates.

A directory containing only `.gitkeep` is a reserved boundary, not an implemented capability.

## Capability maturity

Nexa reports maturity explicitly:

`Concept -> Architecture Defined -> Specification Approved -> Contract Implemented -> Runtime Integrated -> Concrete Adapter Implemented -> System Verified -> User Accepted -> Release Ready`

Historical statements such as “Phase 3 complete” and “Phase 4 complete” refer to their documented deterministic/headless technical gates. They are not claims that those subsystems were product/release complete.

## Start here

For development or review, read:

1. [`CHATGPT_WORKFLOW.md`](CHATGPT_WORKFLOW.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`docs/PROJECT-STATUS.md`](docs/PROJECT-STATUS.md)
4. [`docs/BASELINE.md`](docs/BASELINE.md)
5. [`docs/SPECIFICATION-REGISTRY.md`](docs/SPECIFICATION-REGISTRY.md)
6. [`docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md`](docs/architecture/NEXA-ARCH-002-V1-RELEASE-ARCHITECTURE.md)
7. [`docs/architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md`](docs/architecture/NEXA-R1-IMPLEMENTATION-BASELINE.md)
8. [`docs/adr/0068-v1-r2-walking-skeleton-baseline.md`](docs/adr/0068-v1-r2-walking-skeleton-baseline.md)
9. [`docs/architecture/IMPLEMENTATION-ROADMAP.md`](docs/architecture/IMPLEMENTATION-ROADMAP.md)
10. the applicable subsystem specifications/ADRs/traceability for the selected increment.

## Existing 3D slice

The existing 3D foundation remains independently runnable and testable:

```powershell
cargo test --workspace
cargo check -p nexa-3d --no-default-features
cargo run --bin nexa-3d-validate -- path\to\Nexa.glb path\to\nexa.runtime.json
cargo run --bin nexa-3d-viewer -- path\to\Nexa.glb path\to\nexa.runtime.json
```

The avatar/3D work is retained but does not block the text-first R2 walking skeleton.

## Development selection rule

New work is selected because it advances a concrete release-path blocker and capability maturity—not merely because another narrow contract or ADR can be added.

Every implementation PR must identify:

- its governing parent architecture/specification;
- the release/E2E step it advances;
- the maturity state before and after;
- the evidence required to support that maturity claim.

## Quality and architecture control

Local PR correctness remains mandatory: format, build, lint, tests, contract/dependency boundaries, and traceability.

In addition, the Chief Systems Architect performs independent whole-system reviews at major gates or when drift signals appear. The result is explicitly Continue, Redirect, or Tactical Pause.

Local correctness never substitutes for system-level progress.

## License

The Rust workspace currently declares MIT OR Apache-2.0 licensing. Repository-wide distribution, third-party model/runtime dependencies, and asset provenance must satisfy the release packaging gate before v1 distribution.
