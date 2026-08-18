NEXA-3D-001 — 3D Character, Humanoid Rig, Facial Animation & Rendering Architecture v1.0

Specification ID: NEXA-3D-001
System: Nexa AI Training Tutor
Version: 1.0
Status: Baseline Draft
Depends On: NEXA-CBS-001, NEXA-DOM-001, NEXA-NBP-001, NEXA-EVT-001, NEXA-SPCH-001, NEXA-AVTR-001, NEXA-ORCH-001
Purpose: Define Nexa’s production-grade 3D embodiment, including canonical asset format, humanoid skeleton, facial rig, blendshapes, visemes, gaze, inverse kinematics, gestures, animation graphs, physics, materials, lighting, camera behavior, render abstraction, VRM/glTF compatibility, performance budgets, XR extensibility, and migration from 2D/2.5D to full 3D.

1. Purpose

The 3D subsystem answers:

“How does Nexa become a fully rigged, expressive, renderer-independent 3D character that can speak, gesture, look at content, interact with virtual environments, and preserve the same semantic behavior architecture already defined?”

The core principle remains unchanged:

Tutor Intent
    ↓
BehaviorIntent
    ↓
NEXA-NBP-001
    ↓
3D Behavior Engine
    ↓
Rig / IK / Face / Animation
    ↓
Renderer Adapter
    ↓
Animated 3D Nexa

The Tutor Engine SHALL NOT know anything about bones, blendshapes, shaders, IK chains, or renderer APIs.

2. 3D Architectural Position
                    NEXA-TUTOR-001
                           │
                           ▼
                    BehaviorIntent
                           │
                           ▼
                     NEXA-NBP-001
                           │
                           ▼
                 ┌──────────────────┐
                 │  NEXA-3D-001     │
                 │                  │
                 │ behavior graph   │
                 │ humanoid rig     │
                 │ facial rig       │
                 │ gaze             │
                 │ IK               │
                 │ gestures         │
                 │ physics          │
                 └────────┬─────────┘
                          │
                 Renderer Adapter API
                  /        |        \
                 ▼         ▼         ▼
              Native     Godot     Unity/etc.
             Rust/WGPU
3. Core Responsibilities

The 3D subsystem SHALL own or coordinate:

canonical 3D character package;
humanoid skeleton;
facial topology;
facial controls;
blendshapes;
viseme mapping;
jaw and tongue animation;
hand and finger rigs;
inverse kinematics;
procedural gaze;
head/neck tracking;
posture;
gesture animation;
additive animation;
animation state graphs;
motion blending;
retargeting;
root motion policy;
hair simulation;
clothing simulation;
accessory simulation;
physically based materials;
lighting profiles;
camera framing;
level of detail;
render quality scaling;
avatar/environment interaction;
renderer abstraction;
VRM/glTF import/export;
XR compatibility.
4. Explicit Non-Responsibilities

The 3D subsystem SHALL NOT determine:

learner mastery;
pedagogy;
spoken content;
assessment policy;
tool permissions;
knowledge retrieval;
curriculum sequencing.

It performs semantic behavior already determined elsewhere.

5. Canonical Asset Strategy

The preferred strategy is:

glTF 2.x
   +
VRM-compatible humanoid metadata
   +
Nexa-specific extensions

This gives us:

broad tooling support;
standardized mesh/material representation;
humanoid avatar semantics;
portable asset pipelines;
future engine independence.
6. Why Not Engine-Native Assets as Canonical

Nexa SHALL NOT make:

Unity prefab
Unreal asset
Godot scene

the canonical source of truth.

Those may be deployment artifacts.

Canonical source should remain engine-neutral.

7. Canonical Package

Recommended structure:

avatars/
└── nexa-3d/
    ├── manifest.yaml
    ├── model/
    │   ├── nexa.glb
    │   └── source/
    ├── textures/
    ├── materials/
    ├── skeleton/
    ├── facial/
    ├── blendshapes/
    ├── visemes/
    ├── animations/
    ├── gestures/
    ├── poses/
    ├── physics/
    ├── lighting/
    ├── cameras/
    ├── lod/
    ├── mappings/
    └── tests/
8. Asset Manifest
avatar:
  id: nexa.3d
  version: 1.0.0
  character: nexa


formats:
  canonical: gltf
  vrm_compatible: true


rig:
  humanoid: true
  facial: true
  fingers: true
  jaw: true
  tongue: optional


render:
  pbr: true
  transparent_glasses: true


capabilities:
  full_body: true
  seated_pose: true
  standing_pose: true
  gaze_ik: true
  hand_ik: true
  pointing_ik: true
  canonical_visemes: true
9. Humanoid Skeleton

The canonical rig SHOULD use a conventional humanoid hierarchy.

Root
└── Hips
    ├── Spine
    │   └── Chest
    │       └── UpperChest
    │           ├── Neck
    │           │   └── Head
    │           ├── LeftShoulder
    │           │   └── LeftUpperArm
    │           │       └── LeftLowerArm
    │           │           └── LeftHand
    │           └── RightShoulder
    │               └── RightUpperArm
    │                   └── RightLowerArm
    │                       └── RightHand
    ├── LeftUpperLeg
    │   └── LeftLowerLeg
    │       └── LeftFoot
    └── RightUpperLeg
        └── RightLowerLeg
            └── RightFoot
10. Required Humanoid Bones

At minimum:

hips
spine
chest
neck
head


left/right shoulder
left/right upper arm
left/right lower arm
left/right hand


left/right upper leg
left/right lower leg
left/right foot

For high-quality animation, additional twist and helper bones SHOULD be supported.

11. Finger Skeleton

Each hand SHOULD support:

thumb 0–3
index 0–3
middle 0–3
ring 0–3
little 0–3

Finger articulation is important for:

pointing;
typing;
open-hand explanations;
grabbing;
future XR.
12. Hand Pose Library

Baseline semantic hand poses:

relaxed
open
point
pinch
fist
typing
thumbs_up
two_hand_explain

These SHOULD be reusable across gestures.

13. Facial Rig Strategy

Nexa SHOULD use a hybrid face rig:

blendshapes
   +
joint controls where useful

Blendshapes are preferred for:

expressions;
visemes;
brows;
cheeks;
eyelids.

Bones/joints may assist:

jaw;
eyes;
tongue;
glasses alignment.
14. Required Facial Controls

At minimum:

blink_left
blink_right


eye_wide_left
eye_wide_right


brow_up_left
brow_up_right
brow_down_left
brow_down_right


smile_left
smile_right
frown_left
frown_right


jaw_open
jaw_left
jaw_right


mouth_pucker
mouth_wide
mouth_narrow


cheek_raise_left
cheek_raise_right
15. Canonical Facial Control Layer

The runtime SHOULD expose semantic controls:

pub struct FaceControlState {
    pub eyes: EyeControlState,
    pub brows: BrowControlState,
    pub mouth: MouthControlState,
    pub jaw: JawControlState,
    pub expression: ExpressionBlendState,
}

Renderer-specific blendshape names SHALL remain behind adapters.

16. Expression Blendshapes

Recommended baseline expressions:

neutral
focused
curious
thinking
encouraging
skeptical
concerned
serious
surprised
confused
soft_smile
celebrating
corrective
17. Expression Composition

Expressions SHOULD be combinable.

Example:

focused
+
slight smile
+
raised brow

The system SHOULD avoid requiring one monolithic blendshape for every emotional combination.

18. Facial Asymmetry

Subtle asymmetry SHOULD be supported.

Examples:

single-brow raise
slight one-sided smile
asymmetric skepticism

These significantly improve naturalness.

19. Eye Rig

Eyes SHOULD be independently rotatable.

The runtime SHALL support:

left eye target
right eye target
vergence
blink
squint
eye openness
20. Gaze Target Model
pub enum ThreeDGazeTarget {
    Camera,
    Student,
    WorldPoint(Vec3),
    Object(EntityId),
    ScreenObject(String),
    Neutral,
}
21. Gaze Architecture
semantic target
     ↓
target resolver
     ↓
world-space point
     ↓
eye IK
     ↓
head follow
     ↓
neck/spine follow if needed
22. Eye-Head Coordination

For small gaze shifts:

eyes dominate

For medium shifts:

eyes + head

For large shifts:

eyes + head + upper torso
23. Gaze Constraints

The runtime SHALL constrain:

eye rotation
head yaw
head pitch
neck rotation

to avoid unnatural poses.

24. Eye Contact

When addressing the learner directly:

gaze target = camera/student

but micro-variation SHOULD prevent lifeless staring.

25. Saccades

Small saccades SHOULD be procedural and bounded.

They may be reduced during:

direct questioning;
warnings;
precise demonstrations.
26. Blink System

3D blinking SHOULD use facial controls, not whole-face animation clips.

Blink timing SHOULD remain pseudo-random but deterministic in test mode.

27. Jaw Rig

Jaw motion SHOULD be separately controllable from mouth blendshapes.

This improves:

lip sync;
natural vowels;
mouth opening;
speech realism.
28. Tongue

Tongue rigging is optional for MVP.

It MAY improve close-up speech realism but is lower priority than:

jaw;
visemes;
eye quality;
expression blending.
29. Canonical Visemes

Reuse the speech architecture's canonical viseme set.

REST
A
E
I
O
U
MBP
FV
L
WQ
TH
CHSH
R
30. 3D Viseme Mapping

Each viseme SHOULD map to:

mouth blendshapes
jaw open
lip compression
lip rounding
tongue position where available
31. Coarticulation

The 3D runtime SHOULD support:

previous viseme
current viseme
next viseme

blending.

This is especially important for close-up facial animation.

32. Phoneme Extension

Future higher-quality speech MAY use phoneme-level input rather than canonical visemes.

The runtime contract SHOULD permit both.

pub enum LipSyncInput {
    Viseme(VisemeEvent),
    Phoneme(PhonemeEvent),
}
33. Speech Synchronization

Audio playback SHALL remain the timing master.

3D face animation MUST NOT free-run independently.

34. Head Motion During Speech

Procedural head motion MAY be generated from:

phrase boundaries;
emphasis;
question intonation;
speech style.

It SHOULD remain subtle.

35. Body Rig

The body SHALL support:

hips
spine
chest
shoulders
arms
hands
legs
feet

plus optional helper bones.

36. Seated and Standing Modes

Nexa SHOULD support at least:

pub enum BodyMode {
    Seated,
    Standing,
}

The initial desktop tutor MAY favor seated presentation.

37. Seated Tutor Mode

Advantages:

fits workstation aesthetic;
reduces full-body animation requirements;
naturally supports terminal/code tutoring;
simplifies framing.
38. Standing Instructor Mode

Useful for:

presentation mode;
full-screen teaching;
large diagrams;
future XR.
39. Posture System
pub enum PostureIntent {
    Neutral,
    Attentive,
    LeanForward,
    Relaxed,
    Serious,
    Demonstrating,
}
40. Inverse Kinematics

The runtime SHOULD support IK for:

head
eyes
hands
feet

Hand IK is especially important for pointing at arbitrary content.

41. IK Architecture
semantic target
      ↓
world-space transform
      ↓
IK solver
      ↓
