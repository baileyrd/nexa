# NEXA-NBP-001 — Nexa Behavior Protocol v1.0

**Specification ID:** NEXA-NBP-001
**Protocol Name:** Nexa Behavior Protocol
**Version:** 1.0
**Status:** Baseline Draft
**Purpose:** Define the stable semantic interface between Nexa's cognitive systems and avatar/rendering systems.

---

## 1. Purpose

The **Nexa Behavior Protocol (NBP)** defines how higher-level Nexa systems communicate behavioral intent to lower-level avatar, speech, animation, and presentation runtimes.

NBP exists to ensure that Nexa's intelligence does not directly depend on:

* animation clip names;
* skeletal rigs;
* facial parameter implementations;
* Live2D internals;
* Unity;
* Unreal Engine;
* VRM;
* WebGL;
* speech vendors;
* rendering APIs.

The protocol SHALL express **meaning and intent**, not implementation details.

---

## 2. Primary Architectural Rule

The protocol boundary is:

```text
Tutor / Pedagogy / Orchestrator
            │
            │ semantic behavior intent
            ▼
      Nexa Behavior Protocol
            │
            ▼
       Behavior Engine
            │
            ▼
 Avatar / Speech / UI Runtime
```

The cognitive layer may request:

```text
explain
encouraging
look_at_diagram
point_at_object
speak_text
```

It SHALL NOT request:

```text
play animation 147
rotate bone arm_r by 23 degrees
set Live2D ParamAngleX = 0.42
```

---

# 3. Protocol Objectives

NBP SHALL provide:

1. technology independence;
2. deterministic message structure;
3. semantic behavior representation;
4. validation;
5. versioning;
6. extensibility;
7. observability;
8. graceful degradation;
9. synchronization of speech and behavior;
10. support for future 2D, 3D, VR, and AR runtimes.

---

# 4. Message Categories

NBP v1.0 defines the following primary message classes:

```text
behavior.command
behavior.update
behavior.cancel

speech.command
speech.update
speech.cancel

gaze.command
gesture.command
expression.command

canvas.command

runtime.state
runtime.capabilities
runtime.error
runtime.ack
```

The preferred high-level interface is `behavior.command`.

Lower-level commands exist for specialized runtime control.

---

# 5. Standard Envelope

Every NBP message SHALL use a common envelope.

```json
{
  "nbp_version": "1.0",
  "message_id": "018f5e91-16bd-7e67-83df-9d11f18444a1",
  "message_type": "behavior.command",
  "timestamp": "2026-08-17T23:30:00Z",
  "session_id": "018f5e90-acde-7a45-b369-102662c8112a",
  "sequence": 1042,
  "source": "nexa.orchestrator",
  "target": "nexa.avatar.primary",
  "correlation_id": null,
  "payload": {}
}
```

---

# 6. Envelope Fields

## 6.1 `nbp_version`

Required.

```json
"nbp_version": "1.0"
```

Identifies the protocol version used by the message.

---

## 6.2 `message_id`

Required.

Globally unique identifier for the message.

UUIDv7 is recommended where supported.

---

## 6.3 `message_type`

Required.

Identifies the payload schema.

Example:

```json
"message_type": "behavior.command"
```

---

## 6.4 `timestamp`

Required.

ISO-8601 UTC timestamp.

---

## 6.5 `session_id`

Required for user-facing runtime behavior.

Associates the behavior with an active tutoring session.

---

## 6.6 `sequence`

Recommended.

Monotonically increasing sequence number within a session or transport stream.

Useful for:

* ordering;
* debugging;
* replay;
* dropped-message detection.

---

## 6.7 `source`

Required.

Logical originating subsystem.

Example:

```text
nexa.orchestrator
nexa.behavior
nexa.speech
nexa.canvas
```

---

## 6.8 `target`

Optional.

Logical receiving subsystem.

If omitted, transport-specific routing may determine the consumer.

---

## 6.9 `correlation_id`

Optional.

Links messages belonging to the same logical operation.

For example:

```text
behavior.command
speech.command
canvas.command
runtime.ack
```

may all share a single correlation ID.

---

