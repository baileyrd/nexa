# NEXA-3D-FIRST-MODEL-ACCEPTANCE-001 — First Nexa Model Acceptance Checklist

**Gate:** no candidate GLB enters application integration until every required item passes or has a dated, approved exception.

## A. Identity and visual fidelity — human review

- [ ] Front, left/right profile, back, and 3/4 views visibly match the approved NEXA-3D-REF-003 baseline.
- [ ] Nexa remains an adult, stylized-realistic technical instructor; no independent redesign has been introduced.
- [ ] Face silhouette, eye shape, eyebrow readability, mouth proportions, and neutral attentive expression match the approved sheets.
- [ ] Dark layered hair, its silhouette from every view, technical glasses, jacket, and restrained violet/cyan accents match the approved sheets.
- [ ] Glasses are physically plausible and remain legible in the `Adjust_Glasses` pose.
- [ ] Hands and fingers read cleanly at tutor camera distance in point, open palm, pinch, typing, and thumbs-up gestures.
- [ ] Focused does not read angry; encouraging is warm but not over-cheerful; skeptical is subtle; corrective is non-judgmental.

## B. Geometry, rig, and deformation — human plus automated inspection

- [ ] `cargo run --bin nexa-3d-validate -- Nexa_vNNN.glb nexa_vNNN.runtime.json` exits successfully and emits an accepted report with at least one skin and one morph target.
- [ ] Exactly one documented canonical runtime armature drives each skinned Nexa mesh; manifest armature, head, eyes, and jaw names resolve to exported GLB nodes.
- [ ] Skin weights are normalized; no vertex uses more than four runtime influences.
- [ ] Neutral A-pose/rest pose has no unintended twist, collapse, clipping, or ground penetration.
- [ ] Head, jaw, eyes, hands/fingers, and glasses-adjacent regions deform correctly through approved actions.
- [ ] All required expression and viseme morph names are present and mapped explicitly in the runtime manifest.
- [ ] Each viseme reads at conversational camera range, returns to `REST`, and does not visibly damage lips, teeth, cheeks, or glasses.
- [ ] Morph targets mix correctly with one expression and one head/eye gaze state.

## C. Animation and behavior — viewer evidence

- [ ] Each required action loads by its canonical name, starts at zero, and has no unexplained root drift.
- [ ] Loops (`Idle_Seated`, `Listening`, `Typing` where applicable) loop cleanly.
- [ ] `Point_Left`, `Point_Right`, and `Open_Hand_Explain` preserve clean silhouette at tutor framing.
- [ ] Timeline scrubbing reaches start/end without a pose discontinuity.
- [ ] Eye gaze tracks a learner target; head follows with lower weight; limits prevent unnatural rotation.
- [ ] Viseme trigger hook accepts every canonical viseme name and leaves unrelated facial expression channels intact.

## D. GLB and delivery integrity — automated where possible

- [ ] Candidate is a single GLB with embedded buffers/images and no missing external files.
- [ ] Units, forward axis, ground origin, source hash, reference version, exporter version, and manifest version are recorded.
- [ ] All object transforms are applied; no accidental camera/light/test geometry exports.
- [ ] Textures, materials, normals, UVs, and tangent usage are valid under the target PBR convention.
- [ ] `cargo test` passes for the runtime crate.
- [ ] `cargo run --bin nexa-3d-viewer -- Nexa_vNNN.glb` opens and reports expected skins, morph count, and animation names.

## Required evidence bundle

Attach: exported GLB checksum; headless validation report; runtime-manifest JSON; viewer report; neutral front/profile/back/3/4 captures; facial expression sheet captures; viseme strip; gesture captures; reviewer, date, and accepted reference version.

## Acceptance decision

| Decision | Meaning |
|---|---|
| Accept | Candidate may integrate against NEXA-3D-001. |
| Accept with exception | Exception is documented with owner, consequence, and expiration. |
| Reject | Candidate returns to the Blender source stage; GLB is not patched manually. |