joint targets
      ↓
animation blend
42. Pointing IK

Nexa SHOULD not rely only on prerecorded left/right pointing clips.

Instead:

gesture intent = point
target = object
      ↓
select base animation
      ↓
solve arm IK toward target
      ↓
adjust shoulder/body

This dramatically improves flexibility.

43. Pointing Hand

The runtime SHOULD choose:

left
right

based on:

target side;
current pose;
screen layout;
dominant-hand bias;
occlusion.
44. IK Reachability

Targets outside comfortable reach SHALL degrade to:

gaze
+
directional gesture

rather than distort the body.

45. Hand IK

Future use cases include:

touch panel
grab virtual object
type on keyboard
hold tablet
manipulate hologram
46. Foot IK

Useful for standing modes to maintain contact with uneven terrain.

Not required for initial seated desktop presentation.

47. Animation Graph

The 3D runtime SHOULD use layered animation graphs.

Conceptually:

Base locomotion/posture
        +
Behavior state
        +
Upper-body gesture
        +
Additive head motion
        +
Gaze IK
        +
Facial expression
        +
Lip sync
        +
Physics
48. Base State Graph
Idle
 ↕
Attentive
 ↕
Listening
 ↕
Thinking
 ↕
Speaking
 ↕
Waiting

Higher-level states such as:

Explaining
Correcting
Warning
Celebrating

may modify posture/expression/gesture layers.

49. Animation Layers

Recommended order:

0. base pose
1. locomotion / seated loop
2. behavior state
3. upper-body gesture
4. hand pose
5. head additive
6. gaze IK
7. facial expression
8. lip sync
9. secondary physics
50. Layer Masks

A gesture such as pointing SHOULD usually affect:

chest
shoulder
arm
hand

without overwriting:

legs
mouth
eyes
51. Additive Animation

Additive clips SHOULD support:

nod;
head tilt;
small lean;
shoulder emphasis;
speech gestures.
52. Gesture Catalog

Baseline 3D gestures:

nod
small_nod
shake_head
head_tilt
point
open_hand
two_hand_explain
thinking_chin
adjust_glasses
shrug
lean_forward
lean_back
thumbs_up
celebrate
attention
typing
53. Gesture Variants

Every commonly repeated gesture SHOULD support multiple variants.

Example:

point_soft
point_direct
point_cross_body
point_near
54. Gesture Retargeting

Animation assets SHOULD be retargetable onto the canonical humanoid skeleton.

The canonical Nexa rig SHALL define the retarget reference pose.

55. Reference Pose

Recommended canonical pose:

T-pose or A-pose

with clearly documented joint orientation.

A-pose is generally preferable for more natural shoulder deformation.

56. Retargeting Contract
pub trait MotionRetargeter {
    fn retarget(
        &self,
        source: &Skeleton,
        target: &Skeleton,
        clip: &AnimationClip,
    ) -> RetargetResult<AnimationClip>;
}
57. Root Motion

Default desktop tutor behavior SHOULD use:

root motion disabled

because Nexa is generally stationary.

Standing/XR modes MAY enable root motion.

58. Procedural Motion

The runtime SHOULD generate procedurally:

blink
eye tracking
head follow
breathing
micro-posture
subtle hand settling
59. Motion Matching

Motion matching is NOT required for v1.

It may become relevant for future free-roaming 3D environments.

60. Breathing

3D breathing MAY animate:

chest
upper spine
shoulders

with extremely low amplitude.

61. Idle Micro-Motion

Procedural idle should include:

weight shifts
small hand motion
head correction
eye saccades
breathing
blinks

without appearing restless.

62. Hair Model

Nexa's hair is a major identity feature.

The asset SHOULD include separated hair components suitable for secondary simulation.

63. Hair Physics

Possible implementations:

spring bones
verlet chains
cloth strips
engine-specific hair simulation

The canonical package SHOULD describe physics semantically rather than mandate one solver.

64. Hair Quality Levels
Low:
minimal spring bones


Medium:
multiple chains


High:
more strands + collision


Ultra:
advanced simulation
65. Clothing

Nexa's cyber/technical clothing SHOULD use a combination of:

skinned mesh
secondary bones
optional cloth simulation
66. Cloth Simulation Priority

Cloth quality SHALL remain below:

face
gaze
speech
gesture

in performance priority.

67. Glasses

The glasses SHOULD be treated as a rigged accessory attached to the head.

They MAY support:

material reflections;
subtle emissive UI effects;
adjustment gesture;
future AR-overlay animation.
68. Glasses Transparency

Transparent materials SHALL be implemented with care to avoid:

sorting artifacts;
eye obscuration;
excessive reflections.
69. Materials

The canonical model SHOULD use PBR materials.

Recommended channels:

base color
normal
metallic
roughness
emissive
occlusion
70. Stylization

Nexa need not be photorealistic.

The preferred look is:

stylized realistic

or:

high-end anime/cyber semi-realism

with believable facial deformation and controlled materials.

71. Material Identity

The 3D model SHOULD preserve:

dark clothing
purple accents
cyan/magenta technology highlights
dark hair
signature glasses

from the canonical 2D character design.

72. Skin Shading

Skin SHOULD use an appropriate soft shading model.

Subsurface scattering MAY be supported in high-quality renderers, but the canonical asset SHALL not require it.

73. Eye Shading

Eyes deserve dedicated material treatment.

Important properties include:

corneal highlight;
iris depth;
pupil response;
sclera shading.

Poor eye rendering can make otherwise good characters appear lifeless.

74. Eye Highlight

A consistent catchlight SHOULD be maintained under normal lighting.

75. Lighting Profiles