# 7. `behavior.command`

The primary behavior instruction is:

```json
{
  "message_type": "behavior.command",
  "payload": {
    "behavior_id": "beh_018f5e...",
    "state": "explaining",
    "priority": 50,
    "interruptibility": "phrase_boundary",
    "emotion": {},
    "gaze": {},
    "gesture": {},
    "speech": {},
    "presentation": {},
    "timing": {}
  }
}
```

---

# 8. Behavior ID

Each executable behavior SHOULD contain a unique `behavior_id`.

This ID is used for:

* cancellation;
* state tracking;
* completion events;
* diagnostics;
* replay.

---

# 9. Behavioral State

Required.

Initial standard state enumeration:

```text
offline
initializing
idle
attentive
listening
processing
thinking
speaking
explaining
demonstrating
questioning
waiting
observing
evaluating
hinting
correcting
encouraging
celebrating
warning
summarizing
debugging
```

Unknown future states SHALL be handled gracefully.

---

# 10. Priority

Behavior commands SHOULD contain a priority value.

Recommended range:

```text
0   lowest
50  normal instructional behavior
100 highest
```

Example:

```json
"priority": 50
```

Higher-priority behaviors may preempt lower-priority behaviors according to runtime policy.

---

# 11. Interruptibility

Supported values:

```text
immediate
word_boundary
phrase_boundary
sentence_boundary
non_interruptible
```

Example:

```json
"interruptibility": "phrase_boundary"
```

This is especially important for speech synchronization.

---

# 12. Emotion Object

```json
{
  "emotion": {
    "preset": "encouraging",
    "valence": 0.72,
    "arousal": 0.42,
    "confidence": 0.88,
    "engagement": 0.91,
    "intensity": 0.65
  }
}
```

---

# 13. Emotional Dimensions

## Valence

Range:

```text
-1.0 to +1.0
```

## Arousal

Range:

```text
0.0 to 1.0
```

## Confidence

Range:

```text
0.0 to 1.0
```

## Engagement

Range:

```text
0.0 to 1.0
```

## Intensity

Range:

```text
0.0 to 1.0
```

---

# 14. Emotional Presets

Initial standard presets:

```text
neutral
curious
focused
thinking
encouraging
concerned
skeptical
serious
excited
celebrating
corrective
surprised
confused
```

A renderer MAY support additional presets.

Numerical emotional dimensions take precedence where conflicts occur.

---

# 15. Gaze Object

```json
{
  "gaze": {
    "target_type": "canvas_object",
    "target_id": "tcp.syn_ack",
    "intensity": 0.85,
    "duration_ms": 2400,
    "lead_time_ms": 200
  }
}
```

---

# 16. Standard Gaze Targets

```text
student
camera
terminal
code_editor
canvas
canvas_object
diagram
whiteboard
quiz
notification
environment_object
none
```

---

# 17. Gaze Lead Time

The avatar SHOULD look at an object before pointing toward it.

Example:

```text
gaze starts
   ↓ 200 ms
gesture starts
   ↓
speech references object
```

This produces more natural behavior.

---

# 18. Gesture Object

```json
{
  "gesture": {
    "type": "point",
    "target_id": "tcp.syn_ack",
    "hand": "right",
    "intensity": 0.6,
    "duration_ms": 1800
  }
}
```

---

# 19. Standard Gesture Vocabulary

NBP v1.0 defines:

```text
none
idle
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

Renderers MAY choose the closest available implementation.

---

# 20. Gesture Degradation

If a requested gesture is unsupported, the runtime SHOULD:

1. choose a compatible alternative;
2. emit a capability/degradation event if configured;
3. continue the behavior.

Example:

```text
requested: point
unsupported
fallback: open_hand
```

The entire response should not fail merely because one gesture is unavailable.

---

# 21. Speech Object

```json
{
  "speech": {
    "text": "The server responds with SYN-ACK.",
    "voice": "nexa_default",
    "style": "instructional",
    "pace": 1.0,
    "pitch": 0.0,
    "energy": 0.55,
    "allow_interruption": true,
    "emit_visemes": true
  }
}
```

---

# 22. Standard Speech Styles

```text
neutral
instructional
conversational
encouraging
questioning
serious
warning
excited
reflective
concise
```

The speech runtime MAY map styles to provider-specific synthesis controls.

---

# 23. Speech Pace

Recommended normalized range:

```text
0.5 very slow
1.0 normal
1.5 fast
```

Runtime-specific limits MAY differ.

---

# 24. Speech Pitch

Normalized relative pitch:

```text
-1.0 lower
 0.0 neutral
