# NEXA-AVTR-001 — Avatar Runtime, Rigging, Animation, Expression & Behavior Engine Architecture v1.0

**Specification ID:** NEXA-AVTR-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Depends On:** NEXA-CBS-001, NEXA-NBP-001, NEXA-EVT-001, NEXA-ORCH-001, NEXA-SPCH-001
**Purpose:** Define Nexa’s visual embodiment layer, including canonical assets, 2D/2.5D rigging, face/body control, expression and gesture systems, gaze, micro-behaviors, lip synchronization, animation blending, runtime capability negotiation, NBP integration, performance constraints, asset versioning, testing, and future 3D extensibility.

---

## 1. Purpose

The Avatar Runtime answers:

> **“How should Nexa physically appear and behave on screen in response to semantic intent?”**

It translates high-level behavior such as:

```text
explaining
encouraging
look_at_diagram
point
speak
```

into:

```text
facial expression
head pose
eye direction
body posture
gesture
mouth animation
hair/clothing movement
rendered frames
```

The avatar runtime SHALL not decide what Nexa teaches or says.

---

# 2. Core Architectural Boundary

```text
TutorResponse
      │
      ▼
BehaviorIntent
      │
      ▼
NEXA-NBP-001
      │
      ▼
Behavior Engine
      │
      ├── State Controller
      ├── Expression Controller
      ├── Gaze Controller
      ├── Gesture Controller
      ├── Lip-Sync Controller
      └── Physics Controller
      │
      ▼
Avatar Adapter
      │
      ├── 2D / Live2D-style
      ├── 2.5D
      └── Future 3D / VRM
      │
      ▼
Renderer
```

This separation is foundational.

---

# 3. Core Responsibilities

The Avatar subsystem SHALL own or coordinate:

* canonical avatar asset loading;
* rig parameters;
* animation state;
* facial expression;
* eye movement;
* gaze;
* head movement;
* body posture;
* gestures;
* lip synchronization;
* viseme-to-mouth mapping;
* idle behaviors;
* blinking;
* breathing;
* micro-movement;
* hair physics;
* clothing/accessory physics;
* animation blending;
* expression blending;
* state transitions;
* runtime capability reporting;
* degraded behavior fallback;
* canvas targeting;
* render timing;
* avatar telemetry.

---

# 4. Explicit Non-Responsibilities

The Avatar subsystem SHALL NOT determine:

* lesson content;
* learner mastery;
* pedagogical strategy;
* TutorResponse text;
* tool authorization;
* speech recognition;
* TTS generation;
* curriculum flow.

---

# 5. Canonical Avatar Identity

The Nexa v1.0 reference artwork SHALL be treated as the canonical visual source.

The runtime implementation MAY change, but these identity elements SHOULD remain recognizable:

* dark hair;
* violet/purple accents;
* AR-style glasses;
* dark technical/cyber clothing;
* cyberpunk-inspired but professional visual style;
* confident and approachable expression;
* purple/cyan/magenta accent palette;
* technical workstation aesthetic.

---

# 6. Asset Architecture

```text
avatar/
└── nexa/
    ├── manifest.yaml
    ├── model/
    ├── textures/
    ├── rig/
    ├── expressions/
    ├── gestures/
    ├── motions/
    ├── physics/
    ├── visemes/
    ├── thumbnails/
    └── tests/
```

---

# 7. Avatar Manifest

Each avatar SHALL have a versioned manifest.

```yaml
avatar:
  id: nexa
  character_version: "1.0"
  rig_version: "1.0"
  behavior_profile: "nexa-default"
  canonical_reference: "nexa-v1-reference"

runtime:
  type: "2d"
  minimum_runtime_version: "0.1.0"

capabilities:
  facial_expression: true
  gaze: true
  head_pose: true
  upper_body_gestures: true
  full_body_gestures: false
  lip_sync: true
  hair_physics: true
  clothing_physics: true
```

---

# 8. Asset Versioning

Avatar assets SHALL be independently versioned from application code.

Recommended version dimensions:

```text
Character Version
Rig Version
Texture Version
Animation Pack Version
Behavior Mapping Version
Physics Version
```

---

# 9. Compatibility

The runtime SHOULD verify compatibility before loading assets.

Example:

```text
avatar rig requires:
runtime >= 0.4

runtime installed:
0.3

result:
reject or degraded load
```

---

# 10. Canonical 2D/2.5D Direction

The first Nexa implementation SHOULD use a layered 2D or 2.5D rig.

Advantages:

```text
lower implementation complexity
high character fidelity
good facial animation
efficient rendering
fast iteration
easy lip sync
desktop/web feasibility
```

---

# 11. Layer Decomposition

The canonical artwork SHOULD eventually be decomposed into independently controllable layers such as:

```text
back hair
rear accessories
body
jacket
neck
face base
nose
ears
left eye
right eye
left iris
right iris
left brow
right brow
mouth
teeth
tongue
glasses
front hair
side hair
hands
forearms
upper arms
foreground accessories
```

Exact layer structure depends on the final rigging system.

---

# 12. Rig Parameter Model

The runtime SHALL expose semantic rig parameters rather than application-specific bone names.

Conceptually:

```rust
pub struct AvatarRigState {
    pub head: HeadPose,
    pub eyes: EyeState,
    pub brows: BrowState,
    pub mouth: MouthState,
    pub body: BodyPose,
    pub hands: HandState,
}
```

---

# 13. Core Head Parameters

Recommended normalized semantic controls:

```text
HeadYaw
HeadPitch
HeadRoll

NeckYaw
NeckPitch

BodyYaw
BodyPitch
```

Values SHOULD use standardized normalized ranges where practical.

---

# 14. Eye Parameters

```text
EyeLookX
EyeLookY

EyeOpenLeft
EyeOpenRight

EyeSmileLeft
EyeSmileRight
```

---

# 15. Brow Parameters

```text
BrowLeftHeight
BrowRightHeight

BrowLeftAngle
BrowRightAngle

BrowLeftShape
BrowRightShape
```

---

# 16. Mouth Parameters

```text
MouthOpen
MouthWidth
MouthSmile
MouthFrown
MouthPucker
MouthForm
```

The renderer MAY expose more detailed controls internally.

---

# 17. Body Parameters

```text
BodyLeanX
BodyLeanY
ShoulderLeft
ShoulderRight
ArmLeft
ArmRight
HandLeft
HandRight
```

For the MVP, upper-body control is sufficient.

---

# 18. Parameter Normalization

The avatar adapter SHOULD translate normalized semantic values into renderer-specific values.

Example:

```text
Behavior Engine:
HeadYaw = 0.35

Live2D adapter:
ParamAngleX = 10.5°
```