The 3D runtime SHOULD define semantic lighting presets.

pub enum LightingProfile {
    TutorDesk,
    NeutralStudio,
    Presentation,
    DarkCyber,
    BrightClassroom,
}
76. Default Tutor Lighting

Recommended:

soft key light
subtle fill
cool purple/cyan rim
low-intensity background practicals

This preserves Nexa's cyber aesthetic without overpowering instructional content.

77. Lighting Independence

Behavior intent SHALL NOT control raw light intensities.

The presentation layer may select lighting profiles.

78. Camera Model

The runtime SHOULD provide semantic camera modes.

pub enum AvatarCameraMode {
    Portrait,
    Medium,
    WaistUp,
    FullBody,
    Presentation,
    XR,
}
79. Desktop Default

Recommended:

medium / waist-up

because:

face remains readable;
hands remain visible;
gestures are meaningful;
desktop space remains manageable.
80. Camera Framing

The camera controller SHOULD preserve:

face visibility
hands when needed
content visibility
81. Gesture-Aware Camera

In large presentation mode, the camera MAY widen slightly during major gestures.

This SHOULD remain subtle.

82. Camera Shall Not Distract

Frequent cinematic cuts are inappropriate for normal tutoring.

The camera SHOULD normally remain stable.

83. Environment Interaction

The 3D runtime SHOULD support a world abstraction.

pub trait InteractionWorld {
    fn resolve_target(
        &self,
        id: &InteractionTargetId,
    ) -> Option<Transform>;
}
84. Interaction Targets

Examples:

lesson diagram
terminal panel
virtual keyboard
holographic packet
whiteboard
server rack
lab object
85. Screen-to-World Mapping

Desktop UI elements MAY be represented as virtual world targets.

screen object
    ↓
presentation adapter
    ↓
3D anchor
    ↓
gaze/point target
86. Hybrid 2D/3D Workspace

One important deployment model is:

3D Nexa
   +
2D UI panels

This is likely ideal for the initial tutor workspace.

87. Example Layout
┌─────────────────────────────────────────────────────┐
│                                                     │
│  3D Nexa              Lesson / Diagram / Code       │
│  waist-up             Canvas                        │
│                                                     │
│                       Terminal / Lab                 │
│                                                     │
└─────────────────────────────────────────────────────┘

Nexa can gaze and point toward the 2D panels through mapped anchors.

88. Future Full 3D Classroom

Later:

virtual room
    │
    ├── Nexa
    ├── virtual displays
    ├── whiteboard
    ├── network equipment
    └── learner viewpoint

The same NBP semantics remain valid.

89. XR Readiness

The architecture SHOULD preserve compatibility with:

VR
AR
mixed reality

without requiring XR in v1.

90. XR Interaction Extensions

Future capabilities may include:

hand tracking
controller pointing
spatial audio
shared anchors
room-scale movement
object manipulation
91. Personal Space

Future XR versions SHALL define a comfortable minimum distance between Nexa and the learner viewpoint.

92. LOD System

Nexa SHOULD support multiple geometry/material detail levels.

pub enum AvatarLod {
    L0,
    L1,
    L2,
    L3,
}
93. Suggested LOD Roles
L0:
close-up


L1:
desktop normal


L2:
small panel


L3:
distant/background
94. Face Preservation

Even lower LODs SHOULD preserve facial readability when Nexa is still a conversational character.

95. Texture LOD

Texture resolution MAY scale independently from mesh LOD.

96. Render Quality
pub enum ThreeDQuality {
    Low,
    Medium,
    High,
    Ultra,
    Adaptive,
}
97. Quality Degradation Order

When performance is poor, reduce in this order:

post-processing
cloth complexity
hair complexity
shadow resolution
reflection quality
mesh LOD

Preserve as long as possible:

face
eyes
mouth
gaze
core gesture
98. Frame Budget

Target:

60 FPS desktop

A robust runtime SHOULD remain usable at:

30 FPS

on lower-end hardware.

99. Frame-Time Budget

At 60 FPS:

~16.67 ms per frame

The total includes:

simulation;
animation;
physics;
skinning;
rendering;
UI compositing.
100. GPU Independence

The architecture SHALL not assume NVIDIA-specific features.

It SHOULD support:

AMD
NVIDIA
Intel
Apple

through renderer abstraction where possible.

101. Native Rust Renderer

A native Rust renderer is a viable strategic option.

Possible conceptual stack:

Rust
 ↓
wgpu
 ↓
Vulkan / DX12 / Metal / WebGPU

This would align well with the broader Rust-based Nexa architecture.

102. Renderer Contract
pub trait ThreeDRenderer {
    fn initialize(
        &mut self,
        config: RendererConfig,
    ) -> RenderResult<()>;


    fn load_avatar(
        &mut self,
        package: &ThreeDAvatarPackage,
    ) -> RenderResult<AvatarRenderHandle>;


    fn render(
        &mut self,
        frame: &ThreeDFrame,
    ) -> RenderResult<()>;
}
103. Renderer Adapter Strategy

Potential adapters:

native-wgpu
godot
unity
unreal
webgpu-browser

Only one needs to exist initially.

104. Recommended Initial Renderer Direction

For architectural alignment, I would prioritize:

Rust + wgpu

for the first serious native prototype.

Reasons:

keeps the runtime in Rust;
supports Windows/Linux/macOS;
maps to major GPU APIs;
avoids engine lock-in;
integrates well with the rest of the platform.

A Godot adapter could be useful later for faster content production.