+1.0 higher
```

---

# 25. Speech Energy

Range:

```text
0.0 to 1.0
```

This represents expressive energy, not audio volume.

---

# 26. Viseme Events

When enabled, the speech runtime SHOULD emit:

```json
{
  "message_type": "speech.update",
  "payload": {
    "behavior_id": "beh_123",
    "speech_id": "speech_456",
    "event": "viseme",
    "viseme": "MBP",
    "start_ms": 120,
    "duration_ms": 75
  }
}
```

---

# 27. Standard Viseme Vocabulary

NBP v1.0 recommends:

```text
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
```

Adapters MAY translate more detailed phoneme sets into this vocabulary.

---

# 28. Presentation Object

The `presentation` object communicates synchronization with UI or training content.

```json
{
  "presentation": {
    "attention_target": "canvas.tcp_handshake",
    "highlight_target": "tcp.syn_ack",
    "highlight_style": "instructional",
    "duration_ms": 3000
  }
}
```

This SHOULD remain semantic rather than renderer-specific.

---

# 29. Timing Object

```json
{
  "timing": {
    "start": "immediate",
    "delay_ms": 0,
    "expected_duration_ms": 4200,
    "completion_policy": "speech_complete"
  }
}
```

---

# 30. Completion Policies

Supported policies:

```text
immediate
gesture_complete
speech_complete
presentation_complete
all_components_complete
explicit_cancel
```

---

# 31. Complete Behavior Example

```json
{
  "nbp_version": "1.0",
  "message_id": "018f5e91-16bd-7e67-83df-9d11f18444a1",
  "message_type": "behavior.command",
  "timestamp": "2026-08-17T23:30:00Z",
  "session_id": "018f5e90-acde-7a45-b369-102662c8112a",
  "sequence": 1042,
  "source": "nexa.orchestrator",
  "target": "nexa.avatar.primary",
  "correlation_id": "018f5e90-fc92-75a0-9899-8890489270ff",
  "payload": {
    "behavior_id": "beh_018f5e91",
    "state": "explaining",
    "priority": 50,
    "interruptibility": "phrase_boundary",
    "emotion": {
      "preset": "encouraging",
      "valence": 0.72,
      "arousal": 0.42,
      "confidence": 0.88,
      "engagement": 0.91,
      "intensity": 0.65
    },
    "gaze": {
      "target_type": "canvas_object",
      "target_id": "tcp.syn_ack",
      "intensity": 0.85,
      "duration_ms": 2400,
      "lead_time_ms": 200
    },
    "gesture": {
      "type": "point",
      "target_id": "tcp.syn_ack",
      "hand": "right",
      "intensity": 0.6,
      "duration_ms": 1800
    },
    "speech": {
      "text": "The server responds with SYN-ACK.",
      "voice": "nexa_default",
      "style": "instructional",
      "pace": 0.95,
      "pitch": 0.0,
      "energy": 0.55,
      "allow_interruption": true,
      "emit_visemes": true
    },
    "presentation": {
      "attention_target": "canvas.tcp_handshake",
      "highlight_target": "tcp.syn_ack",
      "highlight_style": "instructional",
      "duration_ms": 3000
    },
    "timing": {
      "start": "immediate",
      "delay_ms": 0,
      "expected_duration_ms": 4200,
      "completion_policy": "speech_complete"
    }
  }
}
```

---

# 32. Behavior Update

An active behavior MAY be modified.

```json
{
  "message_type": "behavior.update",
  "payload": {
    "behavior_id": "beh_018f5e91",
    "emotion": {
      "preset": "curious",
      "intensity": 0.5
    }
  }
}
```

Updates SHOULD preserve unspecified fields.

---

# 33. Behavior Cancellation

```json
{
  "message_type": "behavior.cancel",
  "payload": {
    "behavior_id": "beh_018f5e91",
    "reason": "student_interruption",
    "transition": "graceful"
  }
}
```

---

# 34. Cancellation Modes

```text
immediate
graceful
phrase_boundary
sentence_boundary
```

---

# 35. Runtime Acknowledgment

A runtime MAY acknowledge accepted commands.

```json
{
  "message_type": "runtime.ack",
  "payload": {
    "message_id": "018f5e91-16bd-7e67-83df-9d11f18444a1",
    "behavior_id": "beh_018f5e91",
    "status": "accepted"
  }
}
```

---

# 36. Runtime Status Values

```text
accepted
queued
started
completed
cancelled
rejected
degraded
failed
```

---

# 37. Runtime State Event

```json
{
  "message_type": "runtime.state",
  "payload": {
    "avatar_id": "nexa.primary",
    "state": "explaining",
    "behavior_id": "beh_018f5e91"
  }
}
```

---

# 38. Runtime Capabilities

Every avatar runtime SHOULD be able to advertise capabilities.

```json
{
  "message_type": "runtime.capabilities",
  "payload": {
    "avatar_id": "nexa.primary",
    "supports": {
      "speech": true,
      "visemes": true,
      "gaze": true,
      "facial_expression": true,
      "upper_body_gestures": true,
      "full_body_gestures": false,
      "canvas_pointing": true
    },
    "gestures": [
      "nod",
      "point",
      "open_hand",
      "thinking_chin"
    ],
    "expressions": [
      "neutral",
      "curious",
      "encouraging",
      "focused",
      "celebrating"
    ]
  }
}
```

---

# 39. Capability Negotiation

The Behavior Engine SHOULD consult runtime capabilities before dispatching specialized behavior.

Example:

```text
Behavior request
      ↓
