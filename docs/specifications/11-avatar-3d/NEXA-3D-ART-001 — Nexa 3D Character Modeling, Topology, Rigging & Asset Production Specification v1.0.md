NEXA-3D-ART-001 — Nexa 3D Character Modeling, Topology, Rigging & Asset Production Specification v1.0

Specification ID: NEXA-3D-ART-001
System: Nexa AI Training Tutor
Version: 1.0
Status: Baseline Draft
Depends On: NEXA-CBS-001, NEXA-AVTR-001, NEXA-3D-001
Purpose: Define the actual production requirements for building Nexa as a high-quality real-time 3D character asset, including modeling standards, proportions, topology, facial deformation, skeleton, skinning, blendshapes, visemes, hair, clothing, materials, LODs, export rules, validation, and the Blender-centered production workflow.

1. Primary Objective

The goal is to create a canonical 3D Nexa asset that is:

visually faithful to the established character;
expressive at conversational distance;
suitable for real-time desktop rendering;
compatible with facial animation and lip sync;
capable of upper-body and full-body gestures;
capable of pointing and gaze IK;
renderer-neutral;
exportable through glTF/GLB;
compatible with VRM semantics where practical;
maintainable as a long-lived production asset.

The first production target is not maximum photorealism.

The target is:

A highly expressive, polished, stylized-realistic cyber tutor who looks convincing during prolonged face-to-face interaction.

2. Character Production Pipeline
Canonical Nexa Artwork
        ↓
Reference Standardization
        ↓
Turnaround Sheets
        ↓
Facial Expression Sheet
        ↓
Gesture / Pose Sheet
        ↓
Base Sculpt
        ↓
Production Retopology
        ↓
UV Layout
        ↓
Textures / Materials
        ↓
Humanoid Skeleton
        ↓
Skinning
        ↓
Facial Rig
        ↓
Visemes
        ↓
Corrective Shapes
        ↓
Hair / Clothing Physics
        ↓
Animation Tests
        ↓
LOD Production
        ↓
glTF / VRM Export
        ↓
Nexa Asset Validation
        ↓
Canonical Runtime Package
3. Source of Visual Truth

The approved Nexa reference artwork SHALL be the visual source of truth.

The 3D production team SHALL not independently reinterpret core character identity without explicit design review.

Identity-critical features include:

face silhouette
eye shape
hair silhouette
hair color
glasses
violet/purple accent treatment
technical clothing
age presentation
body proportions
cyber/hacker aesthetic
overall demeanor
4. Required Reference Package

Before production modeling begins, the project SHOULD have:

Front turnaround
3/4-front turnaround
Left profile
Right profile
Back
3/4-back
Neutral facial close-up
Body proportion guide
Color palette
Material reference
Accessory detail sheet

These SHALL use consistent proportions.

5. Expression Reference Package

At minimum:

neutral
soft smile
full smile
focused
curious
thinking
skeptical
concerned
serious
corrective
surprised
confused
encouraging
celebrating

These images define deformation goals for the facial rig.

6. Gesture Reference Package

At minimum:

neutral seated
neutral standing
listening
thinking chin-touch
open-hand explanation
single-hand point left
single-hand point right
two-hand explanation
small nod
head tilt
glasses adjustment
thumbs up
attention/warning
typing
7. Production Coordinate Standard

Canonical model space SHOULD use:

Units: meters
Up axis: Y
Forward: -Z or documented canonical forward
Handedness: right-handed
Origin: ground plane centered beneath pelvis

The exact forward axis SHALL be fixed once and documented.

8. Character Scale

Nexa SHALL have a canonical physical height, even though most desktop usage will be waist-up.

A suitable initial target is approximately:

1.65–1.72 meters

The exact canonical height SHOULD be selected during turnaround creation and then frozen.

9. Body Proportion Style

The character SHOULD use believable adult human proportions with restrained stylization.

Desired direction:

realistic torso proportions
slightly stylized facial features
slightly larger expressive eyes than strict realism
natural shoulders
natural arm length
natural hand size

Avoid extreme anime proportions that reduce credibility during technical instruction.

10. Head-to-Body Ratio

A reasonable target:

~7.0–7.5 heads tall

depending on final stylization.

This SHALL be validated against the canonical concept art.

11. Facial Priority

The face is the highest-value area of the asset.

Production priority SHALL be:

1. eyes
2. mouth
3. brows
4. cheeks
5. jaw
6. nose
7. hairline
8. secondary facial details

A mediocre body with a strong face is usable.

A perfect body with an unconvincing face is not.

12. Mesh Organization

Recommended top-level mesh separation:

Body
Head
Eyes
Teeth
Tongue
Hair_Back
Hair_Side_L
Hair_Side_R
Hair_Front
Glasses
Jacket
Shirt
Pants
Shoes
Accessories

Some parts MAY be merged later for runtime optimization.

13. Head Mesh

The head SHOULD be its own deformation-optimized mesh or submesh.

It SHALL support:

facial blendshapes
jaw motion
eye sockets
brow deformation
cheek deformation
lip deformation
14. Facial Topology

Facial topology SHALL follow deformation loops around:

eyes
brows
nose
mouth
nasolabial region
jawline
cheeks

Topology SHALL prioritize animation quality over sculpt convenience.

15. Mouth Topology

The mouth SHALL include sufficient concentric edge loops for:

closure
smile
frown
pucker
wide mouth
lip compression
speech visemes
16. Mouth Cavity

A proper mouth cavity SHOULD be modeled.

Minimum:

inner lips
mouth interior
teeth
tongue or tongue placeholder

This avoids hollow or flat-looking speech animation.

17. Teeth

Upper and lower teeth SHOULD be separate objects or logically separable meshes.

They SHALL not deform with lip blendshapes.

18. Tongue

The tongue is optional for MVP but the mouth topology SHOULD leave room for later support.

Recommended eventual controls:

tongue up
tongue forward
tongue lateral

Useful for phoneme-level close-up animation.

19. Eyes

Each eye SHOULD use separate spherical geometry.

Required:

left eye
right eye
iris/pupil representation
corneal highlight capability

The eyes SHALL rotate independently.

20. Eye Geometry

Avoid painted-flat eyes for the canonical 3D model.

Real eye geometry provides much stronger gaze realism.

21. Eyelids

Upper and lower eyelids SHALL conform properly to eye curvature.

Blink deformation MUST:

cover the eye naturally
avoid clipping
avoid volume collapse
22. Brow Topology

Brows may be:

mesh
texture + geometry hybrid

but the facial mesh SHALL support believable brow-region deformation.

23. Nose

The nose should remain relatively stable during most expressions.

Extreme deformation SHOULD be avoided.

24. Cheeks

Cheek deformation is important for:

smiling
squinting
surprise
encouragement

The facial rig SHOULD include cheek raise controls.

25. Jawline

Jaw motion SHALL preserve:

chin volume
cheek continuity
neck transition

during speech.

26. Neck Topology

The neck requires sufficient geometry for:

yaw
pitch
roll

without severe stretching.

27. Shoulder Topology

Shoulders are one of the most failure-prone areas.

Production topology SHALL support:

arm raise
pointing
cross-body gestures
two-hand explanation

Corrective shapes SHOULD be expected.

28. Elbows

Elbow topology SHALL preserve volume during:

90° bend
full flexion
pointing
typing
29. Wrists

Wrist twisting requires appropriate edge flow and possibly helper bones.

30. Hands

Hands are extremely important because Nexa teaches through gestures.

The canonical model SHALL include fully modeled hands with articulated fingers.

31. Finger Geometry

Each finger SHALL have sufficient segments to preserve shape during curling.

Avoid ultra-low-poly mitten-like hand topology.

32. Fingernails

Simple fingernail geometry or texture detail MAY be included.

This is lower priority than hand silhouette and deformation.

33. Target Runtime Mesh Density

The production asset SHOULD support multiple quality targets rather than one fixed polygon count.

Illustrative target ranges:

L0 close-up:
80k–140k triangles


L1 desktop:
45k–80k triangles


L2 compact:
20k–40k triangles


L3 distant:
8k–20k triangles

These are starting engineering targets, not immutable limits.

34. Triangle Distribution

Budget SHOULD be concentrated on:

face
eyes
hands
hair silhouette
deforming clothing

Less important areas may use lower density.

35. LOD Philosophy

LOD reduction SHALL preserve:

face silhouette
eye shape
mouth silhouette
hand readability
hair silhouette

as long as conversational use remains possible.

36. UV Strategy

Recommended:

UV0:
primary material textures


UV1:
optional lightmap or special-purpose data

UV islands SHOULD prioritize visible face/hands.

37. Texture Sets

Likely texture groups:

Head/Skin
Body/Skin
Hair
Clothing
Glasses
Accessories

Atlas consolidation MAY be performed for runtime optimization.

38. Texture Resolution Targets

Illustrative master sizes:

Head:      4096
Body:      2048–4096
Hair:      2048–4096
Clothing:  2048–4096
Accessories: 1024–2048

Runtime packaging MAY generate lower-resolution variants.

39. PBR Maps

Canonical materials SHOULD support:

Base Color
Normal
Roughness
Metallic
Ambient Occlusion
Emissive

Optional:

Thickness
Subsurface mask
40. Skin Material

Skin SHOULD use:

non-metallic
moderate roughness variation
subtle normal detail
controlled subsurface approximation

Avoid overly plastic skin.

41. Skin Detail

Micro-details MAY include:

pores
subtle lip detail
small tonal variation

but should not fight the stylized character direction.

42. Hair Material

For stylized real-time hair, preferred approach is likely:

mesh masses
+
hair cards where needed

rather than thousands of simulated strands.

43. Hair Silhouette

Hair silhouette SHALL be treated as identity-critical.

Optimization SHALL not significantly change the recognizable outline.

44. Hair Segmentation

Recommended major dynamic groups:

rear mass
left side
right side
front fringe
long accent strands
45. Hair Physics Bones

Physics chains MAY be placed through major movable sections.

They SHOULD be sparse enough to remain stable.

46. Clothing Construction

Clothing SHOULD be modeled as actual geometry where silhouette matters.

The canonical design SHOULD include the established:

dark cyber jacket
technical underlayer
purple/cyan details
47. Clothing Deformation

Garments SHALL be tested under:

arms forward
arms raised
cross-body pointing
typing
seated pose
standing pose
48. Cloth Simulation

Only loose elements require real secondary simulation.

Most clothing should remain skinned to the skeleton.

49. Glasses Geometry

The glasses SHALL be real geometry.

They should include:

frame
lenses
optional emissive/UI elements
50. Glasses Identity

The glasses are one of Nexa's strongest recognizable features.

They SHALL not become generic fashion eyewear during 3D translation.

51. Lens Material

Lens material SHOULD support:

high transparency
low distortion
controlled reflection

without obscuring the eyes.

52. Glasses Rigging

Glasses SHOULD follow the head but permit an adjustment animation.

Optional control:

GlassesOffset

for the glasses-adjust gesture.

53. Accessory Strategy

Accessories SHALL be:

visually meaningful
low clutter
semantically consistent

Avoid overloading Nexa with cyberpunk props that reduce her professional tutor appearance.

54. Canonical Skeleton

Recommended hierarchy:

Root
└── Hips
    ├── Spine01
    ├── Spine02
    ├── Chest
    ├── UpperChest
    │   └── Neck
    │       └── Head
    │
    ├── Clavicle_L
    │   └── UpperArm_L
    │       └── LowerArm_L
    │           └── Hand_L
    │
    └── Clavicle_R
        └── UpperArm_R
            └── LowerArm_R
                └── Hand_R

plus legs and fingers.

55. Root Bone

A root bone SHOULD exist separately from hips.

This simplifies:

retargeting
root motion
world placement
56. Spine

At least 3 useful torso articulation segments SHOULD exist.

Example:

Spine01
Spine02
Chest
UpperChest
57. Neck

At least one neck joint is required.

A two-joint neck MAY improve natural head tracking.

58. Shoulder/Clavicle Bones

Clavicles SHALL be independently animated.

They are important for pointing and open-arm gestures.

59. Arm Twist Bones

Recommended:

UpperArmTwist_L/R
ForearmTwist_L/R

These improve deformation during pronation/supination.

60. Leg Twist Bones

Optional but useful for standing/full-body quality.

61. Finger Bones

Required per hand:

Thumb 3
Index 3
Middle 3
Ring 3
Little 3

A metacarpal layer MAY be included for higher-quality hand motion.

62. Eye Bones

Recommended:

Eye_L
Eye_R

Optional:

EyeTarget

for authoring convenience.

Runtime gaze SHOULD still use semantic targets.

63. Jaw Bone

Required:

Jaw

It SHALL rotate from anatomically plausible placement.

64. Tongue Bones

Optional for later phoneme refinement.

65. Hair Bones

Hair physics bones SHALL be clearly namespaced.

Example:

Hair_Back_01
Hair_Back_02
Hair_Side_L_01
...
66. Cloth/Accessory Bones

Likewise:

JacketTail_L_01
Cable_R_01
67. Bone Naming Convention

Recommended canonical pattern:

PascalCase
_L / _R suffix

Examples:

UpperArm_L
ForearmTwist_R
Eye_L
68. Joint Orientation

Bone axes SHALL be standardized.

The production pipeline SHALL reject arbitrarily oriented rig bones that complicate retargeting.

69. Rest Pose

Canonical rest pose SHOULD be:

A-pose

unless a strong production reason favors T-pose.

The exact pose SHALL be stored as canonical rig metadata.

70. Skinning Standard

Skinning SHALL prioritize believable deformation during tutoring motions rather than extreme athletic movement.

Primary validation motions:

typing
pointing
open-hand explanation
chin touch
glasses adjustment
arm crossing
seated posture
head turns
71. Maximum Skin Influences

Runtime target SHOULD support:

4 influences per vertex

as the baseline portable target.

Higher counts MAY exist in authoring but SHOULD be reduced during export if necessary.

72. Weight Normalization

Weights SHALL sum correctly and contain no stray influences.

73. Skinning Validation Pose Library

At minimum:

arms up
arms forward
arms crossed
point left
point right
typing
hands near face
seated
head extreme yaw
head extreme pitch
74. Corrective Blendshapes

Corrective shapes SHOULD be expected for:

shoulder raise
elbow flexion
wrist bend
neck extremes
jaw extremes
large smile
75. Facial Rig Philosophy

The canonical facial rig SHOULD support both:

semantic expression controls
speech articulation controls

without the two systems fighting each other.

76. Facial Blendshape Naming

Recommended names should be semantic and engine-neutral.

Example:

Face_Blink_L
Face_Blink_R
Face_BrowUp_L
Face_BrowUp_R
Face_Smile_L
Face_Smile_R
Face_Frown_L
Face_Frown_R
Face_CheekRaise_L
Face_CheekRaise_R
Face_MouthWide
Face_MouthPucker
77. VRM Expression Mapping

Where VRM-compatible equivalents exist, the exporter SHOULD map them.

78. ARKit Compatibility

The face rig MAY additionally map to common ARKit-like expression concepts where useful, but Nexa's own semantic names remain canonical.

79. Core Expression Controls

Minimum:

Blink_L
Blink_R
EyeWide_L
EyeWide_R
EyeSquint_L
EyeSquint_R


BrowUp_L
BrowUp_R
BrowDown_L
BrowDown_R
BrowInnerUp


Smile_L
Smile_R
Frown_L
Frown_R


CheekRaise_L
CheekRaise_R


MouthWide
MouthNarrow
MouthPucker


JawOpen
JawLeft
JawRight
80. Required Viseme Shapes

At minimum:

Viseme_REST
Viseme_A
Viseme_E
Viseme_I
Viseme_O
Viseme_U
Viseme_MBP
Viseme_FV
Viseme_L
Viseme_WQ
Viseme_TH
Viseme_CHSH
Viseme_R
81. Viseme Neutrality

Viseme shapes SHALL be created from a neutral emotional base.

Emotion is layered separately.

82. Viseme Test Phrases

The asset pipeline SHOULD include standard speech tests containing varied phonemes.

Examples should include technical vocabulary and ordinary conversation.

83. Expression/Viseme Compatibility

Every major expression SHALL be tested while speaking.

Critical combinations:

soft smile + speech
focused + speech
serious + speech
encouraging + speech
84. Mouth Closure

MBP and rest states SHALL fully close the lips without visible gaps.

85. Jaw Open

Maximum speech jaw opening SHALL remain natural.

Extreme blendshape values SHOULD not be used during ordinary TTS.

86. Lip Roll and Compression

Optional advanced controls:

LipUpperIn
LipLowerIn
LipPress

These may improve speech realism.

87. Eye Expression Independence

Blinking SHALL work correctly while:

smiling
squinting
brow raised
skeptical
88. Expression Limits

Every facial control SHALL define a safe range.

Extreme combinations that break the mesh SHOULD be clamped.

89. Facial Combination Testing

Automated or scripted validation SHOULD sample combinations such as:

smile + blink
surprise + jaw open
skeptical + blink
focused + speech
90. Expression Presets

Expression presets SHOULD be authored as weighted control sets rather than baked single targets where possible.

Example:

focused:
  BrowDown_L: 0.18
  BrowDown_R: 0.18
  EyeSquint_L: 0.08
  EyeSquint_R: 0.08
  MouthNarrow: 0.04
91. High-Level Emotion Layer

Presets SHOULD be designed so runtime intensity can scale from:

0.0 → 1.0

without breaking deformation.

92. Blendshape Budget

The runtime face may contain dozens of morph targets.

A practical target SHOULD balance:

expressiveness
memory
GPU morph cost
portability

A baseline range of approximately 40–80 production facial targets is reasonable for a sophisticated real-time face.

93. Shape Key Storage

Blender source SHOULD use well-named shape keys that map cleanly into glTF morph targets.

94. Animation Production

Animation assets SHOULD be authored at a standard rate such as:

30 FPS or 60 FPS source

Runtime playback SHALL be time-based.

95. Required Core Animation Clips

Initial set:

Idle_Seated
Idle_Standing
Listening
Thinking
Speaking_Subtle
OpenHand_L
OpenHand_R
Point_L
Point_R
TwoHandExplain
ChinThink
AdjustGlasses
Nod
SmallNod
HeadShake
ThumbsUp
Attention
Typing
Celebrate_Subtle
Celebrate_Strong
96. Clip Root Policy

Desktop upper-body gestures SHOULD generally use:

in-place animation

not uncontrolled root motion.

97. Animation Loop Requirements

Loops MUST have clean start/end continuity.

Especially:

idle
listening
thinking
speaking subtle
typing
98. Animation Interruptibility

Each animation SHALL declare:

fully interruptible
interruptible after anticipation
non-interruptible critical window

Most tutoring gestures SHOULD be highly interruptible.

99. Animation Metadata

Example:

animation:
  id: point_right_01
  duration_ms: 1350
  loop: false
  interruptible: true
  affects:
    - chest
    - right_arm
    - right_hand
100. IK Compatibility

Pointing animations SHOULD be authored as IK-friendly base motions.

The runtime can then adapt the final hand location.

101. Hand Pose Separation

Finger pose SHOULD ideally be layered independently from major arm animation.

This enables:

pointing finger
open hand
typing pose

without unique full-body clips for every combination.

102. Seated Pose

The seated rig SHALL support:

neutral workstation posture
lean forward
lean back
typing
pointing
turning toward lesson canvas
103. Chair Interaction

If a visible chair is used, its dimensions and seat height SHOULD be standardized.

The chair itself SHOULD not become required for all render modes.

104. Standing Pose

Standing neutral SHALL support:

weight shift
pointing
open-hand explanation
presentation
105. Seated-to-Standing Transition

Not required for MVP.

If implemented later, it SHALL be a deliberate animation rather than teleportation.

106. Hair Collision

Hair physics SHOULD include basic collision against:

head
shoulders
upper torso

to reduce clipping.

107. Clothing Collision

Loose jacket elements MAY require torso/arm collision proxies.

108. Physics Collider Strategy

Use simple primitives:

sphere
capsule
plane

rather than high-cost mesh collision where practical.

109. Physics Stability

Physics SHALL be tested at:

30 FPS
60 FPS
variable frame rates

and should not explode under temporary frame drops.

110. Physics Reset

On teleport, scene reload, or major pose change, secondary physics SHALL support a controlled reset.

111. Material Naming

Recommended:

MAT_Skin
MAT_Eyes
MAT_Hair
MAT_Jacket
MAT_Shirt
MAT_GlassesFrame
MAT_GlassesLens
MAT_Accent
112. Texture Naming

Recommended:

T_Nexa_Head_BaseColor
T_Nexa_Head_Normal
T_Nexa_Head_Roughness
...
113. Mesh Naming

Recommended:

M_Nexa_Head
M_Nexa_Body
M_Nexa_Hair_Back
M_Nexa_Glasses
114. Armature Naming

Recommended:

RIG_Nexa
115. Collection Structure in Blender

Suggested:

NEXA
├── GEO
│   ├── BODY
│   ├── HEAD
│   ├── HAIR
│   ├── CLOTHING
│   └── ACCESSORIES
├── RIG
├── ANIM
├── COLLISION
├── LOD
└── EXPORT
116. Blender Source File Policy

The canonical .blend file SHOULD remain clean and production-oriented.

Avoid:

hundreds of unnamed objects
temporary sculpt duplicates
unused materials
unapplied transforms
orphan data
117. Transform Policy