105. Render Scene
pub struct ThreeDScene {
    pub avatar: AvatarRenderHandle,
    pub camera: CameraState,
    pub lighting: LightingState,
    pub environment: EnvironmentRenderState,
}
106. Avatar Frame
pub struct ThreeDFrame {
    pub skeleton: SkeletonPose,
    pub face: FaceControlState,
    pub materials: MaterialOverrides,
    pub physics: PhysicsState,
}
107. Skeleton Pose
pub struct SkeletonPose {
    pub joints: Vec<JointTransform>,
}
108. Renderer Decoupling

The behavior engine SHOULD compute semantic pose targets.

The renderer MAY implement:

GPU skinning;
morph targets;
materials;
lighting;
final rasterization.
109. Animation Asset Format

Animation clips SHOULD be importable from standard formats.

Preferred:

glTF animations
FBX during authoring only

FBX SHOULD NOT be required as the runtime canonical format.

110. Source Authoring Formats

Artists may work in:

Blender
Maya
3ds Max
other DCC tools

Export pipeline compiles to canonical package.

111. Blender Pipeline

A likely practical production pipeline is:

concept art
    ↓
Blender modeling
    ↓
retopology
    ↓
UV
    ↓
texturing
    ↓
rigging
    ↓
shape keys
    ↓
animations
    ↓
glTF/VRM export
    ↓
Nexa validation
112. Character Modeling Stages

Recommended:

1. turnarounds
2. base sculpt
3. topology
4. face topology
5. clothing
6. hair
7. UVs
8. materials
9. skeleton
10. weights
11. face rig
12. visemes
13. gestures
14. LODs
113. Character Turnaround

Before modeling, we SHOULD create:

front
3/4
side
back

reference views.

This is a prerequisite for a consistent 3D model.

114. Facial Reference Sheet

We SHOULD also create:

neutral
smile
curious
focused
thinking
skeptical
concerned
surprised
serious
corrective

reference expressions.

115. Gesture Reference Sheet

Needed for animation design:

point left
point right
open hand
two-hand explain
chin think
adjust glasses
nod
thumbs up
typing
warning
116. Character Proportion Specification

The 3D model SHOULD preserve the visual proportions established by the canonical concept rather than relying on generic avatar proportions.

117. Face Topology

Face topology SHALL prioritize deformation quality around:

eyes
brows
mouth
nasolabial region
jaw
cheeks
118. Mouth Topology

Adequate edge loops around lips are mandatory for credible visemes.

119. Eye Geometry

Eyes SHOULD be separate geometry with independent rotation.

120. Teeth

Upper and lower teeth MAY be separate meshes.

121. Hair Modeling

For initial stylized Nexa, hair SHOULD likely use:

mesh cards / stylized hair masses

rather than strand-level simulation.

This balances quality and performance.

122. Retopology

The production mesh SHOULD be optimized for deformation and real-time rendering.

High-resolution sculpt geometry SHALL not be used directly at runtime.

123. Skinning

Weights SHOULD be validated for:

shoulders
elbows
wrists
hips
knees
neck
jaw

These are common deformation failure zones.

124. Twist Bones

Upper/lower limb twist bones are recommended for realistic forearm and upper-arm deformation.

125. Corrective Blendshapes

Corrective shapes MAY be used for:

shoulders;
elbows;
wrists;
extreme facial poses.
126. Animation Retarget Profile

The avatar package SHOULD contain a retarget profile mapping canonical joints.

127. Rig Metadata
pub struct HumanoidRigMetadata {
    pub joints: HashMap<HumanoidJoint, JointId>,
    pub rest_pose: RestPose,
    pub axis_conventions: AxisConvention,
}
128. Coordinate System

The canonical package SHALL explicitly define:

up axis
forward axis
handedness
units

to avoid engine-specific ambiguity.

129. Recommended Convention

A reasonable canonical convention:

Y-up
meters
right-handed

Adapters may convert as required.

130. Scale

Nexa's canonical height SHOULD be defined in meters, even if she is normally rendered waist-up.

131. Behavior-to-3D Mapping

Example:

NBP:
state = explaining
gesture = point
target = tcp.syn_ack
emotion = focused

3D engine resolves:

base state = explaining
face = focused
gaze target = tcp.syn_ack
right arm base gesture = point
IK target = mapped content anchor
132. Behavior Resolver
pub trait ThreeDBehaviorResolver {
    fn resolve(
        &self,
        command: &NbpBehaviorCommand,
        world: &dyn InteractionWorld,
    ) -> ThreeDBehaviorPlan;
}
133. 3D Behavior Plan
pub struct ThreeDBehaviorPlan {
    pub state: AvatarBehaviorState,
    pub pose: PostureIntent,
    pub expression: ExpressionState,
    pub gaze: ThreeDGazePlan,
    pub gesture: GesturePlan,
    pub speech: Option<SpeechSyncPlan>,
}
134. Speech Plan Integration

SpeechSyncPlan SHOULD reference:

speech_id
viseme stream
audio clock
phrase boundaries
135. Gesture Timing

Gesture timing SHOULD allow semantic markers:

before_phrase
on_keyword
after_phrase
during_sentence
136. Speech Emphasis Hooks

Future TutorResponse MAY tag:

emphasis anchors

which can trigger:

head nod;
hand emphasis;
brow movement.
137. Animation Timing Event
pub struct BehaviorTimingCue {
    pub offset: Duration,
    pub cue: BehaviorCueType,
}
138. Behavior Arbitration

The 3D system SHALL reuse the arbitration principles from NEXA-AVTR-001.

Priority example:

system warning
student interruption
explicit gesture
speech gesture
state loop
idle
139. Interruption

When the student interrupts:

speech cancelled
      ↓
lip sync stops
      ↓
gesture exits
      ↓
head turns toward learner
      ↓
Nexa enters listening pose