Check runtime capabilities
      ↓
Supported?
 ┌────┴────┐
 yes       no
 │         │
 ▼         ▼
send     transform
          ↓
       fallback
```

---

# 40. Error Message

```json
{
  "message_type": "runtime.error",
  "payload": {
    "code": "NBP_GESTURE_UNSUPPORTED",
    "severity": "warning",
    "behavior_id": "beh_018f5e91",
    "message": "Requested gesture is unsupported.",
    "recoverable": true
  }
}
```

---

# 41. Error Severity

```text
debug
info
warning
error
fatal
```

---

# 42. Error Philosophy

Recoverable presentation errors SHALL NOT terminate a tutoring session.

For example:

* missing animation;
* unsupported gesture;
* unavailable hair physics;
* unavailable canvas highlight;

should normally degrade gracefully.

Failures affecting meaning require stronger handling.

Examples:

* speech generation failure;
* invalid behavior command;
* broken synchronization;
* unknown protocol version.

---

# 43. State Machine Enforcement

The Behavior Engine SHOULD validate legal state transitions.

Example:

```text
listening → thinking
```

is valid.

A direct transition such as:

```text
offline → celebrating
```

may be rejected or normalized through intermediate states.

---

# 44. Recommended Transition Graph

```text
OFFLINE
   ↓
INITIALIZING
   ↓
IDLE
   ↓
ATTENTIVE
   ├─────────────► LISTENING
   │                   ↓
   │               PROCESSING
   │                   ↓
   │                THINKING
   │                   ↓
   ├─────────────► EXPLAINING
   ├─────────────► QUESTIONING
   ├─────────────► DEMONSTRATING
   ├─────────────► CORRECTING
   ├─────────────► ENCOURAGING
   └─────────────► WARNING
                       ↓
                      IDLE