Before export:

scale applied
rotation applied where appropriate
consistent origin

Armature transforms SHALL be handled carefully to preserve animation.

118. Modifier Policy

Export pipeline SHALL define which modifiers are:

applied
retained
converted
ignored

Examples:

Subdivision → usually applied/generated by LOD workflow
Mirror → applied
Armature → retained/exported
Corrective modifiers → baked if unsupported
119. Sculpt Source

High-resolution sculpt MAY remain in separate source collections/files.

It SHOULD not contaminate runtime export.

120. Bake Pipeline

Normal and other high-frequency detail SHOULD be baked from high-poly source to optimized runtime mesh where useful.

121. LOD Generation

LODs MAY be generated using:

manual retopology
controlled decimation
hybrid approach

The face and hands SHOULD receive manual review at every level.

122. LOD Bone Reduction

Lower LODs MAY reduce:

hair bones
finger bones
secondary twist bones

if renderer supports per-LOD skeleton variations.

Otherwise maintain the same skeleton for simplicity.

123. Facial LOD

Lower levels MAY reduce active facial targets.

Example:

L0: full face
L1: full conversational face
L2: reduced asymmetric controls
L3: core blink + mouth + simple expression
124. Texture Packaging

Runtime build SHOULD support texture compression appropriate to target platform.

Canonical source textures SHALL remain lossless or high-quality master files.

125. glTF Export Profile

The export profile SHALL preserve:

meshes
skin
skeleton
morph targets
animations
materials
textures
node hierarchy
126. GLB

A .glb package SHOULD be supported for convenient runtime distribution.

127. External Texture Option

Separate .gltf + external textures MAY also be supported during development and debugging.

128. Draco/Mesh Compression

Geometry compression MAY be used where renderer support is reliable.

It SHALL not be mandatory for canonical source validation.

129. VRM Export

VRM export SHOULD map:

humanoid bones
expressions
look-at semantics
spring bones

where possible.

Nexa-specific metadata may remain in extensions.

130. VRM Limitations

VRM SHALL not force us to discard capabilities required by Nexa.

Where Nexa semantics exceed VRM, custom extensions SHALL be permitted.

131. Nexa Metadata Extension

Example conceptual payload:

{
  "nexa": {
    "avatarVersion": "1.0.0",
    "rigProfile": "nexa-humanoid-1",
    "visemeProfile": "nexa-viseme-1",
    "behaviorProfile": "nexa-default"
  }
}
132. Asset Hashing

Exported assets SHALL be hashed.

GLB hash
texture hashes
animation pack hash
manifest hash

This supports reproducibility and cache invalidation.

133. Asset Build Manifest

Example:

build:
  avatar_version: 1.0.0
  rig_version: 1.0.0
  face_version: 1.0.0
  animation_version: 1.0.0
  source_commit: abc123
  glb_sha256: ...
134. Validation Tool

The Nexa asset pipeline SHOULD automatically check:

scale
axis
skeleton
required joints
morph targets
materials
texture references
animation clips
physics metadata
LOD references
manifest
135. Geometry Validation

Check for:

non-manifold geometry
degenerate triangles
invalid normals
duplicate vertices where harmful
zero-area faces
136. Material Validation

Check:

missing textures
unsupported shader requirements
invalid transparency setup
excessive material count
137. Rig Validation

Check:

required bones
hierarchy
axis conventions
rest pose
bone scale
duplicate names
138. Skin Validation

Check:

unweighted vertices
too many influences
invalid weights
stray bone influences
139. Facial Validation

Check:

required morph targets
range consistency
mouth closure
blink coverage
left/right symmetry where expected
140. Animation Validation

Check:

missing clips
invalid frame ranges
root motion
unexpected scale animation
non-looping loop clips
141. Naming Validation

The asset compiler SHALL flag nonconforming names.

142. Performance Validation

Asset gate SHOULD report:

triangles by LOD
vertices
materials
draw-call estimate
morph target count
bones
texture memory
animation memory
143. Visual Acceptance Review

Automated checks are insufficient.

Each release SHALL undergo human review using standard camera and lighting.

144. Canonical Review Views

At minimum:

front neutral
3/4 neutral
profile
smile
focused
speaking
point left
point right
typing
seated
standing
145. Face Match Review

The 3D face SHALL be compared against the approved reference using overlays where practical.

146. Hair Match Review