The transition SHOULD be continuous, not snapped.

140. Animation Exit Strategy

Every gesture SHOULD define:

interruptible?
blend-out duration
safe exit pose
141. Pose Recovery

The runtime SHALL know the current pose.

New animations SHOULD blend from current pose rather than assuming a neutral starting frame.

142. Behavior Replay

NBP + audio + viseme streams SHOULD be replayable through the 3D runtime.

This gives us deterministic animation testing.

143. Headless 3D Mode

The behavior/animation solver SHOULD be testable without actual GPU rendering.

NBP
  ↓
pose graph
  ↓
joint transforms
144. Headless Assertions

Tests can verify:

right hand target reached
gaze target correct
mouth returned to rest
gesture cancelled
state transitioned
145. Skeleton Validation

Asset validation SHALL confirm:

required joints exist
joint hierarchy valid
bone lengths plausible
rest pose valid
no duplicate mappings
146. Facial Validation

Check:

required blendshapes
blink independence
mouth closure
jaw opening
viseme coverage
expression ranges
147. Viseme Validation

Every canonical viseme SHALL have a valid mapping or declared fallback.

148. IK Validation

Automated tests SHOULD verify that target positions:

left
right
high
low
near
far

produce valid arm poses within configured limits.

149. Gesture Validation

Check:

state compatibility
duration
interruptibility
target requirements
left/right variants
fallback
150. Performance Tests

Measure:

mesh triangles
draw calls
skin joints
blendshape count
CPU animation cost
GPU skinning cost
physics cost
VRAM
frame time
151. Baseline Desktop Budget

The exact numbers can be tuned later, but the architecture SHOULD set budgets for:

triangle count
texture memory
blendshape count
active physics chains
animation layers

rather than allow assets to grow without control.

152. Close-Up Quality Mode

For portrait tutoring, facial resolution matters more than full-body geometry.

Quality budgets SHOULD reflect camera mode.

153. Full-Body Mode

When full body is visible:

facial detail may reduce
body/gesture quality increases

through LOD.

154. Asset Streaming

Large assets MAY be loaded progressively.

Example:

core avatar
   ↓
high-res textures
   ↓
optional gesture pack
155. Startup Priority

Required first:

base mesh
face
skeleton
default materials
core states

Optional later:

advanced gestures
high-end physics
extra outfits
156. Outfit System

Future Nexa packages MAY support multiple outfits.

pub struct OutfitDefinition {
    pub id: OutfitId,
    pub mesh_parts: Vec<MeshPartId>,
    pub material_set: MaterialSetId,
}
157. Outfit Compatibility

Outfits SHALL be compatible with:

skeleton
physics
collision
gesture envelopes
158. Accessory System

Potential accessories:

glasses
headset
AR ear device
wrist device
tablet
159. Accessory Interaction

Accessories MAY have semantic states.

Example:

glasses display active
headset active
tablet held
160. Holographic UI

Future visual effects MAY show:

floating diagrams
packet flows
code overlays
network maps

near Nexa.

These SHOULD remain presentation-layer features rather than required avatar semantics.

161. Shader Extensions

Nexa-specific shaders MAY support:

subtle emissive accents
glass UI glow
cyber trim
holographic effects

But the canonical avatar SHALL remain renderable without proprietary shaders.

162. Fallback Materials

Every material SHOULD have a standard PBR fallback.

163. Renderer Capability Negotiation
pub struct ThreeDRendererCapabilities {
    pub morph_targets: bool,
    pub compute_skinning: bool,
    pub cloth: bool,
    pub hair_physics: bool,
    pub hdr: bool,
    pub post_processing: bool,
    pub xr: bool,
}
164. Graceful Degradation

If renderer lacks cloth simulation:

use baked/skinned cloth

If it lacks advanced eye shader:

use standard PBR eye material

If it lacks high-end shadows:

use reduced lighting profile
165. Rendering Failure

If 3D renderer fails entirely:

fallback to 2D Nexa

SHOULD be architecturally possible.

166. Avatar Mode Selection
pub enum AvatarRuntimeMode {
    ThreeD,
    TwoD,
    Headless,
    Hidden,
}
167. 3D-to-2D Fallback

Because behavior semantics are shared through NBP:

same TutorResponse
same BehaviorIntent
same NBP

can drive either:

3D runtime

or:

2D runtime

This is one of the most important benefits of the architecture we've established.

168. Character Identity Consistency

The 3D character SHALL preserve recognizable visual identity from the canonical Nexa artwork.

Review criteria:

face
hair silhouette
glasses
palette
clothing
age presentation
demeanor
cyber aesthetic
169. Character Drift Gate

A 3D asset release SHOULD require explicit visual comparison against the canonical reference.

170. Animation Personality

Animation SHALL reinforce Nexa as:

technically confident
calm
curious
slightly playful
approachable
professional
171. Movement Style

Nexa SHOULD generally move with:

controlled
precise
economical
purposeful

motion.

Not:

hyperactive
cartoonish
random
172. Tutor Performance Style

When explaining:

clear hand gestures
controlled gaze
small posture changes
direct eye contact at key moments
173. Hacker Aesthetic Without Caricature

The character may visually read as cyber/hacker-oriented through:

clothing;
lighting;
environment;
glasses;
UI;
technical confidence.

She SHOULD NOT rely on clichéd constant hood-up, green-code-rain behavior.

174. Desktop Environment

Initial 3D scene MAY include:

dark workstation
subtle cyber lighting
floating lesson canvas
terminal panel
ambient technical environment

The environment SHOULD remain secondary to training content.