```

---

# 45. Behavior Composition

The renderer SHOULD compose behavior from independent channels.

```text
Behavior
│
├── state
├── emotion
├── gaze
├── face
├── body
├── gesture
├── speech
└── presentation
```

This allows, for example:

```text
state = explaining
emotion = curious
gesture = point
gaze = diagram
speech = active
```

without requiring a unique animation asset for every combination.

---

# 46. Channel Ownership

Recommended ownership:

```text
State        Behavior Engine
Emotion      Behavior Engine
Face         Avatar adapter
Gaze         Avatar adapter
Gesture      Avatar adapter
Lip sync     Speech/avatar adapter
Voice        Speech runtime
Canvas       Presentation runtime
```

---

# 47. Behavior Arbitration

Multiple requested behaviors may conflict.

The Behavior Engine SHALL arbitrate using:

1. priority;
2. current state;
3. interruptibility;
4. timing constraints;
5. safety;
6. pedagogical importance.

Example:

```text
idle gesture priority 10
student interruption priority 80
warning priority 90
```

Higher-priority behavior wins.

---

# 48. Behavior Queue

A runtime MAY maintain:

```text
active behavior
high-priority queue
normal queue
background behavior queue
```

Background behaviors include:

* blinking;
* breathing;
* subtle gaze;
* idle posture changes.

These should not block instructional actions.

---

# 49. Background Behaviors

Background behavior SHALL remain separate from explicit instructional behavior.

Example:

```text
explicit:
  explain + point

background:
  breathing
  blink
  hair physics
```

---

# 50. Student Interruption Flow

```text
Nexa speaking
    ↓
student.speech.started
    ↓
Orchestrator decides interruption
    ↓
behavior.cancel
    ↓
speech stops at permitted boundary
    ↓
avatar returns to attentive/listening
```

---

# 51. Canvas Command

NBP may coordinate visual instructional content.

```json
{
  "message_type": "canvas.command",
  "payload": {
    "action": "highlight",
    "target_id": "tcp.syn_ack",
    "style": "instructional",
    "duration_ms": 2500
  }
}
```

---

# 52. Standard Canvas Actions

```text
show
hide
focus
highlight
annotate
clear_annotation
zoom
pan
step
reset
```

---

# 53. Semantic Target IDs

Target IDs SHOULD describe conceptual objects.

Prefer:

```text
tcp.syn_ack
osi.transport_layer
rust.borrow_checker
diagram.node.server
```

Avoid:

```text
button47
rect_09
object_18274
```

Adapters MAY internally map semantic IDs to runtime object IDs.

---

# 54. Behavior Templates

Frequently used behaviors MAY be stored as templates.

Example:

```yaml
template: nexa.explain.diagram
defaults:
  state: explaining
  emotion:
    preset: focused
  gaze:
    target_type: canvas_object
  gesture:
    type: point
  speech:
    style: instructional
```

The orchestrator provides only changing values.

---

# 55. Template Expansion

Input:

```json
{
  "template": "nexa.explain.diagram",
  "target_id": "tcp.syn_ack",
  "speech": {
    "text": "This packet acknowledges the client's SYN."
  }
}
```

Expanded internally into a complete NBP behavior.

This reduces repetitive generation.

---

# 56. LLM Boundary

The LLM SHOULD NOT generate arbitrary NBP JSON directly without validation.

Recommended sequence:

```text
LLM
 ↓
Typed Tutor Response
 ↓
Schema Validation
 ↓
Behavior Planner
 ↓
NBP Builder
 ↓
Protocol Validation
 ↓