The Behavior Engine SHALL not know `ParamAngleX`.

---

# 19. Avatar Runtime Trait

```rust
pub trait AvatarRuntime: Send + Sync {
    fn capabilities(&self) -> AvatarCapabilities;

    fn load(
        &mut self,
        avatar: &AvatarPackage,
    ) -> AvatarResult<()>;

    fn apply(
        &mut self,
        frame: AvatarControlFrame,
    ) -> AvatarResult<()>;

    fn update(
        &mut self,
        delta: Duration,
    ) -> AvatarResult<()>;

    fn render(
        &mut self,
        target: &mut RenderTarget,
    ) -> AvatarResult<()>;
}
```

---

# 20. Control Frame

The Behavior Engine SHOULD produce one semantic control frame per update interval.

```rust
pub struct AvatarControlFrame {
    pub state: AvatarBehaviorState,
    pub expression: ExpressionState,
    pub gaze: GazeState,
    pub gesture: GestureState,
    pub mouth: MouthState,
    pub pose: PoseState,
}
```

---

# 21. Behavior State

The Avatar subsystem consumes the semantic states defined in earlier specifications.

```rust
pub enum AvatarBehaviorState {
    Idle,
    Attentive,
    Listening,
    Thinking,
    Speaking,
    Explaining,
    Demonstrating,
    Questioning,
    Waiting,
    Observing,
    Evaluating,
    Hinting,
    Correcting,
    Encouraging,
    Celebrating,
    Warning,
    Summarizing,
    Debugging,
}
```

---

# 22. State Controller

The State Controller SHALL coordinate default pose, expression, gaze, and animation policies.

Example:

```text
THINKING
  ↓
default:
  head tilt = slight
  gaze = off-center
  expression = focused
  gesture = optional chin touch
```

These are defaults, not hard overrides.

---

# 23. Layered Behavior Composition

A final pose SHALL be composable.

```text
base state
    +
emotion
    +
gesture
    +
gaze
    +
speech
    +
micro-behavior
    =
final frame
```

---

# 24. Channel Priority

Recommended control priority:

```text
Safety/Warning behavior
      ↓
Explicit NBP gesture
      ↓
Speech-linked behavior
      ↓
State default
      ↓
Idle micro-behavior
```

Lower-priority channels SHALL not fight higher-priority commands.

---

# 25. Expression System

Expressions SHOULD be parameter compositions rather than only prerecorded facial images.

```rust
pub struct ExpressionState {
    pub preset: ExpressionPreset,
    pub intensity: f32,
    pub blend_time: Duration,
}
```

---

# 26. Expression Presets

Initial catalog:

```text
neutral
soft_smile
smile
wide_smile
curious
focused
thinking
surprised
concerned
skeptical
encouraging
corrective
excited
celebrating
confused
serious
```

---

# 27. Expression Definition

Example:

```yaml
expression:
  id: curious

  parameters:
    brow_left_height: 0.18
    brow_right_height: 0.18
    eye_open_left: 0.92
    eye_open_right: 0.92
    mouth_smile: 0.10
    head_roll: 0.08

  blend_in_ms: 180
  blend_out_ms: 220
```

---

# 28. Expression Blending

Hard switches SHOULD be avoided.

```text
neutral
   ↓ blend
curious
```

instead of:

```text
neutral → instant snap → curious
```

---

# 29. Expression Stack

Multiple expression influences MAY coexist.

Example:

```text
base:
focused

pedagogy:
encouraging

speech:
slight smile
```

A blending system SHOULD resolve them.

---

# 30. Emotion Translation

The Behavior Engine may receive:

```text
valence
arousal
confidence
engagement
```

and map them to facial parameters.

---

# 31. Continuous Emotion Mapping

Example:

```text
high positive valence
      ↓
mouth smile ↑

high engagement
      ↓
eye openness ↑
forward posture ↑

low arousal
      ↓
gesture amplitude ↓
```

This permits more subtle behavior than fixed presets alone.

---

# 32. Gaze System

Gaze is one of the most important elements of perceived intelligence.

Nexa SHALL support explicit gaze targets.

---

# 33. Gaze Targets

```rust
pub enum GazeTarget {
    Student,
    Camera,
    Canvas,
    CanvasObject(String),
    Terminal,
    CodeEditor,
    EnvironmentObject(String),
    Neutral,
}
```

---

# 34. Gaze Controller

```rust
pub struct GazeController {
    pub current_target: GazeTarget,
    pub target_position: Vec2,
    pub smoothing: GazeSmoothing,
}
```

---

# 35. Eye-First Gaze

For small target changes:

```text
eyes move first
head follows slightly
```

This generally looks more natural.

---

# 36. Large Gaze Shift

For larger target shifts:

```text
eyes move
   ↓
head rotates
   ↓
body may follow
```

---

# 37. Gaze Lead Before Gesture

When pointing:

```text
eyes → target
     ↓ 100–300 ms
head → target
     ↓
hand → target
```

This SHOULD be the default timing pattern.

---

# 38. Student Gaze

When listening:

```text
target = student/camera
```

but gaze SHOULD include subtle natural variation rather than staring perfectly into the camera continuously.

---

# 39. Eye Saccades

Natural micro-saccades MAY occur within a controlled range.

They SHALL be disabled during:

```text
important direct gaze
warning
precise pointing
```

where they would reduce clarity.

---

# 40. Blink System

Blinking SHALL be runtime-driven.

```rust
pub struct BlinkController {
    pub next_blink_at: Duration,
    pub state: BlinkState,
}
```

---

# 41. Blink Timing

Blink timing SHOULD be pseudo-random within bounded natural intervals.

The system SHOULD avoid:

```text
blink every exact 4.0 seconds
```

which quickly appears robotic.

---

# 42. Blink Suppression

Blink frequency MAY decrease during:

```text
high concentration
precise observation
```

and increase slightly after sustained eye opening.

---

# 43. Double Blink

Occasional double blinks MAY occur.

They SHOULD remain rare.

---

# 44. Breathing

A subtle breathing cycle SHOULD be present during most states.

```text
body/chest movement
very small
continuous
```

Breathing SHALL pause or reduce only where specific animation requires it.

---

# 45. Idle System

Idle behavior keeps Nexa visually alive.

Idle actions MAY include:

```text
breathing
blink
small head adjustment
small gaze shift
posture correction
glasses adjustment
minor shoulder motion
```

---

# 46. Idle Scheduler

```rust
pub struct IdleScheduler {
    pub candidates: Vec<IdleBehavior>,
    pub last_behavior: Option<IdleBehaviorId>,
}
```

The scheduler SHOULD avoid immediate repetition.

---

