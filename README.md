# Nexa

Nexa is a local-first, adaptive AI training tutor platform. It combines structured tutor intelligence, pedagogy, student modeling, governed knowledge retrieval, curriculum, assessment, interactive labs, speech, and an expressive renderer-independent avatar.

Nexa is not an avatar attached to a chatbot. The avatar is the human-facing embodiment of a larger learning system designed to develop demonstrable competency.

## Architectural rule

Tutor intelligence produces semantic communicative intent. The behavior and avatar layers decide how that intent is physically expressed.

The LLM does not select animation clips, manipulate bones, set blendshape weights, or issue renderer commands.

```text
Student interaction
        |
Session orchestrator
        |
Tutor + pedagogy + knowledge + student state
        |
Structured tutor response and BehaviorIntent
        |
Nexa Behavior Protocol
        |
Speech, canvas, tools, and avatar adapters
```

## Repository layout

| Path | Purpose |
|---|---|
| `apps/` | Desktop, authoring, CLI, and viewer composition roots |
| `crates/` | Reusable domain and subsystem implementations |
| `tools/` | Content, assessment, lab, and asset compilers and validators |
| `content/` | Courses, knowledge, assessments, and labs |
| `assets/` | Avatars, scenes, and speech assets |
| `docs/` | Architecture, specifications, ADRs, governance, and provenance |
| `crates/nexa-3d/` | Renderer-independent 3D runtime, validation, and avatar adapter |
| `crates/nexa-student/` | Headless student model, immutable evidence ledger, and replayable mastery policy |
| `crates/nexa-pedagogy/` | Pure, versioned, explainable policy over read-only mastery projections |
| `crates/nexa-lessons/` | Validated authored curriculum and pure, versioned headless lesson transitions |
| `apps/nexa-3d-viewer/` | `wgpu`/`winit` interactive viewer composition root |
| `tools/nexa-3d-validate/` | GPU-free asset and manifest validation CLI |

A directory containing only `.gitkeep` reserves a planned boundary; it does not indicate that the capability is implemented.

## Start here

- [Reconstructed baseline policy](docs/BASELINE.md)
- [Canonical specification registry](docs/SPECIFICATION-REGISTRY.md)
- [Tutor system architecture](docs/Nexa%20Tutor%20System%20%E2%80%94%20Architecture%20v0.1.md)
- [Character and behavior specification](docs/Nexa%20Character%20&%20Behavior%20Specification%20v1.0.md)
- [Implementation roadmap](docs/architecture/IMPLEMENTATION-ROADMAP.md)
- [Architecture decisions](docs/adr/)
- [Contributing](CONTRIBUTING.md)

## Current implementation

The first implemented slice is split across `nexa-3d`, `nexa-3d-validate`, and `nexa-3d-viewer`: a renderer-independent GLB validation library, headless acceptance tool, and minimal `wgpu`/`winit` debug viewer. The library retains the `nexa_3d_runtime` Rust import name for migration compatibility. See [ADR-0008](docs/adr/0008-controlled-3d-workspace-migration.md) for ownership and the complete old-to-new path map.

It currently validates and exercises:

- renderable GLB scene geometry
- skeleton hierarchy and GPU skinning
- morph targets and animation timelines
- gaze and viseme control hooks
- semantic runtime manifests
- headless CI-safe asset acceptance

Renderer-neutral contracts remain in `nexa-avatar`; GPU, window, and OS input composition exists only in the viewer application.

Phase 3 includes dependency-light `nexa-student` and `nexa-pedagogy` slices. Canonical evidence is append-only,
duplicate ingestion is idempotent, and mastery is a derived projection replayed with an explicit policy
version. The pure pedagogy policy reads projections without mutation and returns only available routing
options with stable rationale codes. Only ports and deterministic test adapters exist; no database or async
runtime has been selected. The lesson slice validates immutable authored graphs and consumes only
explicitly authored pedagogy routes into atomic progress transitions. Phase 3 remains incomplete
pending the assessment engine.
See [ADR-0010](docs/adr/0010-learning-state-evidence-and-persistence.md),
[ADR-0011](docs/adr/0011-pedagogy-policy-ownership-and-versioning.md),
[ADR-0012](docs/adr/0012-governed-curriculum-and-lesson-transitions.md), and the
[Phase 3 traceability matrix](docs/architecture/PHASE-3-TRACEABILITY.md).

## Run the implemented 3D slice

```powershell
cargo test --workspace
cargo check -p nexa-3d --no-default-features
cargo run --bin nexa-3d-validate -- path\to\Nexa.glb path\to\nexa.runtime.json
cargo run --bin nexa-3d-viewer -- path\to\Nexa.glb path\to\nexa.runtime.json
```

The viewer supports skeleton/node inspection, morph inspection, animation playback and scrubbing, orbit/zoom, gaze targeting, and viseme hooks. See [NEXA-3D-RUNTIME-001](docs/specifications/11-avatar-3d/NEXA-3D-RUNTIME-001.md) for its contracts.

## Development status

The reconstructed specifications are the working design baseline, not a claim that every subsystem is implemented. Development proceeds in governed phases:

1. baseline governance
2. shared domain, event, and behavior contracts
3. embodiment migration
4. learning core
5. knowledge and tutor intelligence
6. complete session orchestration
7. authoring, packaging, and operations

See the roadmap for phase gates and acceptance outcomes.

## License

The current Rust package declares MIT or Apache-2.0 licensing. Repository-wide licensing and third-party asset provenance will be formalized before distribution.

## Phase 2 embodiment acceptance

The headless composition now accepts a complete NBP command, evaluates renderer-neutral capabilities, dispatches through `AvatarPort`, and returns correlated NBP acknowledgements/state/errors plus typed lifecycle events. A successful synchronous fake execution emits `accepted`, `started`, then `completed`; acceptance never implies completion. Optional unsupported facilities and unresolved semantic canvas targets degrade explicitly, cancellation is terminal, and rejection/failure remain distinct.

```rust
let report = adapter.handle(nexa_avatar::AvatarRequest::try_from(&nbp_message)?);
let outputs = report.to_nbp_messages(
    &nbp_message,
    "nexa.avatar".parse()?,
    nexa_domain::Sequence::new(1),
    caller_supplied_message_ids,
)?;
```

The caller supplies output identities and sequences. The core is synchronous and contains no renderer, GPU, window, audio provider, network, persistence, or async-runtime dependency. See [ADR-0009](docs/adr/0009-embodiment-acceptance-and-lifecycle.md) and the [Phase 2 traceability matrix](docs/architecture/PHASE-2-TRACEABILITY.md).