Behavior Engine
```

This protects the runtime from malformed model output.

---

# 57. Typed Tutor Response

A higher-level response may look like:

```json
{
  "speech": "Exactly. The server now acknowledges the SYN.",
  "intent": "explain",
  "emotion": "encouraging",
  "attention": {
    "type": "diagram",
    "target": "tcp.syn_ack"
  },
  "gesture": "point"
}
```

This is then converted to NBP.

---

# 58. Security Boundary

NBP SHALL NOT permit arbitrary runtime code execution.

Prohibited examples:

```text
execute_script
eval_javascript
run_shell_command
load_arbitrary_plugin
```

NBP describes presentation behavior only.

Tools and lab execution SHALL use separate controlled protocols.

---

# 59. Data Minimization

NBP messages SHOULD NOT contain unnecessary learner information.

The avatar runtime usually does not need:

* learner name;
* student history;
* grades;
* private profile information.

It only requires enough information to render behavior.

---

# 60. Transport Independence

NBP is a logical protocol and MAY be transported over:

```text
in-process channels
WebSocket
QUIC
TCP
IPC
named pipes
Unix domain sockets
message broker
shared-memory queue
```

The protocol SHALL NOT depend on one transport.

---

# 61. Serialization

NBP v1.0 SHALL define JSON as the canonical human-readable representation.

Future binary encodings MAY include:

* MessagePack;
* CBOR;
* Protobuf.

Binary encodings SHALL preserve the same logical schema.

---

# 62. Schema Validation

Every message SHOULD be validated before execution.

Validation includes:

```text
required fields
type correctness
enum correctness
range validation
protocol version
message schema
state validity
```

---

# 63. Unknown Fields

NBP consumers SHOULD ignore unknown optional fields where safe.

This supports forward compatibility.

Unknown required semantics SHOULD generate a validation error.

---

# 64. Protocol Versioning

Version format:

```text
MAJOR.MINOR
```

Example:

```text
1.0
1.1
2.0
```

---

# 65. Compatibility Rules

Minor versions SHOULD remain backward compatible.

Example:

```text
1.0 consumer
```

may safely process a `1.1` message if unknown optional fields are ignored.

Major versions MAY contain breaking changes.

---

# 66. Feature Extensions

Vendor/runtime-specific extensions MUST use namespaces.

Example:

```json
{
  "extensions": {
    "live2d.physics_hint": {},
    "unity.camera_hint": {}
  }
}
```

Core cognitive systems SHOULD NOT depend on extensions.

---

# 67. Diagnostics

Runtime diagnostics SHOULD record:

```text
message_id
behavior_id
state
start time
completion time
degradation
error
latency
speech timing
gesture timing
```

This allows precise analysis of avatar behavior.

---

# 68. Replay

NBP message streams SHOULD be recordable.

A recorded sequence can be replayed without invoking the LLM.

This enables:

* regression testing;
* animation tuning;
* demonstrations;
* debugging;
* deterministic QA.

---

# 69. Deterministic Test Mode

The runtime SHOULD support a deterministic mode that disables or seeds randomized behaviors such as:

* blinking;
* idle gestures;
* gaze jitter;
* physics randomness.

This is useful for automated tests.

---

# 70. Example: Questioning

```json
{
  "message_type": "behavior.command",
  "payload": {
    "behavior_id": "beh_question_001",
    "state": "questioning",
    "emotion": {
      "preset": "curious",
      "intensity": 0.55
    },
    "gaze": {
      "target_type": "student"
    },
    "gesture": {
      "type": "head_tilt"
    },
    "speech": {
      "text": "What do you think the server sends next?",
      "style": "questioning",
      "allow_interruption": true
    },
    "timing": {
      "completion_policy": "speech_complete"
    }
  }
}
```

---

# 71. Example: Correction

```json
{
  "message_type": "behavior.command",
  "payload": {
    "behavior_id": "beh_correct_001",
    "state": "correcting",
    "emotion": {
      "preset": "encouraging",
      "valence": 0.35,
      "confidence": 0.9
    },
    "gaze": {
      "target_type": "student"
    },
    "gesture": {
      "type": "small_nod"
    },
    "speech": {
      "text": "You're close. The ACK is correct, but we're missing one flag.",
      "style": "encouraging"
    }
  }
}
```

---

# 72. Example: Celebration

```json
{
  "message_type": "behavior.command",
  "payload": {
    "behavior_id": "beh_celebrate_001",
    "state": "celebrating",
    "emotion": {
      "preset": "celebrating",
      "intensity": 0.78
    },
    "gesture": {
      "type": "celebrate"
    },
    "speech": {
      "text": "Exactly. And you reasoned through it instead of guessing.",
      "style": "excited"
    }
  }
}
```

---

# 73. Example: Warning

```json
{
  "message_type": "behavior.command",
  "payload": {
    "behavior_id": "beh_warning_001",
    "state": "warning",
    "priority": 90,
    "emotion": {
      "preset": "serious",
      "intensity": 0.75
    },
    "gaze": {
      "target_type": "student"
    },
    "gesture": {
      "type": "attention"
    },
    "speech": {
      "text": "That command will recursively delete the target directory.",
      "style": "warning",
      "pace": 0.85
    }
  }
}
```

---

# 74. Minimum Runtime Implementation

An NBP-compliant MVP runtime MUST support:

```text
behavior.command
behavior.cancel
runtime.ack
runtime.state
runtime.error