# 47. Idle Intensity

Idle movement SHOULD remain low amplitude during instructional activity.

The user should watch the lesson, not the animation system.

---

# 48. Listening State

Listening defaults:

```text
gaze → student
body → slight forward engagement
expression → attentive
mouth → rest
gesture → occasional nod
```

---

# 49. Listening Acknowledgment

During long student utterances Nexa MAY use:

```text
small nod
slight expression change
subtle head tilt
```

without interrupting verbally.

---

# 50. Thinking State

Thinking defaults:

```text
gaze slightly away
head tilt
focused expression
optional hand-to-chin
low-amplitude movement
```

The state SHALL visually correspond to real processing time.

---

# 51. Thinking Transition

```text
listening
   ↓
input completed
   ↓
thinking
   ↓
speech ready
   ↓
explaining/speaking
```

---

# 52. Speaking State

Speaking combines:

```text
mouth visemes
expression
gaze
gesture
head motion
body motion
```

The mouth SHALL not be the only animated channel.

---

# 53. Head Motion During Speech

Small speech-linked head motion MAY occur based on:

```text
phrase boundaries
emphasis
question intonation
```

The effect SHOULD be subtle.

---

# 54. Gesture System

Gestures SHALL be semantic.

Initial gesture vocabulary:

```text
none
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
small_clap
celebrate
attention
typing
```

---

# 55. Gesture Definition

```rust
pub struct GestureDefinition {
    pub id: GestureId,
    pub semantic_type: GestureType,
    pub compatible_states: Vec<AvatarBehaviorState>,
    pub duration: Duration,
    pub interruptibility: GestureInterruptibility,
    pub animation_ref: AnimationRef,
}
```

---

# 56. Gesture Compatibility

A `celebrate` gesture SHOULD generally not run during:

```text
warning
serious correction
assessment failure
```

unless explicitly overridden.

---

# 57. Gesture Intensity

The same semantic gesture SHOULD allow multiple intensities.

Example:

```text
point:
  low   → small hand movement
  medium → clear arm gesture
  high   → emphatic point
```

---

# 58. Gesture Variation

Repeated gestures SHOULD have variants.

Example:

```text
explain_open_hand_01
explain_open_hand_02
explain_open_hand_03
```

The semantic command remains:

```text
open_hand
```

---

# 59. Gesture Selection

The Behavior Engine selects among valid variants based on:

```text
recent history
current posture
target position
intensity
animation availability
```

---

# 60. Gesture Transitions

The runtime SHOULD prevent impossible pose jumps.

```text
hands_down
    ↓
point
```

may require a short transition animation.

---

# 61. Gesture Cancellation

Gestures SHALL support interruption.

Example:

```text
Nexa pointing
      ↓
student interrupts
      ↓
gesture.cancel
      ↓
return to listening pose
```

---

# 62. Canvas Pointing

Canvas-aware gestures need target coordinates.

```rust
pub struct CanvasTarget {
    pub object_id: String,
    pub screen_position: Vec2,
}
```

---

# 63. Canvas Coordinate Mapping

The Avatar Runtime SHOULD receive semantic screen-space target information through a presentation adapter.

It SHALL NOT query Tutor logic directly.

---

# 64. Pointing Accuracy

A point gesture SHOULD visually approximate the referenced UI object.

The hand does not need pixel-perfect intersection, but the direction must be unambiguous.

---

# 65. Pointer Alternative

If hand pointing is unsupported, fallback MAY be:

```text
gaze target
+
canvas highlight
```

This still communicates attention clearly.

---

# 66. Lip-Sync Input

The Avatar subsystem receives canonical viseme timing from NEXA-SPCH-001.

```rust
pub struct LipSyncFrame {
    pub speech_id: SpeechId,
    pub viseme: Viseme,
    pub intensity: f32,
}
```

---

# 67. Canonical Viseme Mapping

Example mapping:

```text
REST → mouth neutral
A    → open wide
E    → wide horizontal
I    → narrow smile
O    → rounded
U    → small rounded
MBP  → lips closed
FV   → lower lip / upper teeth
```

Exact shapes are rig-specific.

---

# 68. Viseme Adapter

```rust
pub trait VisemeMapper {
    fn map(
        &self,
        viseme: Viseme,
        intensity: f32,
    ) -> MouthParameters;
}
```

---

# 69. Coarticulation

Human speech mouth shapes overlap.

The runtime SHOULD blend adjacent visemes.

```text
previous
   ↓
current
   ↓
next
```

rather than treating every viseme as an isolated pose.

---

# 70. Mouth Smoothing

A small smoothing filter SHOULD prevent jitter from rapid viseme events.

It SHALL not introduce noticeable lag.

---

# 71. Speech Completion

On `speech.playback.completed`:

```text
mouth → REST
```

using a brief natural closing transition.

---

# 72. Lip-Sync Cancellation

When speech is interrupted:

```text
clear pending visemes
mouth blend → rest
```

immediately enough to avoid the avatar continuing to "talk" silently.

---

# 73. Facial Animation During Speech

Lip sync SHALL coexist with emotion.

Example:

```text
mouth viseme
+
soft smile
```

The smile SHALL not destroy phoneme readability.

---

# 74. Glasses

Nexa's glasses are a core visual identity element.

The rig SHOULD support:

```text
subtle reflections
position relative to face
optional adjustment gesture
```

They SHOULD NOT obscure eye expressions.

---

# 75. Hair Rig

Hair SHOULD be segmented sufficiently for natural movement.

Possible groups:

```text
back hair
left side
right side
front fringe
accent strands
```

---

# 76. Hair Physics

Hair movement MAY respond to:

```text
head movement
body movement
spring physics
damping
```

---

# 77. Hair Physics Principle

Physics SHOULD enhance life without producing distracting "jelly" motion.

---

# 78. Clothing Physics

Loose clothing components MAY receive secondary motion.

Examples:

```text
jacket panels
straps
small accessories
```

---

# 79. Accessory Physics

Optional accessories such as:

```text
cables
ear accessories
hood strings
```

may use constrained physics.

---

# 80. Physics Independence

Physics SHALL remain a rendering concern.

NBP SHOULD NOT issue commands such as:

```text
hair_spring_constant = 0.4
```

---

# 81. Animation Types

The runtime SHOULD distinguish:

```rust
pub enum AnimationType {
    StateLoop,
    Transition,
    Gesture,
    OneShot,
    SpeechSupport,
    Physics,
}
```

---

# 82. State Loops

Examples:

```text
idle_loop
listening_loop
thinking_loop
speaking_loop
waiting_loop
```

State loops SHOULD be subtle and seamless.

---

# 83. Transition Animations

Examples:

```text
idle → attentive
attentive → listening
thinking → speaking
speaking → listening
```

Transitions MAY be explicit clips or procedural blends.

---

# 84. One-Shot Animations

Examples:

```text
adjust_glasses
celebrate
thumbs_up
shrug
```

One-shots SHALL return to a valid state pose afterward.

---

# 85. Animation Blending

The runtime SHOULD blend compatible animations.

Example:

```text
speaking loop
+
point gesture
+
lip sync
+
eye gaze
```

---

# 86. Animation Layers

Conceptually:

```text
Layer 0: base posture
Layer 1: state animation
Layer 2: upper-body gesture
Layer 3: expression
Layer 4: gaze
Layer 5: mouth/lip sync
Layer 6: physics
```

Actual implementation may differ.

---

# 87. Layer Masks

Gestures SHOULD affect only necessary body regions.

Example:

```text
point:
right arm + shoulder

not:
entire face + left arm + head
```

unless designed that way.

---

# 88. Procedural Motion

Some movement SHOULD be procedural rather than prerecorded.

Examples:

```text
gaze
head follow
blink
breathing
mouth
```

---

# 89. Behavior Engine

The Behavior Engine is the semantic runtime brain for animation.

```rust
pub trait BehaviorEngine {
    fn accept(
        &mut self,
        command: NbpBehaviorCommand,
    ) -> BehaviorResult<BehaviorHandle>;

    fn cancel(
        &mut self,
        behavior_id: BehaviorId,
    ) -> BehaviorResult<()>;

    fn update(
        &mut self,
        delta: Duration,
    ) -> BehaviorResult<AvatarControlFrame>;
}
```

---

# 90. Behavior Handle

```rust
pub struct BehaviorHandle {
    pub behavior_id: BehaviorId,
    pub state: BehaviorExecutionState,
}
```

---

# 91. Behavior Execution State

```rust
pub enum BehaviorExecutionState {
    Queued,
    Starting,
    Active,
    Completing,
    Completed,
    Cancelled,
    Degraded,
    Failed,
}
```

---

# 92. Behavior Arbitration

Multiple behavior requests MAY coexist.

Example:

```text
background idle
explicit speech
warning
student interruption
```

The engine SHALL arbitrate deterministically.

---

# 93. Arbitration Inputs

```text
priority
state compatibility
interruptibility
timing
safety
current gesture
speech state
```

---

# 94. Priority Example

```text
idle behavior             10
normal explaining         50
student interruption      80
warning                    90
emergency/system           100
```

---

# 95. Behavior Queue

A runtime MAY maintain:

```text
active foreground behavior
foreground queue
background behavior pool
```

---

# 96. Background Behaviors

Background activity includes:

```text
blink
breathing
hair physics
minor idle movement
```

These SHALL not block foreground instructional behavior.

---

# 97. Behavior Templates

Common semantic behaviors SHOULD be defined as templates.

Example:

```yaml
behavior:
  id: explain_point

  state: explaining
  expression: focused
  gaze: target
  gesture: point

  defaults:
    gesture_intensity: 0.55
    gaze_lead_ms: 180
```

---

# 98. Template Versioning

Behavior templates SHALL be versioned independently.

This allows animation tuning without changing Tutor logic.

---

# 99. NBP Adapter

The adapter translates NBP into Behavior Engine requests.

```text
NBP:
gesture = point

Adapter:
semantic gesture request

Behavior Engine:
select point variant

Avatar Adapter:
actual rig animation
```

---

# 100. Unknown NBP Behavior

If the runtime receives unsupported semantics:

```text
requested:
two_hand_explain

runtime:
unsupported
```

fallback:

```text
open_hand
```

and emit:

```text
avatar.behavior.degraded
```

---

# 101. Runtime Capability Advertisement

```rust
pub struct AvatarCapabilities {
    pub expressions: Vec<ExpressionPreset>,
    pub gestures: Vec<GestureType>,

    pub gaze: bool,
    pub lip_sync: LipSyncCapability,

    pub upper_body: bool,
    pub full_body: bool,

    pub physics: PhysicsCapabilities,
}
```

---

# 102. Lip-Sync Capability

```rust
pub enum LipSyncCapability {
    None,
    Amplitude,
    CanonicalViseme,
    FullPhoneme,
}
```

---

# 103. Capability Negotiation

The orchestrator/Behavior Planner SHOULD know whether Nexa can physically execute a requested behavior.

This avoids invalid commands reaching the renderer.

---

# 104. Renderer Independence

Core avatar semantics SHALL remain independent from:

```text
Live2D Cubism
Spine
Godot
Unity
Unreal Engine
WebGL
WebGPU
VRM
```

Adapters MAY support these technologies.

---

# 105. Initial Adapter

The initial implementation SHOULD use:

```text
AvatarRuntime
      ↓
2D Runtime Adapter
      ↓
Nexa Rig
```

The exact engine can be selected during implementation.

---

# 106. Future 3D Adapter

A future 3D system SHOULD implement the same semantic runtime contract.

```text
BehaviorIntent
      ↓
NBP
      ↓
Behavior Engine
      ↓
VRM/3D Adapter
```

Tutor logic remains unchanged.

---

# 107. 3D Expansion

Future capabilities may include:

```text
full-body locomotion
room navigation
object interaction
VR
AR
hand tracking
camera staging
```

These SHALL extend, not replace, the semantic architecture.

---

# 108. Render Loop

The avatar runtime SHOULD use a stable frame update.

Conceptually:

```text
receive commands
      ↓
update behavior
      ↓
update animation
      ↓
update physics
      ↓
apply rig values
      ↓
render frame
```

---

# 109. Frame Timing

```rust
pub struct FrameTiming {
    pub delta: Duration,
    pub frame_index: u64,
    pub render_timestamp: Instant,
}
```

---

# 110. Target Frame Rate

The preferred desktop target SHOULD be:

```text
60 FPS
```

with graceful support for lower rates.

---

# 111. Variable Frame Rate

Animation updates SHALL use elapsed time rather than assuming a fixed frame count.

---

# 112. Frame Drops

If frame rate drops:

```text
physics and animation timing
```

SHOULD remain temporally correct.

---

# 113. Render Performance Budget

Performance budgets SHOULD include:

```text
CPU update time
GPU render time
memory
texture memory
animation update
physics update
```

---

# 114. Speech Priority

During speech, lip-sync timing SHOULD take priority over optional secondary animation effects.

If performance degrades:

```text
reduce physics
reduce idle motion
preserve mouth + gaze + core gesture
```

---

# 115. Quality Levels

```rust
pub enum AvatarQualityLevel {
    Low,
    Medium,
    High,
    Ultra,
    Adaptive,
}
```