Evaluate:

silhouette
bangs
side profile
back volume
accent strands
147. Clothing Match Review

Evaluate:

silhouette
color blocking
accent placement
technology details
148. Expression Quality Gate

Expressions SHALL be evaluated for:

readability
naturalness
character consistency
lack of distortion
149. Speech Quality Gate

The face MUST remain convincing during:

normal speech
smiling speech
serious speech
questioning speech
150. Pointing Quality Gate

Pointing must read clearly from desktop tutor framing.

151. Typing Quality Gate

Hands SHALL plausibly reach and operate a virtual keyboard if typing animation is enabled.

152. Glasses Quality Gate

Glasses SHALL:

stay aligned
not clip into face
not hide eye animation
not flicker through transparency artifacts
153. Runtime Test Sentences

The package SHOULD include standard spoken phrases for animation review.

Examples:

"Let's look at the SYN-ACK packet."
"Run cargo test and tell me what changed."
"Good. Now explain why that worked."
"That result doesn't match what we expected."

These exercise both technical pronunciation and facial performance.

154. Asset Source Control

Canonical production assets SHOULD be tracked in version control appropriate for large binaries.

The repository strategy MAY use:

Git LFS
dedicated asset repository
artifact store

while code remains in the main Rust repository.

155. Source Asset Separation

Recommended:

nexa/
├── code/
└── assets/
    └── avatar/

or separate repositories if binary scale warrants it.

156. Artifact Publication

Built GLB/VRM packages SHOULD be published as versioned artifacts rather than requiring every developer to run Blender.

157. Blender Version Pinning

The production pipeline SHOULD pin or record the Blender version used for official builds.

This improves reproducibility.

158. Add-On Versioning

Any required Blender add-ons SHALL also be versioned.

Avoid hidden workstation-only dependencies.

159. Automated Blender Build

The pipeline SHOULD eventually support:

blender --background ...

for automated validation/export.

160. CI Asset Validation

CI MAY run:

schema checks
glTF validation
bone validation
manifest validation
texture reference validation
animation presence checks

Full visual review remains manual.

161. Golden Asset Package

Once Nexa reaches an acceptable production baseline, a golden package SHOULD be frozen for regression testing.

162. Asset Regression

New packages SHOULD be compared for:

bone count
morph target changes
material changes
mesh metrics
animation catalog
manifest compatibility
163. Compatibility Policy

Minor avatar revisions SHOULD preserve runtime contracts.

Breaking rig changes require a major rig version.

164. Semantic Versioning

Recommended:

avatar:      1.2.0
rig:         1.0.0
face:        1.1.0
animations:  1.3.2
165. Rig Breaking Change

Examples:

renaming canonical bone
removing required morph target
changing rest-pose orientation
changing coordinate convention

These require major-version consideration.

166. Non-Breaking Change

Examples:

better texture
extra gesture
improved corrective shape
additional LOD
167. Alternate Outfits

Alternate clothing SHOULD not require a new canonical skeleton.

168. Alternate Hairstyles

Alternate hairstyles SHALL remain compatible with:

head scale
physics interface
glasses
facial visibility
169. Future Character Customization

Potential later options:

outfits
accessories
scene themes
hair variants

Core face identity SHOULD remain stable for canonical Nexa.

170. Production Milestone A — Reference Lock

Deliverables:

turnaround
face sheet
gesture sheet
palette
material reference
height/proportion spec

No modeling should be considered canonical until this is approved.

171. Production Milestone B — Graybox Character

Deliverables:

head
body
hair blockout
clothing blockout
glasses
basic proportions

No final textures required.

172. Production Milestone C — Production Topology

Deliverables:

final L0 topology
face loops
hands
mouth cavity
eyes
clothing
hair meshes
173. Production Milestone D — Rigged Neutral Character

Deliverables:

humanoid skeleton
finger rig
jaw
eyes
skin weights
seated/standing validation
174. Production Milestone E — Talking Head

Deliverables:

facial controls
expressions
visemes
blink
gaze
jaw
first TTS lip-sync test

At this stage Nexa should already be usable as a 3D conversational tutor.

175. Production Milestone F — Teaching Upper Body

Deliverables:

arms
hands
point
open hand
nod
thinking
typing
IK
176. Production Milestone G — Final Materials

Deliverables:

skin
hair
eyes
clothing
glasses
emissives
PBR tuning
177. Production Milestone H — Physics & Polish

