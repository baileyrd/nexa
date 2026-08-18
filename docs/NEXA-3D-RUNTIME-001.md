# NEXA-3D-RUNTIME-001 — Nexa 3D Runtime Validation Viewer

**Status:** implementation scaffold  
**Depends on:** NEXA-3D-001, NEXA-3D-ART-001, NEXA-3D-REF-003  
**Visual authority:** approved canonical Nexa turnaround, face/expression, viseme, hand, and gesture sheets. This specification does not reinterpret or supersede them.

## Purpose

Provide a small Rust application that confirms a candidate Nexa GLB is structurally usable before it reaches any production renderer. It owns inspection and debug control semantics, not character art direction or final rendering.

## Scope and explicit non-goals

The first executable slice loads a GLB, reports its structural content, initializes a `wgpu` surface, and offers a stable control model for orbit/debug camera, skeleton inspection, morph targets, animation timeline, gaze, and viseme hooks. Mesh/material drawing and editor-grade UI are deliberate next increments. A production renderer is outside this artifact.

## Architecture

```text
NEXA-3D-001 behavior/orchestration
        │ semantic intent only
        ▼
NexaAvatarAdapter ───────────────► AvatarRenderer port
        │                                  │
        │                                  ├─ wgpu renderer
        │                                  ├─ Bevy renderer
        │                                  └─ headless test double
        ▼
RuntimeControls ── camera / timeline / gaze / viseme
        │
        ▼
GLB inspector ── metadata + acceptance gates ── CI/headless
```

`AvatarRenderer` is the renderer boundary. The NEXA behavior layer sends canonical names (for example `Focused`, `Point_Right`, `A`) rather than glTF node indices or `wgpu` objects. The selected renderer resolves those names through the asset manifest.

`behavior::AvatarBehaviorEvent` is the preferred NEXA-3D-001 integration input. Its dispatcher transforms expression, gesture, gaze, and viseme intents into adapter commands, preserving event order and keeping authoring/orchestration independent of rendering technology.

## Stable semantic contract

The runtime exposes four commands: `ExpressionCommand`, `VisemeCommand`, `GazeCommand`, and `GestureCommand`. Renderers must treat missing or unmapped names as diagnosable validation errors, never silently substitute an unrelated pose.

Canonical content namespaces:

| Category | Canonical identifier examples |
|---|---|
| expressions | `Neutral`, `Focused`, `Encouraging`, `Skeptical`, `Corrective` |
| visemes | `REST`, `A`, `E`, `I`, `O`, `U`, `MBP`, `FV`, `L`, `WQ`, `TH`, `CHSH`, `R` |
| gestures | `Idle_Seated`, `Point_Left`, `Point_Right`, `Open_Hand_Explain`, `Adjust_Glasses`, `Thumbs_Up`, `Typing`, `Listening` |
| gaze targets | world-space semantic targets resolved by host scene (`Learner`, `LessonDisplay`, `Terminal`, `Diagram:<id>`) |

## First viewer controls

| Input | Action |
|---|---|
| `1`, `2`, `3` | skeleton, morph-target, animation inspection mode |
| arrows | orbit camera |
| space | playback toggle |
| `[` / `]` | timeline scrub |
| `G` | eye/head gaze toggle |
| `V` | trigger sample `A` viseme hook |
| `R` | restore debug defaults |

The control state itself is renderer-independent and directly unit-testable. A GUI inspector can replace the title-bar implementation without changing the contract.

`runtime::apply_debug_controls_at` is the time-aware control-to-renderer bridge. It converts enabled gaze to a `GazeCommand`, schedules queued visemes, and emits their sampled blendshape weights each frame. `gaze::solve` provides the portable bounded head/eye yaw-pitch split; `viseme::VisemePlayer` provides attack/hold/release cue envelopes; the active renderer maps both results onto named joints and blendshapes.

## Headless contract

`headless::validate_glb(path)` is the CI-safe required gate. It does not instantiate a GPU surface, window, event loop, audio system, or OS input. It must validate at least one skeleton, one morph target, and nonempty renderable scene geometry. Project CI should invoke this function plus the manifest checks described in the acceptance checklist.

The test suite generates a minimal skinned/morph-enabled glTF fixture at runtime and runs this exact headless path. This is a pipeline-contract test only; the approved Nexa asset remains the visual acceptance authority.

## Rendering increment plan

1. Current: GLB metadata import, headless checks, `wgpu` device/surface, debug input state.
2. Add GPU buffer/material upload and static mesh draw. ✓
3. Add skinning, joint visualization, and named node selection. (Rest-pose and headless-regression-tested sampled-clip joint visualization, plus named-node inventory and selection, are available; mesh skinning remains pending.)
4. Add morph weights, animation sampler evaluation, and timeline UI. (Morph preview, clip-duration looping, and a renderer-neutral sampler with headless-tested GLB linear/step transform-channel adaptation are available; cubic/multi-weight adaptation and GPU skinning remain pending.)
5. Add gaze/IK and viseme mixing through the semantic adapter.
6. Add image regression tests using a deterministic offscreen backend where available.

Every increment retains the headless inspector and must keep `AvatarRenderer` free of concrete renderer types.
