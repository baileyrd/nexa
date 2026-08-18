# NEXA-3D-ART-001 — Nexa 3D Character Modeling, Topology, Rigging & Asset Production Specification

**Version:** 1.0
**Status:** production baseline
**Depends on:** NEXA-3D-001, NEXA-3D-REF-001
**Depended on by:** NEXA-3D-BLENDER-GLB-001, NEXA-3D-RUNTIME-001
**Visual authority:** [NEXA-3D-REF-001](NEXA-3D-REF-001.md) and its approved sheets. No stage in this pipeline may redesign the character.
**Source:** transcribed from the [design conversation export](reference/NEXA-3D-SOURCE-CONVERSATION.md)

## Objective

Build a canonical, expressive, real-time 3D Nexa that is visually faithful, renderer-neutral,
GLB/glTF exportable, VRM-compatible where practical, and maintainable.

## Production pipeline

```text
canonical artwork
    └─ reference standardization
        └─ turnaround sheets
            └─ facial / gesture sheets
                └─ base sculpt
                    └─ retopology
                        └─ UVs
                            └─ textures / materials
                                └─ humanoid skeleton
                                    └─ skinning
                                        └─ facial rig
                                            └─ visemes
                                                └─ corrective shapes
                                                    └─ hair / clothing physics
                                                        └─ animation tests
                                                            └─ LODs
                                                                └─ glTF / VRM export
                                                                    └─ validation
                                                                        └─ runtime package
```

## Identity-critical features

These are the features that make the character recognizably Nexa. Treat any change to them as
a redesign requiring sign-off, not an optimization:

- face silhouette
- eye shape
- hair silhouette and color
- glasses
- violet accents
- technical clothing
- adult presentation
- body proportions
- cyber/hacker aesthetic
- composed demeanor

## Geometry and topology

- Head, eyes, teeth, hair, glasses, clothing, and accessories are separate and organized.
- Facial topology is optimized around eyes, brows, mouth, cheeks, and jaw.
- Proper mouth cavity and teeth; independent spherical eyes with conforming eyelids.
- Fully articulated hands and fingers.

### LOD triangle targets

These are illustrative targets, not hard gates.

| LOD | Triangles |
|---|---|
| L0 | 80k – 140k |
| L1 | 45k – 80k |
| L2 | 20k – 40k |
| L3 | 8k – 20k |

## Materials

PBR: base color, normal, roughness, metallic, ambient occlusion, emissive.

## Rig

- Canonical A-pose.
- Root bone, multi-segment spine, clavicles, twist bones, fingers, eye bones, jaw bone.
- Baseline of **4 skin influences per vertex** for portability.

## Facial targets and visemes

Semantic facial targets, plus the canonical viseme set:

`REST`, `A`, `E`, `I`, `O`, `U`, `MBP`, `FV`, `L`, `WQ`, `TH`, `CHSH`, `R`

## Core animation set

Seated idle, standing idle, listening, thinking, speaking, open hand, point left, point right,
two-hand explain, chin think, adjust glasses, nod, head shake, thumbs up, attention, typing,
celebration.

## Tooling and validation

- Blender is the likely production DCC. Official builds record Blender and add-on versions.
- An asset compiler validates geometry, rig, skin, facial targets, animation, materials, and
  performance. NEXA-3D-RUNTIME-001's `nexa-3d-validate` is the current implementation of that
  gate.
- The first runnable gate begins at the talking-head milestone rather than waiting for the
  entire body and environment.
