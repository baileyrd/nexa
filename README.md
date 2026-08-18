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

The first implemented slice is split across `nexa-3d`, `nexa-3d-validate`, and `nexa-3d-viewer`: a renderer-independent GLB validation library, headless acceptance tool, and minimal `wgpu`/`winit` debug viewer. The library retains the `nexa_3d_runtime` Rust import name for migration compatibility. See [ADR-0009](docs/adr/0009-controlled-3d-workspace-migration.md) for ownership and the complete old-to-new path map.

It currently validates and exercises:

- renderable GLB scene geometry
- skeleton hierarchy and GPU skinning
- morph targets and animation timelines
- gaze and viseme control hooks
- semantic runtime manifests
- headless CI-safe asset acceptance

Renderer-neutral contracts remain in `nexa-avatar`; GPU, window, and OS input composition exists only in the viewer application.

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
