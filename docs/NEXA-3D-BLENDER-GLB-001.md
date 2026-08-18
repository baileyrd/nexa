# NEXA-3D-BLENDER-GLB-001 — Blender-to-GLB Production and Export Pipeline

**Status:** production baseline  
**Visual authority:** NEXA-3D-REF-003 and its approved sheets. No export or optimization stage may redesign Nexa’s face, hair silhouette, glasses, clothing language, proportion, expression goals, hands, or gestures.

## Source layout

```text
assets/nexa/
  blender/Nexa_master.blend            authoritative editable source
  blender/exports/Nexa_v001.glb        immutable exported candidate
  manifests/nexa_v001.runtime.json     semantic name map and provenance
  validation/                          automated report + visual approval evidence
```

`Nexa_master.blend` remains the authoring source. A GLB is a compiled delivery artifact and is never edited as the authoritative asset.

## Coordinate, naming, and scene rules

| Rule | Required value |
|---|---|
| units | metric; 1 Blender unit = 1 meter |
| up | +Y in exported glTF |
| canonical facing | -Z; document and test it in the manifest |
| ground contact | sole at Y = 0 in neutral pose |
| object transforms | apply scale/rotation before export; scale must be 1,1,1 |
| collection | export only `NEXA_EXPORT` collection |
| armature | `Nexa_Rig`; skin roots named explicitly |
| mesh names | `Nexa_Body`, `Nexa_Hair`, `Nexa_Glasses`, `Nexa_Jacket`, etc. |
| materials | `Nexa_<part>_<surface>`; stable, descriptive names |

Use a relaxed A-pose as the neutral/rest pose unless a formally approved runtime pose update supersedes it. Never bake a decorative pose into the bind pose.

## Rig and animation rules

The GLB includes exactly the runtime skeleton hierarchy required by the exported skinned meshes. Keep deformation bones; remove control-only bones unless a runtime requirement expressly needs them. Deformation weights must be normalized, and vertices must use no more than four influences for the first runtime target.

All actions exported to GLB have stable names from the approved catalog, e.g. `Idle_Seated`, `Point_Left`, `Point_Right`, `Open_Hand_Explain`, `Adjust_Glasses`, `Thumbs_Up`, `Typing`, and `Listening`. Actions start at time zero, use a documented frame rate, and avoid unexplained root drift. Looping actions state `loop: true` in the runtime manifest.

## Face and morph-target rules

The base mesh vertex order is locked before facial shape-key production. Every shape key must be relative to `Basis`, have no topology change, and use canonical names listed below. Use separate targets for visemes and expressions; production mixing is controlled by the runtime rather than baking a smile into a phoneme.

```text
Expressions: Neutral, Soft_Smile, Encouraging, Focused, Curious, Thinking,
Skeptical, Concerned, Serious, Corrective, Surprised, Confused

Visemes: REST, A, E, I, O, U, MBP, FV, L, WQ, TH, CHSH, R
```

If Blender/exporter naming transforms are needed, the explicit source-to-GLB mapping belongs in `nexa_v001.runtime.json`; do not rely on implicit string guessing.

## Materials and textures

Use glTF 2.0 metallic-roughness PBR materials. Bake maps in UV space and pack only when documented. Avoid renderer-specific shader nodes, unsupported procedural textures, hidden external dependencies, and unsupported drivers. Pack all used images into the `.blend` before final export; the delivered GLB embeds all required buffers and images.

Validate color management under the project’s documented view transform. Emissive violet/cyan accents remain restrained and must not erase the approved dark technical material hierarchy.

## Export procedure

1. Duplicate or save a versioned Blender production file; never overwrite the last accepted source.
2. In `NEXA_EXPORT`, verify only approved export objects are enabled; remove test meshes, cameras, lights, guides, high-poly source, and control widgets.
3. Apply transforms, inspect armature scale, verify bone weights, normals, UVs, material slots, and shape-key names.
4. Check every approved action and a neutral/rest-pose return.
5. Pack external textures and save the source `.blend`.
6. Export `glTF 2.0` as **GLB**, selected objects only, with animations, skins, morph targets, normals, tangents when normal maps are used, and materials/textures included.
7. Re-import the exported GLB into a clean Blender scene. Compare front, profile, rear, and 3/4 silhouette to approved reference; test animation, eyes, mouth, glasses, and hands.
8. Run `nexa_3d_runtime::headless::validate_glb` and record its report with the candidate.
9. Run the viewer inspection. Capture required evidence and complete the acceptance checklist before promotion.

## Versioning and provenance

Every GLB has immutable versioned name (`Nexa_vNNN.glb`), source Blender file/hash, exporter version, export time, reference-sheet version, and runtime-manifest version. A changed topology, bind skeleton, shape-key set, or canonical animation name creates a new asset version and requires a full regression of this checklist.