---

# 116. Adaptive Quality

The runtime MAY reduce:

```text
physics iterations
secondary motion
texture resolution
post-processing
```

when frame timing deteriorates.

---

# 117. Core Facial Animation SHALL Remain

Quality degradation SHOULD preserve:

```text
eyes
mouth
basic expression
gaze
```

because these are essential to communication.

---

# 118. Avatar Positioning

The UI SHOULD support flexible avatar layout.

Potential modes:

```text
left tutor panel
right tutor panel
floating portrait
full-screen instructor
picture-in-picture
```

The runtime SHALL not assume one layout.

---

# 119. Avatar Bounds

```rust
pub struct AvatarViewport {
    pub position: Vec2,
    pub size: Vec2,
    pub scale: f32,
}
```

---

# 120. Canvas Relationship

The UI presentation layer SHALL expose the position of referenced visual objects.

The avatar may use those positions for gaze and pointing.

---

# 121. Screen Edge Awareness

The Behavior Engine SHOULD avoid gestures that extend awkwardly beyond the avatar viewport.

---

# 122. Mirror Poses

Some gestures MAY support left/right mirrored variants.

This helps point toward content on either side of Nexa.

---

# 123. Dominant Hand

Nexa SHOULD have a canonical dominant hand for neutral behavior.

Recommended:

```text
right hand
```

but pointing SHOULD switch where screen composition requires.

---

# 124. Pose Memory

The runtime SHOULD know the current physical pose so transitions remain natural.

Avoid:

```text
current right arm raised
      ↓
new animation assumes arm down
      ↓
visual snap
```

---

# 125. Gesture Planning

The Gesture Controller SHOULD select transitions based on current pose.

---

# 126. Idle Reentry

After a gesture finishes:

```text
gesture ending pose
       ↓
blend
       ↓
current state loop
```

not necessarily directly to canonical neutral.

---

# 127. Expression Persistence

Expressions SHOULD persist only as long as semantically appropriate.

Example:

```text
celebrating
   ↓
soft_smile
   ↓
attentive
```

rather than remaining exaggerated indefinitely.

---

# 128. State-Dependent Expression Defaults

Examples:

```text
Listening → attentive
Thinking → focused
Correcting → corrective/encouraging
Warning → serious
Celebrating → positive/high energy
```

---

# 129. Corrective Behavior

Correction SHOULD not visually look hostile.

Recommended:

```text
focused
slight concern
small nod
calm posture
```

not:

```text
anger
eye roll
mocking smile
```

---

# 130. Encouragement Behavior

Encouragement MAY use:

```text
soft smile
small nod
slight forward lean
```

rather than exaggerated celebration.

---

# 131. Celebration Levels

```rust
pub enum CelebrationLevel {
    Subtle,
    Moderate,
    Strong,
}
```

---

# 132. Subtle Celebration

For ordinary success:

```text
smile
small nod
```

---

# 133. Strong Celebration

Reserved for meaningful milestones:

```text
course completion
mastery breakthrough
difficult lab success
```

---

# 134. Warning Behavior

Warning behavior SHOULD increase clarity.

```text
direct gaze
serious expression
reduced idle motion
attention gesture
deliberate posture
```

---

# 135. Warning SHALL NOT Be Theatrical

Avoid alarms or dramatic gestures for routine cautions.

---

# 136. Debugging Behavior

Debugging state defaults:

```text
gaze → terminal/code
expression → focused
gesture → minimal
occasional thinking movement
```

---

# 137. Demonstration State

When demonstrating:

```text
gaze alternates between content and student
gesture toward relevant region
expression focused/confident
```

---

# 138. Questioning State

Questioning defaults:

```text
gaze → student
head tilt slight
curious expression
mouth completes speech
then wait
```

---

# 139. Waiting State

After a question:

```text
mouth rest
gaze mostly toward student
low movement
occasional blink
no distracting gesture
```

This gives the learner space to think.

---

# 140. Animation Event Model

The runtime SHOULD emit:

```text
avatar.loaded
avatar.ready

avatar.state.changed

avatar.behavior.started
avatar.behavior.completed
avatar.behavior.cancelled
avatar.behavior.degraded
avatar.behavior.failed

avatar.expression.started
avatar.expression.completed

avatar.gesture.started
avatar.gesture.completed

avatar.gaze.started
avatar.gaze.completed

avatar.runtime.performance_degraded
```

---

# 141. Behavior Started Example

```json
{
  "event_type": "avatar.behavior.started",
  "payload": {
    "avatar_id": "nexa.primary",
    "behavior_id": "beh-8821",
    "state": "explaining"
  }
}
```

---

# 142. Degradation Example

```json
{
  "event_type": "avatar.behavior.degraded",
  "payload": {
    "behavior_id": "beh-8821",
    "requested_gesture": "two_hand_explain",
    "fallback_gesture": "open_hand"
  }
}
```

---

# 143. Performance Event

```json
{
  "event_type": "avatar.runtime.performance_degraded",
  "payload": {
    "fps": 31.2,
    "target_fps": 60,
    "quality_before": "high",
    "quality_after": "medium"
  }
}
```

---

# 144. Runtime Error Types

```rust
pub enum AvatarError {
    AssetNotFound,
    AssetIncompatible,
    RigInvalid,
    AnimationMissing,
    UnsupportedBehavior,
    RenderFailed,
    DeviceLost,
    OutOfMemory,
    InvalidControlFrame,
}
```

---

# 145. Avatar Failure Fallback

If Nexa cannot render:

```text
speech + text
```

SHOULD continue.

The avatar SHALL not be a single point of failure for tutoring.

---

# 146. Partial Asset Failure

If one gesture animation is missing:

```text
fallback gesture
```

rather than full runtime failure.

---

# 147. Asset Validation

At load time the system SHOULD validate:

```text
required rig parameters
required expression mappings
required viseme mappings
required state loops
manifest compatibility
```

---

# 148. Required MVP Parameters

Minimum:

```text
head yaw
head pitch
head roll

eye X/Y
eye open L/R

brow position

mouth open
mouth form

body lean

breathing
```

---

# 149. MVP Expressions

Minimum:

```text
neutral
focused
curious
encouraging
serious
smile
```

---

# 150. MVP Gestures

Minimum:

```text
idle
nod
head_tilt
point
open_hand
```

---

# 151. MVP States

Minimum:

```text
idle
attentive
listening
thinking
speaking
explaining
questioning
waiting
```

---

# 152. MVP Lip Sync

The first implementation SHOULD support:

```text
canonical visemes
+
smoothing
+
speech playback synchronization
```

---

# 153. MVP Physics

Initial physics SHOULD include only:

```text
hair
small clothing/accessory motion
```

Physics complexity SHOULD remain secondary to facial quality.

---

# 154. MVP Runtime Sequence

```text
NBP behavior.command
      ↓
Behavior Engine
      ↓
state = explaining
emotion = focused
gaze = canvas
gesture = point
      ↓
TTS starts
      ↓
visemes arrive
      ↓
avatar speaks and points
      ↓
speech completes
      ↓
gesture completes
      ↓
avatar → attentive
```

---

# 155. First Visual Acceptance Scenario

Student asks:

> "What does SYN-ACK do?"

Expected:

```text
Nexa → thinking
      ↓
eyes shift toward TCP diagram
      ↓
Nexa → explaining
      ↓
right hand points toward SYN-ACK
      ↓
speech begins
      ↓
accurate mouth movement
      ↓
subtle head motion
      ↓
speech ends
      ↓
hand returns
      ↓
Nexa → attentive
```

---

# 156. Listening Acceptance Scenario

Student begins speaking.

Expected:

```text
active idle behavior cancelled if necessary
      ↓
gaze → student
      ↓
mouth → rest
      ↓
attentive expression
      ↓
subtle nod during longer statement
```

---

# 157. Interruption Acceptance Scenario

Nexa is explaining and pointing.

Student says:

> "Wait."

Expected:

```text
speech cancellation
      ↓
viseme queue cleared
      ↓
point gesture cancelled gracefully
      ↓
Nexa → listening
      ↓
gaze → student
```

No animation snap SHOULD occur.

---

# 158. Expression Acceptance Test

Transition:

```text
focused
   ↓
encouraging
```

SHOULD blend smoothly within the configured transition time.

---

# 159. Gaze Acceptance Test

When the lesson references:

```text
canvas.object = tcp.syn_ack
```

Nexa SHALL visually direct attention toward that object before or during the associated gesture.

---

# 160. Lip-Sync Acceptance Test

Given recorded:

```text
audio
+
viseme timeline
```

replay SHALL produce:

```text
stable audio/visual synchronization
```

without accumulating drift.

---

# 161. Behavior Replay

The runtime SHOULD support deterministic replay of recorded NBP messages.

This enables avatar regression testing without invoking Tutor, Pedagogy, or TTS systems.

---

# 162. Replay Package

A test fixture MAY include:

```text
NBP stream
audio file
viseme timeline
canvas target positions
```

---

# 163. Deterministic Mode

Development/test mode SHOULD disable randomness in:

```text
blink timing
idle selection
gaze jitter
physics seed
```

where possible.

---

# 164. Screenshot Regression

Automated tests MAY render known frames and compare them for major visual regressions.

Minor physics variation SHOULD be disabled during such tests.

---

# 165. Motion Regression Tests

Tests SHOULD verify:

```text
state transitions
gesture completion
interruptions
mouth rest after speech
gaze targeting
expression blend
```

---

# 166. Asset Regression

Every Nexa asset release SHOULD run:

```text
manifest validation
rig validation
animation reference validation
viseme mapping test
state coverage test
```

---

# 167. Behavior Coverage

The runtime SHOULD expose a coverage report.

Example:

```text
NBP gesture catalog:
18

implemented:
15

fallback:
3

missing:
0
```

---

# 168. Performance Testing

Avatar tests SHOULD measure:

```text
average FPS
1% low FPS
CPU utilization
GPU utilization
memory
texture memory
frame time
animation update time
physics time
```

---

# 169. Low-End Hardware Mode

The architecture SHOULD support reduced quality for systems with limited graphics capability.

It SHALL preserve core tutoring interaction.

---

# 170. Headless Avatar Mode

Testing SHOULD permit avatar behavior processing without rendering.

```text
NBP
   ↓
Behavior Engine
   ↓
control frames
```

This supports CI.

---

# 171. Headless Assertions

Tests can verify:

```text
state == explaining
gesture == point
gaze == tcp.syn_ack
mouth == REST after cancellation
```

without GPU rendering.

---

# 172. Avatar Package Format

A portable package SHOULD eventually contain:

```text
manifest
model
textures
animations
expressions
physics
mapping files
license metadata
```

---

# 173. Licensing Metadata

Avatar assets SHOULD track provenance and license information.

```yaml
licenses:
  artwork: internal
  rig: internal
  third_party_assets: []
```

---

# 174. Canonical Character Reference

The master concept artwork SHOULD be retained in the development asset catalog as:

```text
Nexa Character Reference v1.0
```

and used when reviewing future visual changes.

---

# 175. Character Drift Review

Changes SHOULD be evaluated for:

```text
face consistency
hair
glasses
color palette
clothing identity
age presentation
overall personality
```

---

# 176. Expression Reference Sheet

A dedicated expression sheet SHOULD eventually be produced containing:

```text
neutral
curious
focused
thinking
encouraging
serious
corrective
surprised
celebrating
```

This becomes a rigging reference.

---

# 177. Gesture Reference Sheet

A separate gesture sheet SHOULD define:

```text
point left
point right
open hand
two-hand explain
thinking chin
adjust glasses
nod
head tilt
attention
celebrate
```

---

# 178. Animation Bible

The Avatar team SHOULD maintain:

```text
animation meaning
state compatibility
intensity
duration
interruptibility
fallback
```

for every behavior asset.

---

# 179. Behavior Mapping File

Example:

```yaml
gesture:
  point:
    variants:
      - point_right_01
      - point_right_02
      - point_left_01

    fallback:
      open_hand
```

---

# 180. Expression Mapping File

```yaml
expression:
  encouraging:
    base: soft_smile
    parameters:
      brow_height: 0.08
      eye_smile: 0.18
      mouth_smile: 0.32
```

---

# 181. Viseme Mapping File

```yaml
visemes:
  MBP:
    mouth_open: 0.0
    mouth_form: -0.3

  A:
    mouth_open: 0.85
    mouth_form: 0.1
```

---

# 182. Runtime Hot Reload

Development builds SHOULD support hot reload of:

```text
expression mappings
gesture mappings
behavior templates
physics parameters
```

where possible.

This dramatically improves animation tuning.

---

# 183. Asset Hot Reload Safety

Production builds MAY disable unrestricted asset hot reload.

---

# 184. Developer Inspector

A useful avatar development UI SHOULD eventually expose:

```text
current state
expression
gaze target
gesture
viseme
rig parameters
FPS
active behavior
NBP message
```

---

# 185. Manual Parameter Inspector

Rigging developers SHOULD be able to manually manipulate:

```text
head X/Y/Z
eye X/Y
mouth
brows
body
```

to validate ranges.

---

# 186. NBP Debugger

A developer should be able to send:

```text
state = explaining
emotion = curious
gesture = point
target = canvas.object.4
```

without invoking the full tutor stack.

---

# 187. Behavior Recording

The inspector MAY record control streams for later playback.

---

# 188. Camera Awareness

For 2D runtime, "camera" refers primarily to the student's viewpoint.

Future 3D runtime MAY expose real camera position.

---

# 189. Eye Contact

When addressing the learner directly:

```text
gaze target = camera/student
```

This creates the perception of eye contact.

---

# 190. Avoiding Unnatural Staring

Direct gaze SHOULD occasionally relax unless the communication context requires sustained attention.

---

# 191. Screen Position Awareness

When Nexa is docked left:

```text
content likely right
```

gesture defaults may favor her rightward-facing motions.

When docked right, the opposite may apply.

---

# 192. Layout Events

The UI SHOULD notify the avatar layer when viewport geometry changes.

```text
window resized
avatar panel moved
canvas object moved
```

---

# 193. Responsive Rig Scaling

Scaling SHALL preserve:

```text
face readability
mouth visibility
eye visibility
gesture clarity
```

---

# 194. Minimum Display Size

The design SHOULD establish a minimum useful avatar display size below which some gestures are suppressed.

At small sizes:

```text
facial expression + gaze
```

matter more than hand gestures.

---

# 195. Portrait Mode

A compact mode MAY show:

```text
head
shoulders
upper torso
```

with gesture vocabulary reduced appropriately.

---

# 196. Full Instructor Mode

A larger mode MAY enable:

```text
upper-body gestures
larger pointing
more visible physics
```

---

# 197. Runtime Modes

```rust
pub enum AvatarPresentationMode {
    Portrait,
    TutorPanel,
    FullInstructor,
    Hidden,
}
```

---

# 198. Hidden Mode

When hidden:

```text
speech may continue
```

The Behavior Engine MAY reduce rendering work.

---

# 199. Accessibility

The avatar SHALL complement, not replace:

```text
captions
display text
keyboard controls
screen-reader-compatible lesson content
```

---

# 200. Motion Reduction

The system SHOULD support reduced-motion mode.

```rust
pub enum MotionPreference {
    Normal,
    Reduced,
    Minimal,
}
```

---

# 201. Reduced Motion

Reduced-motion mode may suppress:

```text
large gestures
secondary hair movement
frequent idle movement
```

while preserving:

```text
mouth
eyes
essential gaze
subtle expression
```

---

# 202. Flashing Effects

The avatar system SHOULD avoid unnecessary flashing or rapid effects.

---

# 203. Avatar Runtime Privacy

The Avatar Runtime SHOULD not require direct access to:

```text
student competency history
personal profile
assessment answers
```

It needs only behavior semantics.

---

# 204. Security Boundary

Avatar packages SHALL be treated as data.

Loading an avatar package SHALL NOT permit arbitrary script execution unless explicitly sandboxed and approved.

---

# 205. Asset Validation Security

Packages SHOULD validate:

```text
paths
file sizes
manifest schema
supported formats
texture bounds
animation references
```

---

# 206. No Arbitrary NBP Code

NBP SHALL remain semantic.

It SHALL not allow:

```text
eval
run script
load DLL
execute shell
```

---

# 207. Runtime Recovery

If the graphics device is temporarily lost:

```text
renderer reset
      ↓
reload assets
      ↓
restore current semantic state
```

where supported.

---

# 208. State Recovery

After avatar restart:

```text
orchestrator expected state
      ↓
behavior synchronization
      ↓
avatar resumes appropriate state
```

---

# 209. Speech Recovery

If avatar restarts during speech, default policy SHOULD normally:

```text
resume visually at current speech position
```

if timing information remains available.

Otherwise:

```text
wait for next speech turn
```

may be less disruptive.

---

# 210. Runtime Health

```rust
pub struct AvatarHealth {
    pub state: AvatarHealthState,
    pub fps: Option<f32>,
    pub active_avatar: Option<AvatarId>,
    pub active_behavior: Option<BehaviorId>,
}
```

---

# 211. Health States

```rust
pub enum AvatarHealthState {
    Healthy,
    Degraded,
    Unavailable,
    Recovering,
}
```

---

# 212. Avatar Metrics

Collect:

```text
load time
frame time
FPS
behavior start latency
gesture transition latency
gaze target latency
viseme application latency
lip-sync drift
fallback rate
animation missing count
device recovery count
```

---

# 213. Behavior Start Latency

Measure:

```text
NBP command accepted
      ↓
visible behavior begins
```

This is important for conversational responsiveness.

---

# 214. Gaze Latency

Gaze should react quickly enough that Nexa appears aware of content transitions.

---

# 215. Gesture Latency

Large gestures may intentionally have anticipatory delay.

The important distinction is:

```text
planned delay
vs
runtime lag
```

---

# 216. Avatar Test Corpus

The regression corpus SHOULD include:

```text
idle
listen
think
explain
question
correct
encourage
celebrate
warn
point left
point right
speech long
speech short
speech interruption
rapid state transition
reduced-motion mode
low-quality mode
```

---

# 217. Visual Quality Review

Human review remains important for:

```text
personality
naturalness
gesture appropriateness
expression quality
visual consistency
```

Automated metrics alone cannot fully judge character performance.

---

# 218. Uncanny Behavior Avoidance

Special attention SHALL be paid to:

```text
staring
over-blinking
mouth jitter
overactive breathing
gesture repetition
instant pose snaps
exaggerated emotion
head movement during every syllable
```

---

# 219. Behavioral Economy

Nexa SHOULD not move merely because the system can animate her.

Silence and stillness are valid behaviors.

---

# 220. Character Consistency Rule

The animation system SHALL reinforce Nexa's established persona:

```text
intelligent
calm
technically capable
curious
approachable
slightly playful
professional
```

The rig SHALL not turn her into an exaggerated cartoon unless a future character redesign explicitly does so.

---

# 221. MVP Repository Structure

```text
avatar/
└── nexa/
    ├── manifest.yaml
    ├── model/
    ├── textures/
    ├── expressions/
    ├── gestures/
    ├── states/
    ├── physics/
    ├── mappings/
    │   ├── nbp.yaml
    │   ├── expressions.yaml
    │   ├── gestures.yaml
    │   └── visemes.yaml
    └── tests/
```

Runtime crate:

```text
crates/
└── nexa-avatar/
    ├── src/
    │   ├── lib.rs
    │   ├── runtime.rs
    │   ├── package.rs
    │   ├── manifest.rs
    │   ├── behavior.rs
    │   ├── state.rs
    │   ├── arbitration.rs
    │   ├── expression.rs
    │   ├── gaze.rs
    │   ├── gesture.rs
    │   ├── lipsync.rs
    │   ├── viseme.rs
    │   ├── idle.rs
    │   ├── blink.rs
    │   ├── physics.rs
    │   ├── viewport.rs
    │   ├── capability.rs
    │   ├── metrics.rs
    │   ├── errors.rs
    │   └── adapters/
    │       ├── mod.rs
    │       ├── headless.rs
    │       └── runtime_2d.rs
    └── tests/
        ├── behavior.rs
        ├── state.rs
        ├── expression.rs
        ├── gaze.rs
        ├── gesture.rs
        ├── lipsync.rs
        └── replay.rs
```

---

# 222. Dependency Direction

```text
             nexa-domain
                 │
                 ▼
              nexa-nbp
                 │
                 ▼
            nexa-avatar
             /      \
            ▼        ▼
      nexa-events  2D adapter
            │
            ▼
      nexa-orchestrator
```

Speech connects through timing/events rather than becoming an implementation dependency on the model rig.

---

# 223. MVP Implementation Order

The Avatar subsystem SHOULD be built in this order:

```text
1. Load static Nexa
2. Rig head/eyes
3. Add blink
4. Add breathing
5. Add expressions
6. Add gaze
7. Add mouth parameters
8. Add canonical visemes
9. Synchronize recorded speech
10. Add listening/thinking/speaking states
11. Add nod
12. Add pointing
13. Connect NBP
14. Add interruption
15. Add physics
16. Add developer inspector
```

---

# 224. First Avatar Milestone

The first genuinely useful Nexa visual milestone is:

```text
static portrait
      ↓
blink
      ↓
breathing
      ↓
head movement
      ↓
eye tracking
      ↓
expression changes
      ↓
speech lip sync
```

At that point Nexa already feels alive.

---

# 225. Second Avatar Milestone

Add:

```text
listening
thinking
explaining
questioning
nod
point
```

This produces an actual tutoring avatar.

---

# 226. Third Avatar Milestone

Add:

```text
behavior arbitration
advanced gesture variations
hair/clothing physics
canvas targeting
adaptive quality
full interruption
```

---

# 227. Avatar Runtime Acceptance Gate

`NEXA-AVTR-001` SHALL be considered MVP-complete when:

1. Nexa loads from a versioned avatar package.
2. Blink and breathing execute continuously.
3. Gaze can target student and canvas coordinates.
4. Six baseline expressions blend smoothly.
5. Five baseline gestures execute without pose snapping.
6. Canonical visemes drive mouth shapes.
7. Speech and mouth remain synchronized.
8. Speech cancellation stops mouth animation.
9. NBP commands drive behavior.
10. Unsupported gestures degrade gracefully.
11. Runtime capability negotiation works.
12. The avatar can recover to `ATTENTIVE` after a behavior.
13. A headless behavior test suite passes.
14. Frame performance meets the target hardware profile.

---

# 228. Avatar Runtime Invariants

`NEXA-AVTR-001` establishes these invariants:

1. Avatar rendering SHALL remain separate from Tutor intelligence.
2. NBP SHALL be the semantic behavior boundary.
3. Renderer-specific parameter names SHALL not leak into Tutor or Pedagogy layers.
4. Avatar behavior SHALL be composable across independent channels.
5. Gaze SHALL be explicit.
6. Lip sync SHALL use speech timing as the primary synchronization clock.
7. Viseme mapping SHALL remain renderer-specific behind an adapter.
8. Speech cancellation SHALL clear pending mouth movement.
9. Expressions SHOULD blend rather than snap.
10. Gestures SHOULD blend into and out of current posture.
11. Idle behavior SHALL remain lower priority than instructional behavior.
12. Behavioral randomness SHOULD be bounded and controllable.
13. Avatar assets SHALL be versioned.
14. Behavior templates SHALL be versioned.
15. Asset loading SHALL validate compatibility.
16. Missing optional animation assets SHALL degrade gracefully.
17. Avatar failure SHALL not terminate basic tutoring.
18. Core facial communication SHALL be preserved under performance degradation.
19. The initial implementation SHOULD prioritize facial quality over complex physics.
20. Canvas targeting SHALL use presentation-layer coordinates.
21. NBP SHALL not allow arbitrary code execution.
22. Avatar packages SHALL be treated as untrusted data.
23. Reduced-motion presentation SHALL be supported.
24. Headless deterministic behavior testing SHALL be supported.
25. Recorded NBP streams SHOULD be replayable.
26. Future 3D implementations SHALL preserve the same semantic interfaces.
27. Nexa's visual identity SHALL remain consistent with the canonical character reference.
28. Animation SHALL support pedagogy rather than distract from it.

---

# 229. Architecture Status

We now have the complete human-facing Nexa loop:

```text
                   STUDENT
                      │
              ┌───────┴───────┐
              ▼               ▼
            Voice            Text
              │               │
              ▼               │
        NEXA-SPCH-001          │
              │               │
              └───────┬───────┘
                      ▼
                NEXA-STU-001
                      │
                      ▼
                NEXA-PED-001
                      │
             ┌────────┴────────┐
             ▼                 ▼
       NEXA-KNOW-001      Curriculum
             │                 │
             └────────┬────────┘
                      ▼
               NEXA-TUTOR-001
                      │
                      ▼
               NEXA-ORCH-001
                      │
            ┌─────────┼─────────┐
            ▼         ▼         ▼
          Speech     NBP      Canvas
            │         │
            │         ▼
            │   NEXA-AVTR-001
            │         │
            └────┬────┘
                 ▼
             Animated
               Nexa
```

At this point, the architecture defines how Nexa can **listen, understand, decide how to teach, retrieve governed knowledge, formulate a response, speak it, and physically perform it**.

---

# 230. Next Specification

The next system-level specification should be:

# **NEXA-LESSON-001 — Curriculum, Course, Lesson, Content & Adaptive Learning Flow Architecture v1.0**

This is the next logical dependency because we have now defined the tutor herself, but we need to define the structured training experiences she delivers.

It should cover:

```text
curriculum hierarchy
courses
modules
lessons
lesson steps
learning objectives
competency mappings
prerequisites
lesson branches
content blocks
instructional activities
examples
practice
labs
questions
review
remediation branches
challenge branches
lesson progression
completion rules
resume state
course manifests
content packaging
versioning
local course packs
content dependencies
adaptive lesson routing
Nexa behavior cues
knowledge-source bindings
assessment bindings
lab bindings
course validation
authoring
testing
publishing
```

That will turn Nexa from an excellent freeform tutor into a **complete training platform capable of delivering authored, adaptive courses from beginning to mastery**.
