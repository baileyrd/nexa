# NEXA-3D-001 — 3D Character, Humanoid Rig, Facial Animation & Rendering Architecture

**Version:** 1.0
**Status:** production baseline
**Depended on by:** NEXA-3D-ART-001, NEXA-3D-RUNTIME-001, NEXA-3D-BLENDER-GLB-001
**Source:** transcribed from the [design conversation export](reference/NEXA-3D-SOURCE-CONVERSATION.md)

## Purpose

Define Nexa's production-grade 3D embodiment: canonical asset format, humanoid skeleton,
facial rig, blendshapes, visemes, gaze, inverse kinematics, gestures, animation graphs,
physics, materials, lighting, camera behavior, render abstraction, VRM/glTF compatibility,
performance budgets, XR extensibility, and the migration path from 2D/2.5D to full 3D.

## Architectural flow

```text
Tutor Intent
    │
    ▼
BehaviorIntent
    │
    ▼
NEXA-NBP-001  (Nexa Behavior Protocol)
    │
    ▼
3D Behavior Engine
    │
    ▼
Rig / IK / Face / Animation
    │
    ▼
Renderer Adapter
    │
    ▼
Animated 3D Nexa
```

The 2D and 3D embodiments share the same NEXA Behavior Protocol. A change in embodiment is a
change of adapter, not a change of behavior authoring.

## Canonical asset decisions

| Decision | Value |
|---|---|
| canonical foundation | glTF 2.x |
| humanoid metadata | VRM-compatible |
| extensions | Nexa-specific, layered on top of glTF |
| engine-native assets | Unity prefab, Unreal asset, Godot scene are **deployment artifacts**, never source of truth |

The canonical rig supports humanoid body, fingers, jaw, eyes, facial blendshapes, visemes,
gaze, and IK.

## Behavior and animation model

- Audio playback is the timing master for lip sync.
- Gaze is semantic-target driven, with coordinated eyes, head, neck, and torso.
- Animation is layered, in this order:
  1. base pose
  2. locomotion / seated loop
  3. behavior
  4. upper-body gesture
  5. hand pose
  6. head additive
  7. gaze IK
  8. expression
  9. lip sync
  10. secondary physics

## Renderer abstraction

Native Rust/`wgpu` is a first-class path. Godot, Unity, Unreal, and browser/WebGPU adapters
are future targets behind the same boundary. NEXA-3D-RUNTIME-001 implements this boundary as
the `AvatarRenderer` port.

## Quality and performance

- Quality degradation preserves face, eyes, mouth, gaze, and core gestures **before**
  hair and cloth effects.
- Desktop target is 60 FPS, with a usable 30 FPS fallback.
- Headless animation/IK testing and deterministic NBP replay are required.

## Milestone order

1. Head and upper torso, eyes, blink, expressions, viseme lip sync, head movement, gaze.
2. Arms, hands, and pointing IK.
3. Full body and physics.