Deliverables:

hair physics
clothing secondary motion
corrective shapes
micro-expression tuning
animation polish
178. Production Milestone I — LOD & Runtime Packaging

Deliverables:

L0–L3
optimized textures
compiled GLB
VRM mapping
manifest
validation report
179. First Runnable 3D Prototype Gate

We SHOULD NOT wait until all milestones are complete.

The first runtime prototype should begin as soon as Milestone E exists:

head
upper torso
eyes
blink
jaw
visemes
basic expression
gaze

This allows the Rust runtime to progress in parallel with asset production.

180. Parallel Software Track

While the character is being built:

Asset Team                     Runtime Team


Reference                      GLB loader
Model                          renderer
Rig                ↔           skeleton loader
Face               ↔           morph targets
Visemes            ↔           speech sync
Gaze               ↔           gaze solver
Hands/arms          ↔           IK
Animations          ↔           animation graph

Neither side should wait for the other to finish everything.

181. Minimal Rust 3D Viewer

The first software tool SHOULD support:

load GLB
render Nexa
orbit/debug camera
play animation
inspect skeleton
set morph-target weights
rotate eyes/head

This becomes the technical-art validation environment.

182. Morph Target Inspector

The viewer SHALL allow manual control of:

all facial targets
visemes
expression presets
183. Rig Inspector

Viewer SHOULD display:

bone hierarchy
bone positions
IK targets
joint limits
184. Animation Inspector

Viewer SHOULD:

select clip
play/pause
scrub
set speed
show blend weights
185. Lip-Sync Inspector

Viewer SHOULD accept:

audio
viseme timeline

and replay synchronized speech.

186. Gaze Inspector

Viewer SHOULD allow clicking a point in the scene and making Nexa look toward it.

This gives us the first visible evidence of intelligent presence.

187. Pointing Inspector

Later:

select world target
choose left/right/auto
solve IK
188. Canonical Character Quality Philosophy

Our quality hierarchy is:

1. Identity
2. Face
3. Eyes
4. Speech
5. Gaze
6. Hands
7. Gesture
8. Materials
9. Hair/clothing physics
10. Environmental spectacle

That order should guide every production tradeoff.

189. Things We Should Explicitly Avoid

Avoid:

generic marketplace avatar appearance
overly sexualized design
extreme anime proportions
hyperactive idle animation
poorly deforming shoulders
flat painted eyes
minimal mouth topology
one-shape-fits-all lip sync
engine-locked canonical assets
excessively complex hair simulation before face quality
190. Production Asset Acceptance Gate

NEXA-3D-ART-001 is satisfied when the canonical Nexa package can demonstrate:

recognizable visual identity;
correct physical scale;
valid humanoid skeleton;
independent eye control;
natural blink;
expressive brow and cheek deformation;
jaw articulation;
all canonical visemes;
neutral-to-expression blending;
speech while smiling/focused/serious;
articulated fingers;
seated and standing poses;
point-left and point-right capability;
gaze targeting;
acceptable shoulder/elbow/wrist deformation;
glasses remaining stable during face motion;
stable hair/clothing secondary motion;
LOD availability;
successful GLB export;
successful validation by the Nexa runtime toolchain.
191. Architectural Result

The 3D production chain is now concrete:

                  Canonical Nexa Design
                          │
                          ▼
                 NEXA-3D-ART-001
                  Production Asset
                          │
                          ▼
                   Nexa GLB / VRM
                          │
                          ▼
                    NEXA-3D-001
                      Runtime
                          │
                ┌─────────┼─────────┐
                ▼         ▼         ▼
              Face       IK      Animation
                │         │         │
                └─────────┼─────────┘
                          ▼
                     NEXA-NBP
                          │
                          ▼
                     Tutor Stack

We are now at the point where the next work can stop being conceptual and start producing the actual 3D character and viewer.

192. Next Move

I recommend the next artifact be NEXA-3D-REF-001 — Canonical Nexa 3D Reference & Turnaround Specification, followed immediately by generating the actual turnaround images: front, 3/4, side, back, facial close-up, and expression sheet.

Those images become the locked modeling references for Blender. After that, the next software artifact should be NEXA-3D-RUNTIME-001, defining the minimal Rust + wgpu viewer required to load the first GLB and drive bones, morph targets, gaze, and lip sync.

That puts us one step away from actually constructing 3D Nexa rather than just continuing the architecture.