175. Scene Presets
pub enum NexaScenePreset {
    CyberDesk,
    CleanStudio,
    Classroom,
    Lab,
    Minimal,
}
176. Minimal Mode

A low-distraction mode SHOULD display:

Nexa
+
neutral background
+
training UI

for users who prefer less visual complexity.

177. Accessibility

3D presentation SHALL respect:

reduced motion
captions
high contrast
keyboard navigation
screen-reader-compatible content

The avatar cannot be the only information channel.

178. Reduced Motion in 3D

Suppress or reduce:

cloth
hair
large gestures
camera shifts
idle movement

while preserving:

mouth
eyes
basic expression
essential pointing
179. Motion Sensitivity

Future XR modes SHOULD also support:

stationary camera
teleport-only movement
reduced world motion
180. Asset Security

3D packages SHALL be treated as data.

They SHALL NOT contain arbitrary executable scripts in the canonical package.

181. Nexa glTF Extensions

Custom metadata SHOULD use explicit namespaced extensions.

Conceptually:

EXT_nexa_behavior
EXT_nexa_visemes
EXT_nexa_rig
EXT_nexa_physics
182. Custom Extension Principle

Nexa-specific extensions SHALL be optional where possible.

A generic glTF viewer should still be able to render the base character.

183. VRM Compatibility

Where feasible, humanoid definitions and expressions SHOULD map to VRM-compatible concepts.

This improves interoperability.

184. VRM Import

A compatible VRM avatar MAY eventually be imported and adapted to Nexa semantics.

This could allow future alternate tutor characters.

185. Character Abstraction

Long term:

Nexa Behavior Protocol
       ↓
Avatar Profile
       ↓
Character Package

could support multiple tutor characters while preserving the same intelligence stack.

186. Nexa Remains Canonical

Even if the system later supports multiple avatars, Nexa remains the flagship canonical tutor profile.

187. Toolchain

Recommended production toolchain:

Concept Art
   ↓
Blender
   ↓
glTF/VRM Export
   ↓
Nexa Asset Compiler
   ↓
Validation
   ↓
Runtime Package
188. Asset Compiler
pub trait ThreeDAssetCompiler {
    fn compile(
        &self,
        source: ThreeDAssetSource,
    ) -> ThreeDResult<ThreeDAvatarPackage>;
}
189. Compiler Responsibilities

The compiler SHOULD:

validate skeleton
normalize naming
validate materials
validate blendshapes
validate visemes
generate manifests
build LOD references
compile physics metadata
hash assets
190. CLI

Future tool:

nexa avatar3d validate ./avatars/nexa

and:

nexa avatar3d build ./avatars/nexa
191. Visual Test Scene

A standard test scene SHOULD contain:

neutral lighting
front camera
3/4 camera
side camera
gesture targets
expression controls
viseme test phrases
192. Expression Test Mode

Developer inspector should permit:

expression sliders
blendshape inspection
jaw control
eye control
gaze target movement
193. Skeleton Inspector

Display:

joint hierarchy
joint axes
IK chains
current transforms
constraints
194. Animation Inspector

Display:

active base state
active gesture
blend weights
current clip times
layer masks
195. Physics Inspector

Display:

hair chains
spring strengths
colliders
cloth state
196. Performance Inspector

Display:

FPS
frame time
CPU animation
GPU render
draw calls
triangles
texture memory
LOD
quality mode
197. First 3D Milestone

The first target SHOULD be deliberately narrow:

Nexa head + upper torso
       ↓
humanoid skeleton
       ↓
eyes
       ↓
blink
       ↓
basic face expressions
       ↓
viseme lip sync
       ↓
head movement
       ↓
gaze

This already gives us a functioning 3D talking tutor.

198. Second 3D Milestone

Add:

full upper body
arms
hands
finger poses
nod
open hand
pointing IK
thinking gesture

Now Nexa can teach visually.

199. Third 3D Milestone

Add:

full body
seated/standing transitions
advanced gestures
hair physics
clothing physics
scene interaction
presentation mode
200. Fourth 3D Milestone

Add:

full 3D learning room
spatial diagrams
interactive virtual objects
XR support
201. First 3D Acceptance Scenario

Student asks:

“Which packet is SYN-ACK?”

Expected:

Nexa turns eyes toward diagram
      ↓
head follows
      ↓
right hand raises
      ↓
pointing IK resolves target
      ↓
speech begins
      ↓
accurate facial animation
      ↓
Nexa says:
"The server's response here is SYN-ACK."
      ↓
gaze returns to student
202. Second Acceptance Scenario

Student says:

“Wait.”

While Nexa is pointing and speaking:

audio stops
lip sync stops
hand exits gesture
head turns back
eyes meet learner
Nexa enters listening state

No snap or frozen facial pose is acceptable.

203. Third Acceptance Scenario

Nexa explains code in a right-side editor.

Expected:

gaze right
upper torso rotates slightly
right/left arm selected based on target reach
pointing gesture
speech + visemes
return to attentive pose
204. VRM/glTF Round Trip Acceptance

Canonical Nexa asset SHOULD:

export
import
validate

without losing:

skeleton
materials
facial mappings
core blendshapes
205. Headless IK Test

Given a target position:

x = right
y = shoulder height
z = reachable

the solver SHALL produce a valid arm pose without violating configured joint limits.

206. Lip-Sync Test

Given canonical audio and viseme events:

jaw
lips
mouth shape

SHALL remain synchronized over a long sentence without accumulated drift.

207. Gaze Test

Targets moving across the lesson canvas SHALL produce:

eye movement
then head follow

without exceeding constraints.

208. Renderer Swap Test

The same semantic test sequence SHOULD be executable through:

headless adapter
native renderer
future engine adapter