states:
  idle
  attentive
  listening
  thinking
  speaking
  explaining

emotion:
  neutral
  focused
  encouraging

gaze:
  student
  camera
  canvas_object

gestures:
  idle
  nod
  point

speech:
  text
  style
  interruption
  viseme emission
```

---

# 75. MVP Acceptance Test

Given:

```text
Student asks:
"What is the TCP three-way handshake?"
```

The orchestrator SHALL be capable of producing a valid sequence approximately equivalent to:

```text
attentive
↓
listening
↓
thinking
↓
canvas.show(tcp_handshake)
↓
explaining + point(SYN)
↓
explaining + point(SYN_ACK)
↓
explaining + point(ACK)
↓
questioning
↓
waiting
```

Each transition SHALL be represented using valid NBP messages.

---

# 76. Rust Domain Model Direction

The core Rust representation should resemble:

```rust
pub struct NbpMessage<T> {
    pub nbp_version: ProtocolVersion,
    pub message_id: MessageId,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub session_id: SessionId,
    pub sequence: u64,
    pub source: EndpointId,
    pub target: Option<EndpointId>,
    pub correlation_id: Option<CorrelationId>,
    pub payload: T,
}
```

Behavior payload:

```rust
pub struct BehaviorCommand {
    pub behavior_id: BehaviorId,
    pub state: BehaviorState,
    pub priority: u8,
    pub interruptibility: Interruptibility,
    pub emotion: Option<Emotion>,
    pub gaze: Option<Gaze>,
    pub gesture: Option<Gesture>,
    pub speech: Option<Speech>,
    pub presentation: Option<Presentation>,
    pub timing: Option<Timing>,
}
```

Enums SHOULD be strongly typed.

---

# 77. Recommended Crate

The protocol should live in an independent package:

```text
crates/
└── nexa-nbp/
    ├── src/
    │   ├── lib.rs
    │   ├── envelope.rs
    │   ├── behavior.rs
    │   ├── emotion.rs
    │   ├── gaze.rs
    │   ├── gesture.rs
    │   ├── speech.rs
    │   ├── canvas.rs
    │   ├── runtime.rs
    │   ├── error.rs
    │   └── version.rs
    ├── schemas/
    └── tests/
```

`nexa-nbp` SHOULD have minimal dependencies and no dependency on:

* LLM frameworks;
* GUI frameworks;
* avatar engines;
* speech providers;
* course systems.

---

# 78. Architectural Invariants

NBP v1.0 establishes these invariants:

1. NBP expresses semantic behavior.
2. NBP SHALL remain rendering-engine independent.
3. NBP SHALL remain model-provider independent.
4. NBP SHALL not execute arbitrary code.
5. Behavior messages SHALL be schema validated.
6. Behavior channels SHALL be composable.
7. Unsupported presentation capabilities SHOULD degrade gracefully.
8. Speech SHALL support interruption.
9. Runtime capabilities SHOULD be discoverable.
10. Messages SHOULD be observable and replayable.
11. Unknown optional fields SHOULD support forward compatibility.
12. Cognitive systems SHOULD interact with typed abstractions rather than raw runtime commands.
13. Student-private information SHOULD remain outside NBP unless explicitly necessary.
14. NBP SHALL remain independent of transport.
15. Protocol evolution SHALL follow explicit versioning.

---

# 79. Next Engineering Specification

The next document should be:

**NEXA-EVT-001 — Nexa Event Model & Runtime Event Bus Specification v1.0**

That specification will define how the rest of the platform communicates, including:

```text
student events
session events
tutor events
lesson events
assessment events
lab events
tool events
competency events
speech events
avatar events
errors
telemetry
correlation
event ordering
replay
persistence
```

NBP then becomes one specialized protocol operating on top of that broader event-driven architecture.
