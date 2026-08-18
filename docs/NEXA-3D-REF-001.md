# NEXA-3D-REF-001 — Canonical 3D Character Reference & Turnaround Specification

**Version:** 1.0
**Status:** approved visual authority
**Depended on by:** NEXA-3D-ART-001, NEXA-3D-BLENDER-GLB-001, NEXA-3D-RUNTIME-001
**Source:** transcribed from the [design conversation export](reference/NEXA-3D-SOURCE-CONVERSATION.md)

This document and the two approved sheets it owns are the visual authority for the project.
Downstream specifications, exports, optimizations, and renderers consume them; none of them
reinterpret or supersede them. **Do not redesign the character.**

## Objective

Lock one consistent visual character across front, 3/4, profile, back, facial, expression,
hand, gesture, material, and environment references before serious Blender production begins.

## Character direction

An adult female technical instructor with a stylized-realistic cyber/hacker aesthetic:
intelligent, technically formidable, approachable, composed, slightly playful, modern, and
professional.

- Dark layered hair with subtle purple/cool highlights.
- Signature thin technical glasses with restrained violet/cyan details.
- Slightly stylized, highly readable eyes and expressive brows.
- Dark fitted technical jacket, dark technical underlayer and trousers, reinforced footwear.
- Palette dominated by graphite/charcoal with electric violet and cool cyan accents.
- Cyber details stay restrained: seams, collar indicators, glasses electronics, wrist interface.
- Approximate modeling reference: ~1.68 m, ~7.25 heads tall, slim and naturally athletic.
- Front, side, back, and 3/4 references use a consistent A-pose and orthographic presentation.

## Approved sheets

### NEXA-3D-REF-002 — Canonical Character Turnaround

![NEXA-3D-REF-002 canonical character turnaround](reference/images/NEXA-3D-REF-002_Canonical_Turnaround.png)

Sheet metadata: v1.0, dated 2025-05-18. Eye color **Deep Violet**, height **~1.68 m**,
build **slim / athletic**.

Five views at consistent scale: front, 3/4 front, side, 3/4 back, back, with a height ruler
marked at 0.00 m, 0.50 m, 1.00 m, 1.40 m, and 1.68 m.

| Proportion | Value |
|---|---|
| height | ~1.68 m |
| head / body ratio | ~7.25 heads |
| build | slim / athletic |
| shoulders | moderate |
| arms | proportional |
| legs | proportional |
| hands | expressive |

Palette swatches: base, secondary, accent 1, accent 2, emission 1, emission 2, skin, hair,
metal.

| Surface | Finish |
|---|---|
| jacket fabric | technical fabric, slight sheen |
| pants fabric | matte technical fabric |
| armor panels | satin composite |
| metal parts | gunmetal, brushed |
| emissive elements | soft glow (violet / cyan) |

Sheet notes, which are binding on modeling: A-pose with arms ~15° from body; neutral
expression; orthographic presentation; consistent lighting; no environment or background;
scale reference on the left; the design is symmetrical; **all proportions are final for
modeling**.

### NEXA-3D-REF-003 — Facial, Expression, Viseme, Hand, Gesture & Material Reference Board

![NEXA-3D-REF-003 reference board](reference/images/NEXA-3D-REF-003_Reference_Board.png)

Sheet metadata: v1.0, dated 2025-05-18.

- **Facial close-ups:** front, 3/4 left, profile left.
- **Expression sheet (12):** neutral, soft smile, encouraging, focused, curious, thinking,
  skeptical, concerned, serious, corrective, surprised, confused.
- **Viseme reference (front):** the canonical set, front-facing, one tile per viseme.
- **Hand reference (8):** relaxed, open palm, point, pinch, fist, thumbs up, two fingers,
  typing — each shown from two angles.
- **Upper-body gesture reference (6):** point left, point right, open-hand explain,
  two-hand explain, thinking, adjust glasses.
- **Material and color reference:** the palette swatches above, plus jacket fabric, armor
  panels, metal parts, glasses frame (two variants), glass lens, and emissive elements.
- **Workstation reference (context):** dark materials, purple/cyan lighting, lesson and code
  displays, subtle holographics.

Sheet notes: orthographic reference for modeling and animation; all proportions final;
consistent lighting (key front/top, rim back); no perspective distortion; for use in 3D
modeling, texturing, rigging, and animation.

## Canonical name sets

The sheets carry human-readable display labels; the runtime manifest keys on stable
identifiers. Both are listed below, because they differ in places — notably the sheet's
`CH / SH` tile, whose identifier is `CHSH`. See `assets/nexa_v001.runtime.example.json` for
the template and `src/manifest.rs` for the enforced set.

**Expressions (12 on the sheet).** Neutral, soft smile, encouraging, focused, curious,
thinking, skeptical, concerned, serious, corrective, surprised, confused.
Of these, `manifest.rs` currently *requires* five: `Neutral`, `Focused`, `Encouraging`,
`Skeptical`, `Corrective`. The remaining seven are approved art but not yet gated.

**Visemes (13).** All are required by `manifest.rs`:
`REST`, `A`, `E`, `I`, `O`, `U`, `MBP`, `FV`, `L`, `WQ`, `TH`, `CHSH`, `R`.

**Hands (8 on the sheet).** Relaxed, open palm, point, pinch, fist, thumbs up, two fingers,
typing. NEXA-3D-REF-001's prose additionally names a precision gesture.

**Gestures.** `manifest.rs` requires eight animation mappings:
`Idle_Seated`, `Point_Left`, `Point_Right`, `Open_Hand_Explain`, `Adjust_Glasses`,
`Thumbs_Up`, `Typing`, `Listening`. The REF-003 board covers point left, point right,
open-hand explain, two-hand explain, thinking, and adjust glasses; `Two_Hand_Explain`,
`Thinking`, and `Attention` appear in the art direction but are not yet gated.

## Open discrepancies

Recorded rather than silently resolved, because the sheets are the visual authority and only
their owner can correct them.

1. **Viseme tile labels on NEXA-3D-REF-003.** The canonical set has 13 visemes, but the
   sheet's viseme strip appears to carry 14 tiles with `WQ` labelled twice, which would mean
   one tile is mislabelled and one viseme has no reference art. This was read off the
   rendered sheet, so confirm it against the source before acting. It matters: `manifest.rs`
   requires all 13 viseme names, so a viseme with no sheet coverage will block the
   acceptance gate once a real GLB is submitted.
2. **Gesture coverage.** `manifest.rs` requires `Idle_Seated` and `Listening`, neither of
   which has a tile on the REF-003 gesture strip, while the strip's `two-hand explain` and
   `thinking` are not required by the manifest. Reconcile the two lists before rigging.
3. **Sheet dates.** Both sheets are dated 2025-05-18 while the conversation export is dated
   2026-08-18. The sheet dates are reproduced here as they appear.