without changing NBP messages.

209. Recommended Crate Structure
crates/
└── nexa-3d/
    ├── src/
    │   ├── lib.rs
    │   ├── package.rs
    │   ├── manifest.rs
    │   ├── rig.rs
    │   ├── skeleton.rs
    │   ├── humanoid.rs
    │   ├── face.rs
    │   ├── blendshape.rs
    │   ├── viseme.rs
    │   ├── lipsync.rs
    │   ├── gaze.rs
    │   ├── ik.rs
    │   ├── hand.rs
    │   ├── gesture.rs
    │   ├── animation.rs
    │   ├── graph.rs
    │   ├── pose.rs
    │   ├── retarget.rs
    │   ├── physics.rs
    │   ├── materials.rs
    │   ├── lighting.rs
    │   ├── camera.rs
    │   ├── lod.rs
    │   ├── scene.rs
    │   ├── world.rs
    │   ├── quality.rs
    │   ├── capability.rs
    │   ├── validation.rs
    │   ├── compiler.rs
    │   ├── metrics.rs
    │   └── adapters/
    │       ├── mod.rs
    │       ├── headless.rs
    │       ├── native_wgpu.rs
    │       └── vrm.rs
    └── tests/
        ├── skeleton.rs
        ├── face.rs
        ├── viseme.rs
        ├── gaze.rs
        ├── ik.rs
        ├── gesture.rs
        ├── retarget.rs
        ├── replay.rs
        └── performance.rs
210. Dependency Direction
             nexa-domain
                 │
                 ▼
              nexa-nbp
                 │
                 ▼
              nexa-3d
          /       │        \
         ▼        ▼         ▼
      speech    events    renderer
         \        │         /
          \       │        /
             orchestrator
211. 2D/3D Coexistence

The architecture now becomes:

                  BehaviorIntent
                        │
                        ▼
                  NEXA-NBP-001
                        │
            ┌───────────┴────────────┐
            ▼                        ▼
      NEXA-AVTR-001             NEXA-3D-001
         2D/2.5D                     3D
            │                        │
            └────────────┬───────────┘
                         ▼
                    Presentation

This is deliberate.

We are not throwing away the 2D work.

We are creating interchangeable embodiment runtimes.

212. 3D Invariants

NEXA-3D-001 establishes these invariants:

The 3D avatar SHALL consume semantic behavior rather than Tutor-specific logic.
glTF SHALL be the preferred general asset foundation.
VRM compatibility SHOULD be supported for humanoid semantics.
Engine-native assets SHALL not be the only canonical representation.
Skeleton mappings SHALL be explicit and validated.
Facial animation SHALL remain independent of speech provider.
Viseme mappings SHALL remain canonical.
Audio playback SHALL remain the master lip-sync clock.
Gaze SHALL be target-driven.
Eye, head, and torso movement SHALL coordinate based on target magnitude.
IK SHALL be used where arbitrary interaction targets require it.
Pointing SHALL degrade gracefully when a target is unreachable.
Gesture animation SHALL remain layered with face and lip sync.
Animation layers SHALL use masks where appropriate.
Character motion SHALL remain controlled and purposeful.
Nexa's visual identity SHALL remain consistent with canonical art.
3D package loading SHALL validate skeleton, face, visemes, and materials.
Asset packages SHALL not require arbitrary script execution.
Renderer-specific APIs SHALL remain behind adapters.
GPU vendor assumptions SHALL be avoided.
Core facial communication SHALL survive quality degradation.
Hair and cloth simulation SHALL be lower priority than facial communication.
Runtime SHALL support 30–60 FPS target modes.
3D behavior SHALL support deterministic headless testing.
NBP replay SHALL drive 3D regression testing.
3D and 2D avatars SHALL coexist under the same behavior architecture.
Future XR SHALL extend existing semantics rather than replace them.
Native Rust rendering SHOULD be treated as a first-class implementation path.
Character source assets SHALL remain independently versioned.
The first 3D implementation SHOULD prioritize a highly convincing head/upper-body tutor before pursuing full-world complexity.
213. The Most Important Implementation Decision

At this point, the 3D path has enough architectural definition that we should stop pushing it down the roadmap.

The next stage should not be another dozen abstract subsystem documents before we touch the character.

I recommend the immediate sequence now becomes:

1. NEXA-3D-ART-001
   3D Character Production Specification


2. Produce Nexa turnaround/reference sheets


3. Build canonical Blender model


4. Establish humanoid skeleton


5. Establish face blendshape set


6. Export first glTF/GLB


7. Build minimal Rust + wgpu viewer


8. Add blink + gaze


9. Add viseme lip sync


10. Connect NBP


11. Add arm rig + pointing IK


12. Then return to NEXA-UX-001

That gets us to a real 3D Nexa prototype instead of allowing 3D to remain architectural paper.

214. Next Specification

The next document should therefore be:

NEXA-3D-ART-001 — Nexa 3D Character Modeling, Topology, Rigging & Asset Production Specification

That specification will be artist/technical-artist oriented and define the actual character construction requirements:

character dimensions
face proportions
body proportions
modeling reference
turnaround requirements
mesh separation
topology targets
polygon budgets
facial topology
eye construction
mouth cavity
teeth
hair construction
glasses geometry
clothing construction
UV layout
texture resolution
PBR maps
material definitions
skeleton joint placement
twist bones
finger rig
skin weights
facial blendshape catalog
viseme shapes
corrective shapes
LOD meshes
physics bones
collision volumes
export settings
Blender naming standards
glTF export profile
VRM mappings
validation checklist

That is the point where we start specifying how the actual 3D Nexa asset gets built, not merely how software consumes it.
