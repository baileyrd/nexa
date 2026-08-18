# NEXA-3D-RUNTIME-001

Minimal Rust validation/runtime viewer for the canonical Nexa GLB. It is deliberately a **validation harness**, not the production renderer. Its asset contract and `NexaAvatarAdapter` can be consumed by any renderer that implements the backend-neutral `AvatarRenderer` trait.

## Run

```powershell
cargo run --bin nexa-3d-viewer -- path\to\Nexa.glb path\to\nexa.runtime.json
```

The window draws imported GLB rest-pose geometry with depth testing, authored scene-node transforms, PBR base-color/emissive factors, and source-over material alpha; it automatically frames the bounds and overlays the joint hierarchy in cyan. The terminal reports GLB skeleton, morph-target, and animation validation; the window title reports current inspection state. Texture maps and skinning remain future rendering increments.

Controls: `1` skeleton/node inspection, `J` select the next named GLB node, `2` morph inspection, `M` select the next exported morph target, `Z/X` decrease/increase morph weight, `3` animation inspection, `N` select the next animation, `Space` play/pause, `[` / `]` scrub, arrow keys orbit, mouse wheel zoom, `G` toggle eye/head gaze, `W/A/S/D` move gaze target horizontally/depth, `Q/E` move it vertically, `V` trigger a viseme hook, `R` reset. `Esc` closes.

## Layout

```text
src/
  asset.rs       GLB inspection and deterministic validation report
  avatar.rs      NEXA-3D-001-facing semantic adapter and renderer port
  control.rs     renderer-neutral debug, gaze, viseme, and timeline state
  headless.rs    CI-safe validation runner (no GPU/window)
  viewer.rs      minimal wgpu/winit surface and debug input mapping
docs/            architecture, Blender export contract, acceptance checklist
```

## Commands

```powershell
cargo test
cargo run --bin nexa-3d-validate -- path\to\Nexa.glb path\to\nexa.runtime.json
cargo run --bin nexa-3d-viewer -- path\to\Nexa.glb
```

`nexa-3d-validate` is the recommended CI and build-automation gate. It emits a machine-readable acceptance report and never initializes wgpu, a window, audio, or OS input. It rejects assets missing a skeleton, morph targets, or renderable scene geometry. Library users may instead call `nexa_3d_runtime::headless::validate_glb`.

The repository CI always runs formatting, `cargo check --all-targets`, and headless unit tests. Add a versioned Nexa GLB plus its runtime manifest to CI only after the first asset is accepted; then invoke `nexa-3d-validate` as an additional required job.

`assets/nexa_v001.runtime.example.json` is the required semantic sidecar template. It maps approved NEXA-3D-001 expression, viseme, gesture, and gaze-rig names to the actual exported Blender/GLB identifiers. Passing it as the optional second argument verifies the mappings before the window opens.

The `runtime::apply_debug_controls` bridge converts viewer gaze and queued viseme input into `AvatarRenderer` commands. This keeps facial/IK implementation renderer-independent while providing a unit-tested hook for the first production renderer.
