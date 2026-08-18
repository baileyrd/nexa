# NEXA-EVT-001 — Nexa Event Model & Runtime Event Bus Specification v1.0

**Specification ID:** NEXA-EVT-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Purpose:** Define the canonical event model, runtime event bus, event taxonomy, ordering semantics, persistence rules, replay behavior, observability, and subsystem integration model for Nexa.

---

## 1. Purpose

Nexa is designed as an event-driven system.

The **Nexa Event Model** defines how independently implemented subsystems communicate without requiring direct knowledge of one another.

The event model supports:

* session orchestration;
* student interaction;
* tutor reasoning;
* pedagogy;
* speech;
* avatar behavior;
* lessons;
* labs;
* assessments;
* competency tracking;
* knowledge retrieval;
* tools;
* telemetry;
* persistence;
* replay;
* debugging.

The architectural objective is to prevent the platform from evolving into a tightly coupled application in which every subsystem directly calls every other subsystem.

The preferred model is:

```text
Producer
   │
   ▼
Event Bus
   │
   ├────► Subscriber A
   ├────► Subscriber B
   ├────► Subscriber C
   └────► Persistence / Telemetry
```

---

# 2. Architectural Principle

Subsystems SHOULD communicate using:

```text
facts
commands
responses
state transitions
```

rather than implementation-specific method calls across architectural boundaries.

For example:

```text
student.answer.submitted
```

is preferable to:

```text
student_model.update_from_quiz_form_2(...)
```

The first describes what happened.

The second exposes implementation details.

---

# 3. Event Bus Position in the Architecture

```text
                          ┌─────────────────┐
                          │      USER       │
                          └────────┬────────┘
                                   │
                                   ▼
                             INPUT SYSTEMS
                                   │
                                   ▼
┌────────────────────────────────────────────────────────────┐
│                    NEXA EVENT BUS                          │
└────────────────────────────────────────────────────────────┘
       │            │            │            │
       ▼            ▼            ▼            ▼
 Orchestrator    Tutor       Pedagogy      Student
                                     
       │            │            │            │
       ▼            ▼            ▼            ▼
    Speech        Avatar       Lessons       Labs

       │            │            │            │
       └────────────┴──────┬─────┴────────────┘
                           ▼
                  Persistence / Replay
```

---

# 4. Event Definition

An **event** is an immutable record describing something that occurred within the Nexa system.

Examples:

```text
session.started
student.speech.started
student.answer.submitted
lesson.started
lab.command.executed
tutor.response.completed
speech.completed
avatar.state.changed
competency.updated
```

Once published, an event SHALL NOT be modified.

Corrections SHALL be represented as new events.

---

# 5. Commands Versus Events

Nexa SHALL distinguish between commands and events.

## Command

A request for something to happen.

Example:

```text
speech.synthesize
```

## Event

A statement that something happened.

Example:

```text
speech.synthesis.completed
```

This distinction is fundamental.

---

# 6. Command/Event Flow

```text
Tutor
  │
  │ speech.synthesize
  ▼
Speech Service
  │
  │ speech.synthesis.started
  ▼
Event Bus
  │
  │ speech.synthesis.completed
  ▼
Orchestrator
```

Commands may fail.

Events describe outcomes.

---

# 7. Canonical Event Envelope

All runtime events SHALL use a standard envelope.

```json
{
  "event_version": "1.0",
  "event_id": "0193f24a-9c41-7a4d-b64f-b6fa5b76f001",
  "event_type": "student.answer.submitted",
  "timestamp": "2026-08-17T23:45:00Z",
  "session_id": "0193f249-95c2-79e0-a221-628003c28501",
  "sequence": 1842,
  "source": "nexa.training_ui",
  "subject": "student.current",
  "correlation_id": "0193f249-c239-7434-a223-c1af15431322",
  "causation_id": "0193f249-ba42-7210-8aca-f69323269d50",
  "trace_id": "0193f249-dc15-79e2-9879-b96452e04511",
  "payload": {},
  "metadata": {}
}
```

---

# 8. Envelope Fields

## 8.1 `event_version`

Schema version for the event envelope.

Example:

```json
"event_version": "1.0"
```

---

## 8.2 `event_id`

Globally unique event identifier.

UUIDv7 is recommended.

The identifier SHALL never be reused.

---

## 8.3 `event_type`

Canonical hierarchical event name.

Example:

```text
student.answer.submitted
```

---

## 8.4 `timestamp`

UTC ISO-8601 timestamp indicating when the event occurred.

---

## 8.5 `session_id`

Identifies the training session associated with the event.

Not all administrative events require a session ID.

---

## 8.6 `sequence`

Monotonically increasing ordering value within a defined event stream.

Sequence numbers help detect:

* duplicates;
* gaps;
* out-of-order delivery.

---

## 8.7 `source`

Logical subsystem that emitted the event.

Example:

```text
nexa.orchestrator
nexa.tutor
nexa.speech
nexa.avatar
nexa.lab
```

---

## 8.8 `subject`

Entity primarily associated with the event.

Example:

```text
student.current
lesson.networking.001
lab.linux.004
avatar.nexa.primary
```

---

## 8.9 `correlation_id`

Groups all messages associated with one logical operation.

For example:

```text
student asks question
       ↓
retrieval
       ↓
tutor generation
       ↓
speech
       ↓
avatar behavior
```

may share one correlation ID.

---

## 8.10 `causation_id`

References the event or command that directly caused the current event.

This enables causal reconstruction.

---

## 8.11 `trace_id`

Tracks a workflow across multiple services and processes.

The same trace ID MAY span:

* client;
* orchestrator;
* model service;
* TTS;
* avatar;
* persistence.

---

## 8.12 `payload`

Event-specific data.

---

## 8.13 `metadata`

Non-domain operational metadata.

Possible contents:

```text
runtime version
host
process ID
schema hash
latency
environment
transport
```

Business semantics SHOULD remain in `payload`.

---

# 9. Event Naming Standard

Event names SHALL use:

```text
domain.entity.action
```

or:

```text
domain.action
```

Examples:

```text
session.started
student.answer.submitted
lesson.objective.completed
speech.synthesis.failed
avatar.gesture.completed
```

Names SHALL use lowercase ASCII and dots.

---

# 10. Event Taxonomy

The initial domain taxonomy is:

```text
system.*
session.*
student.*
input.*
tutor.*
pedagogy.*
knowledge.*
memory.*
lesson.*
assessment.*
competency.*
lab.*
tool.*
speech.*
avatar.*
canvas.*
orchestrator.*
security.*
telemetry.*
```

---

# 11. System Events

```text
system.started
system.ready
system.degraded
system.shutdown.requested
system.shutdown.started
system.stopped
system.error
```

Example:

```json
{
  "event_type": "system.ready",
  "payload": {
    "runtime_version": "0.1.0",
    "components_ready": 12
  }
}
```

---

# 12. Session Events

```text
session.created
session.started
session.paused
session.resumed
session.ended
session.failed
session.timeout
session.state.changed
```

Session events establish the lifecycle of a tutoring interaction.

---

# 13. Session Start Event

```json
{
  "event_type": "session.started",
  "payload": {
    "course_id": "networking-fundamentals",
    "lesson_id": "tcp-handshake",
    "student_id": "student-001",
    "mode": "guided_instruction"
  }
}
```

---

# 14. Student Events

```text
student.connected
student.disconnected

student.input.started
student.input.completed

student.speech.started
student.speech.partial
student.speech.completed

student.text.submitted

student.answer.submitted
student.answer.evaluated
student.answer.correct
student.answer.incorrect
student.answer.partial

student.question.asked

student.hint.requested
student.explanation.requested
student.repeat.requested

student.action.observed
student.preference.updated
```

---

# 15. Student Answer Event

```json
{
  "event_type": "student.answer.submitted",
  "payload": {
    "question_id": "tcp-q-001",
    "answer_type": "text",
    "answer": "SYN ACK",
    "self_confidence": 0.74
  }
}
```

---

# 16. Confidence Capture

Where appropriate, the system SHOULD permit the student to provide confidence.

Example:

```text
0.0 = pure guess
1.0 = completely confident
```

This enables Nexa to distinguish:

```text
correct + confident
correct + uncertain
incorrect + confident
incorrect + uncertain
```

---

# 17. Tutor Events

```text
tutor.request.received
tutor.context.prepared

tutor.reasoning.started
tutor.reasoning.completed

tutor.response.started
tutor.response.partial
tutor.response.completed
tutor.response.cancelled
tutor.response.failed

tutor.tool.requested
tutor.retrieval.requested
```

Internal private model reasoning SHALL NOT be exposed through these events.

Events should describe observable processing state and structured outputs.

---

# 18. Tutor Response Event

```json
{
  "event_type": "tutor.response.completed",
  "payload": {
    "response_id": "resp-00931",
    "intent": "explain",
    "speech": "The server responds with SYN-ACK.",
    "behavior": {
      "emotion": "focused",
      "gesture": "point",
      "attention_target": "tcp.syn_ack"
    }
  }
}
```

---

# 19. Pedagogy Events

```text
pedagogy.strategy.selected
pedagogy.strategy.changed

pedagogy.hint.selected
pedagogy.hint.escalated

pedagogy.difficulty.increased
pedagogy.difficulty.decreased

pedagogy.intervention.started
pedagogy.intervention.completed

pedagogy.mastery.detected
pedagogy.misconception.detected
```

---

# 20. Misconception Event

```json
{
  "event_type": "pedagogy.misconception.detected",
  "payload": {
    "concept_id": "networking.tcp.handshake",
    "misconception": "student believes ACK is sent before SYN-ACK",
    "confidence": 0.86,
    "evidence_event_ids": [
      "evt-1001",
      "evt-1007"
    ]
  }
}
```

---

# 21. Knowledge Events

```text
knowledge.query.requested
knowledge.query.completed
knowledge.query.failed

knowledge.document.retrieved
knowledge.concept.retrieved
knowledge.source.selected
knowledge.source.rejected
```

---

# 22. Retrieval Event

```json
{
  "event_type": "knowledge.query.completed",
  "payload": {
    "query_id": "qry-783",
    "result_count": 6,
    "latency_ms": 43,
    "sources": [
      "rfc793",
      "course.networking.tcp"
    ]
  }
}
```

---

# 23. Memory Events

```text
memory.read.requested
memory.read.completed

memory.write.requested
memory.write.completed

memory.updated
memory.expired
memory.deleted
```

Student memory and knowledge memory SHALL remain separate domains.

---

# 24. Lesson Events

```text
lesson.loaded
lesson.started
lesson.paused
lesson.resumed
lesson.completed
lesson.failed

lesson.step.started
lesson.step.completed

lesson.objective.started
lesson.objective.completed
lesson.objective.failed

lesson.branch.selected
```

---

# 25. Lesson Objective Event

```json
{
  "event_type": "lesson.objective.completed",
  "payload": {
    "objective_id": "tcp.handshake.identify_sequence",
    "evidence": [
      "assessment:q3",
      "lab:tcp-observation"
    ]
  }
}
```

---

# 26. Assessment Events

```text
assessment.started
assessment.question.presented
assessment.answer.submitted
assessment.answer.evaluated
assessment.hint.used
assessment.completed
assessment.failed
```

---

# 27. Assessment Completion Event

```json
{
  "event_type": "assessment.completed",
  "payload": {
    "assessment_id": "tcp-checkpoint-1",
    "score": 0.84,
    "mastery_estimate": 0.77,
    "attempt": 2
  }
}
```

---

# 28. Competency Events

```text
competency.created
competency.evidence.added
competency.updated
competency.mastered
competency.regressed
competency.expired
```

---

# 29. Competency Update Event

```json
{
  "event_type": "competency.updated",
  "payload": {
    "competency_id": "networking.tcp.handshake",
    "previous_value": 0.61,
    "new_value": 0.74,
    "evidence_type": "assessment",
    "evidence_id": "tcp-checkpoint-1"
  }
}
```

---

# 30. Lab Events

```text
lab.created
lab.started
lab.ready
lab.paused
lab.reset
lab.completed
lab.failed
lab.destroyed

lab.command.requested
lab.command.executed
lab.command.failed

lab.file.created
lab.file.modified
lab.file.deleted

lab.error.detected
lab.objective.completed
```

---

# 31. Lab Command Event

```json
{
  "event_type": "lab.command.executed",
  "payload": {
    "lab_id": "linux-net-001",
    "command": "ss -tulpn",
    "exit_code": 0,
    "duration_ms": 18,
    "stdout_ref": "artifact://lab/run/00981/stdout",
    "stderr_ref": null
  }
}
```

Large outputs SHOULD be referenced rather than embedded directly.

---

# 32. Tool Events

```text
tool.registered
tool.available
tool.unavailable

tool.execution.requested
tool.execution.started
tool.execution.completed
tool.execution.failed
tool.execution.cancelled
```

---

# 33. Tool Execution Event

```json
{
  "event_type": "tool.execution.completed",
  "payload": {
    "tool_id": "python.runtime",
    "request_id": "toolreq-183",
    "status": "success",
    "duration_ms": 67,
    "result_ref": "artifact://tool/183/result"
  }
}
```

---

# 34. Speech Events

```text
speech.capture.started
speech.capture.completed

speech.transcription.started
speech.transcription.partial
speech.transcription.completed
speech.transcription.failed

speech.synthesis.requested
speech.synthesis.started
speech.synthesis.partial
speech.synthesis.completed
speech.synthesis.failed
speech.synthesis.cancelled

speech.playback.started
speech.playback.completed
speech.playback.cancelled

speech.viseme.emitted
```

---

# 35. Viseme Event

```json
{
  "event_type": "speech.viseme.emitted",
  "payload": {
    "speech_id": "sp-2831",
    "viseme": "MBP",
    "offset_ms": 284,
    "duration_ms": 73
  }
}
```

---

# 36. Avatar Events

```text
avatar.loaded
avatar.ready
avatar.state.changed

avatar.expression.started
avatar.expression.completed

avatar.gaze.started
avatar.gaze.completed

avatar.gesture.started
avatar.gesture.completed

avatar.behavior.started
avatar.behavior.completed
avatar.behavior.cancelled
avatar.behavior.degraded
avatar.behavior.failed
```

---

# 37. Avatar State Event

```json
{
  "event_type": "avatar.state.changed",
  "payload": {
    "avatar_id": "nexa.primary",
    "previous_state": "thinking",
    "new_state": "explaining",
    "behavior_id": "beh-9812"
  }
}
```

---

# 38. Canvas Events

```text
canvas.object.shown
canvas.object.hidden
canvas.object.highlighted
canvas.annotation.created
canvas.annotation.removed
canvas.focus.changed
canvas.reset
```

---

# 39. Orchestrator Events

```text
orchestrator.started
orchestrator.command.received
orchestrator.workflow.started
orchestrator.workflow.completed
orchestrator.workflow.failed
orchestrator.state.changed
```

---

# 40. Security Events

```text
security.policy.checked
security.action.allowed
security.action.denied
security.sandbox.violation
security.permission.changed
security.audit.created
```

Security events SHALL avoid unnecessarily exposing secrets.

---

# 41. Telemetry Events

```text
telemetry.latency.recorded
telemetry.metric.recorded
telemetry.error.recorded
telemetry.resource.recorded
```

Telemetry SHALL remain distinguishable from learning-domain events.

---

# 42. Event Immutability

Published events SHALL be immutable.

If an event contained incorrect information:

```text
event A
```

is retained.

A corrective event is added:

```text
event.corrected
```

or an appropriate domain-specific update event.

This provides auditability.

---

# 43. Event Ordering

Global event ordering is not required.

Ordering SHALL be guaranteed only within clearly defined streams where necessary.

Typical streams:

```text
session
student interaction
speech operation
avatar behavior
lab execution
assessment attempt
```

---

# 44. Sequence Numbers

Each ordered stream SHOULD use monotonically increasing sequence numbers.

Example:

```text
1840 student.speech.started
1841 student.speech.completed
1842 tutor.response.started
1843 tutor.response.completed
1844 speech.synthesis.started
```

---

# 45. Out-of-Order Events

Consumers SHOULD tolerate minor out-of-order delivery where the transport permits it.

Possible responses include:

```text
buffer
reorder
ignore stale event
request state synchronization
```

Behavior depends on event type.

---

# 46. Delivery Semantics

The event infrastructure SHOULD initially target:

```text
at-least-once delivery
```

rather than relying on exactly-once semantics.

Consumers therefore SHALL be designed to be idempotent where practical.

---

# 47. Duplicate Events

Duplicate detection SHOULD use:

```text
event_id
```

Consumers MAY maintain a processed-event cache.

---

# 48. Idempotency

Operations triggered from events SHOULD avoid duplicate side effects.

For example:

```text
competency.updated
```

processed twice SHALL NOT double-apply the competency change.

---

# 49. Event Persistence

Events SHOULD be categorized as:

```text
transient
session-persistent
long-term-persistent
audit
```

---

# 50. Transient Events

Examples:

```text
speech.viseme.emitted
avatar.gaze.started
avatar.expression.started
```

These may not require durable storage in production.

---

# 51. Session-Persistent Events

Examples:

```text
student.answer.submitted
tutor.response.completed
lesson.step.completed
lab.command.executed
```

These are valuable for replay and troubleshooting.

---

# 52. Long-Term Learning Events

Examples:

```text
competency.updated
lesson.completed
assessment.completed
pedagogy.misconception.detected
```

These SHOULD persist as part of the learner's training history.

---

# 53. Audit Events

Examples:

```text
security.action.denied
assessment.certification.completed
administrative policy change
```

Audit storage policies MAY differ from normal event storage.

---

# 54. Event Store

The event store SHOULD support:

```text
append
query by session
query by student
query by event type
query by time
query by correlation ID
query by trace ID
replay
```

Events SHOULD normally be append-only.

---

# 55. Event Replay

Replay SHALL enable reconstruction of a past session.

```text
Recorded events
      ↓
Replay engine
      ↓
State projections
      ↓
UI / diagnostics / simulation
```

Replay MAY exclude non-deterministic external actions unless explicitly simulated.

---

# 56. Replay Modes

Recommended modes:

```text
full
logical
avatar_only
learning_only
diagnostic
```

### Full

Attempts to reproduce the entire session.

### Logical

Rebuilds system state without rendering media.

### Avatar Only

Replays Nexa's behavioral events.

### Learning Only

Reconstructs learner progression.

### Diagnostic

Emphasizes timing and failure events.

---

# 57. Event-Sourced State

Certain Nexa components MAY derive their state from event history.

Candidate components include:

```text
session state
lesson progression
assessment attempts
competency history
```

Not every subsystem must use full event sourcing.

---

# 58. State Projections

A projection converts events into efficient current-state views.

Example:

```text
competency.evidence.added
competency.updated
competency.mastered
           │
           ▼
Current Competency Projection
           │
           ▼
tcp.handshake = 0.92
```

---

# 59. Snapshotting

For long event streams, projections MAY create snapshots.

Example:

```text
events 1–100,000
      ↓
snapshot
      ↓
events 100,001–100,340
```

This prevents rebuilding all state from the beginning.

---

# 60. Event Bus Responsibilities

The bus is responsible for:

* accepting events;
* validating envelopes;
* routing events;
* applying subscriptions;
* propagating correlation context;
* supporting observability;
* handling backpressure;
* optionally persisting events.

The bus SHALL NOT contain business logic.

---

# 61. Subscriber Responsibilities

Subscribers SHALL:

* validate supported payload schemas;
* handle duplicates where applicable;
* avoid blocking the bus indefinitely;
* report processing failures;
* preserve trace context.

---

# 62. Subscription Patterns

Subscribers MAY subscribe by:

```text
exact event type
event prefix
domain
subject
session
```

Examples:

```text
student.answer.submitted

student.*

speech.*

session_id == abc123
```

---

# 63. Wildcard Subscription

Development tooling MAY use wildcard subscriptions.

Example:

```text
*
```

Production use SHOULD be limited because of volume.

---

# 64. Backpressure

High-frequency event sources such as:

```text
speech partial transcription
visemes
animation telemetry
```

may overwhelm slower subscribers.

The system SHALL support backpressure strategies.

---

# 65. Backpressure Strategies

Possible strategies:

```text
buffer
sample
drop
coalesce
throttle
persist and replay
```

Policies SHALL be event-specific.

A competency event must not be treated the same way as an eye-blink telemetry event.

---

# 66. Event Priority

Events MAY carry operational priority.

Recommended priorities:

```text
critical
high
normal
low
telemetry
```

Examples:

```text
security.sandbox.violation → critical
student.answer.submitted   → high
lesson.step.completed      → normal
avatar.gaze.started        → low
telemetry.metric.recorded  → telemetry
```

---

# 67. Command Envelope

Commands SHOULD use a structure closely aligned with events.

```json
{
  "command_version": "1.0",
  "command_id": "0193f260-...",
  "command_type": "speech.synthesize",
  "timestamp": "2026-08-17T23:50:00Z",
  "session_id": "0193f249-...",
  "source": "nexa.orchestrator",
  "target": "nexa.speech",
  "correlation_id": "0193f249-...",
  "causation_id": "0193f248-...",
  "trace_id": "0193f247-...",
  "payload": {}
}
```

---

# 68. Commands Are Not Facts

Consumers SHALL NOT treat a command as evidence that something happened.

For example:

```text
lab.command.execute
```

does not mean the command was successfully executed.

Only:

```text
lab.command.executed
```

establishes that fact.

---

# 69. Request/Response Pattern

Where direct request/response semantics are appropriate:

```text
command
  ↓
accepted/rejected
  ↓
started
  ↓
completed/failed
```

Example:

```text
speech.synthesize
speech.synthesis.started
speech.synthesis.completed
```

---

# 70. Timeouts

Commands requiring completion SHOULD declare or inherit a timeout policy.

Timeout produces an event such as:

```text
tool.execution.timeout
```

or:

```text
orchestrator.workflow.failed
```

depending on scope.

---

# 71. Cancellation

Long-running operations SHOULD support cancellation.

Typical cancellation targets:

```text
tutor generation
speech synthesis
speech playback
avatar behavior
tool operation
lab operation
retrieval
```

Cancellation SHALL itself produce observable events.

---

# 72. Correlation Example

Student asks a question:

```text
student.question.asked
          │
          │ correlation = Q123
          ▼
knowledge.query.requested
          │
          ▼
knowledge.query.completed
          │
          ▼
tutor.response.started
          │
          ▼
tutor.response.completed
          │
          ▼
speech.synthesis.started
          │
          ▼
avatar.behavior.started
          │
          ▼
speech.playback.completed
```

Every event can be reconstructed as part of the same interaction.

---

# 73. Causation Example

```text
student.answer.submitted
          │
          ▼
assessment.answer.evaluated
          │
          ▼
competency.evidence.added
          │
          ▼
competency.updated
          │
          ▼
pedagogy.strategy.changed
```

Each event references its direct causal predecessor.

---

# 74. Distributed Tracing

The trace identifier SHOULD be compatible with distributed tracing systems.

Each subsystem may create spans such as:

```text
session
 ├── speech-to-text
 ├── retrieval
 ├── tutor inference
 ├── TTS
 └── avatar rendering
```

This allows latency analysis.

---

# 75. Latency Measurements

Latency SHOULD be recorded at important boundaries.

Examples:

```text
microphone → transcript
transcript → tutor first token
tutor request → completed response
text → first audio
audio → avatar lip movement
student answer → feedback
```

---

# 76. Real-Time Event Classes

Events may be classified by timing requirements.

```text
hard_realtime
soft_realtime
interactive
background
batch
```

Most Nexa behavior is `interactive` or `soft_realtime`.

The system does not initially require hard real-time guarantees.

---

# 77. High-Frequency Streams

High-frequency streams SHOULD be distinguished from normal domain events.

Examples:

```text
audio frames
visemes
animation parameters
partial speech tokens
```

These MAY use specialized channels while still emitting summarized lifecycle events through the primary event bus.

---

# 78. Media Data

Large binary data SHALL NOT be embedded directly in normal events.

Examples:

```text
audio
video
screenshots
large terminal outputs
model files
```

Events should contain references.

Example:

```json
{
  "audio_ref": "artifact://speech/8392/audio"
}
```

---

# 79. Event Schema Registry

Every stable event SHOULD have a registered schema.

Registry responsibilities:

```text
schema ID
event type
version
validation rules
documentation
compatibility rules
```

---

# 80. Schema Versioning

Event payload schemas SHALL be versioned independently where required.

Example:

```text
student.answer.submitted.v1
student.answer.submitted.v2
```

The preferred public event name may remain stable while schema metadata identifies payload version.

---

# 81. Compatibility

Minor schema additions SHOULD remain backward compatible.

Breaking changes require a new major schema version.

Consumers SHOULD ignore unknown optional fields where safe.

---

# 82. Rust Event Model

The core event type should resemble:

```rust
pub struct Event<T> {
    pub event_version: EventVersion,
    pub event_id: EventId,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub sequence: Option<u64>,
    pub source: EndpointId,
    pub subject: Option<SubjectId>,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<EventId>,
    pub trace_id: Option<TraceId>,
    pub payload: T,
    pub metadata: EventMetadata,
}
```

---

# 83. Typed Events

Critical events SHOULD be strongly typed.

Example:

```rust
pub struct StudentAnswerSubmitted {
    pub question_id: QuestionId,
    pub answer: StudentAnswer,
    pub self_confidence: Option<f32>,
}
```

Avoid passing arbitrary untyped JSON throughout the core application.

---

# 84. Event Type Enumeration

Where practical:

```rust
pub enum EventType {
    SessionStarted,
    SessionEnded,
    StudentAnswerSubmitted,
    TutorResponseCompleted,
    LessonCompleted,
    CompetencyUpdated,
    LabCommandExecuted,
    SpeechSynthesisCompleted,
    AvatarStateChanged,
}
```

Extension domains MAY use string-based identifiers.

---

# 85. Event Bus Trait

Conceptually:

```rust
#[async_trait]
pub trait EventBus {
    async fn publish<E>(&self, event: Event<E>) -> Result<()>
    where
        E: DomainEvent;

    async fn subscribe(
        &self,
        subscription: Subscription
    ) -> Result<EventStream>;
}
```

---

# 86. Local Event Bus

The MVP SHOULD begin with an in-process event bus.

Potential implementation concepts:

```text
tokio broadcast
tokio mpsc
typed channels
custom dispatcher
```

The public architecture SHALL not depend on the chosen mechanism.

---

# 87. Distributed Event Bus

Future deployments MAY use:

```text
NATS
RabbitMQ
Kafka
Redis Streams
custom QUIC messaging
```

The domain event model SHOULD remain unchanged.

---

# 88. Nexa Event Crate

Recommended repository layout:

```text
crates/
└── nexa-events/
    ├── src/
    │   ├── lib.rs
    │   ├── envelope.rs
    │   ├── command.rs
    │   ├── event.rs
    │   ├── bus.rs
    │   ├── subscription.rs
    │   ├── correlation.rs
    │   ├── tracing.rs
    │   ├── errors.rs
    │   └── domains/
    │       ├── session.rs
    │       ├── student.rs
    │       ├── tutor.rs
    │       ├── pedagogy.rs
    │       ├── lesson.rs
    │       ├── competency.rs
    │       ├── lab.rs
    │       ├── speech.rs
    │       └── avatar.rs
    └── tests/
```

---

# 89. NBP Integration

NBP behavior messages SHALL integrate cleanly with the event model.

Example:

```text
tutor.response.completed
          ↓
behavior plan
          ↓
NBP behavior.command
          ↓
avatar.behavior.started
          ↓
avatar.behavior.completed
```

NBP remains responsible for avatar behavior semantics.

NEXA-EVT remains responsible for platform event semantics.

---

# 90. Event-to-NBP Adapter

A dedicated adapter MAY convert platform state into avatar behavior.

```text
Nexa Events
     ↓
Behavior Planner
     ↓
NBP Builder
     ↓
NBP Runtime
```

This keeps the event bus independent of rendering details.

---

# 91. Privacy Boundary

Events SHALL minimize unnecessary learner information.

For example, an avatar event generally does not require:

```text
student full name
assessment history
email address
profile metadata
```

Identifiers SHOULD be opaque where possible.

---

# 92. Secret Handling

Events SHALL NOT contain:

```text
API keys
passwords
access tokens
private encryption keys
raw authentication credentials
```

Sensitive system values should be represented using secure references when absolutely necessary.

---

# 93. Error Events

Errors SHOULD use structured codes.

Example:

```json
{
  "event_type": "speech.synthesis.failed",
  "payload": {
    "error_code": "TTS_PROVIDER_TIMEOUT",
    "severity": "error",
    "recoverable": true,
    "operation_id": "tts-882",
    "message": "Speech synthesis timed out."
  }
}
```

---

# 94. Error Classification

Recommended classes:

```text
validation
transport
timeout
dependency
configuration
runtime
security
resource
model
content
unknown
```

---

# 95. Recovery Events

Where appropriate, recovery SHOULD be observable.

Example:

```text
speech.synthesis.failed
speech.provider.changed
speech.synthesis.started
speech.synthesis.completed
```

---

# 96. Dead-Letter Handling

Events that repeatedly fail processing MAY enter a dead-letter store.

Dead-letter records SHOULD include:

```text
original event
subscriber
failure reason
attempt count
timestamps
```

---

# 97. Observability

The event infrastructure SHOULD expose:

```text
events per second
queue depth
consumer lag
publish latency
processing latency
failure count
retry count
dropped event count
```

---

# 98. Event Logging

Development logging MAY present:

```text
18:42:01.112 student.answer.submitted
18:42:01.118 assessment.answer.evaluated
18:42:01.122 competency.updated
18:42:01.130 tutor.response.started
```

Production logging SHOULD avoid leaking unnecessary user content.

---

# 99. Event Filtering

Diagnostic tools SHOULD support filtering by:

```text
session
event type
domain
source
subject
time
correlation
trace
severity
```

---

# 100. Session Timeline

The developer tooling SHOULD eventually display an event timeline:

```text
00.000  session.started
00.412  lesson.started
02.914  student.question.asked
02.920  tutor.response.started
03.483  knowledge.query.completed
04.172  tutor.response.completed
04.221  speech.synthesis.started
04.470  speech.playback.started
04.475  avatar.behavior.started
08.217  speech.playback.completed
08.291  avatar.behavior.completed
```

This will be extremely valuable for Nexa development.

---

# 101. Deterministic Testing

Tests SHOULD be capable of injecting events into a deterministic bus.

Example:

```text
Given:
student.answer.submitted

Expect:
assessment.answer.evaluated
competency.updated
tutor.response.started
```

---

# 102. Event Contract Testing

Each subsystem SHOULD include contract tests verifying:

```text
events accepted
events emitted
payload schema
correlation propagation
error behavior
idempotency
```

---

# 103. Behavior Testing

Example:

```text
Given:
student.answer.incorrect

And:
hint_level = 1

Expect:
pedagogy.hint.selected

Then:
tutor.response.completed

Then:
avatar behavior = encouraging
```

---

# 104. Replay Testing

Recorded event streams SHOULD become regression test fixtures.

This allows the same training scenario to be replayed across future versions.

---

# 105. Event Fixture Format

A simple test fixture may use JSON Lines:

```text
event-001.jsonl
event-002.jsonl
```

or:

```text
session-tcp-handshake.jsonl
```

Each line contains one event envelope.

---

# 106. MVP Event Set

The initial Nexa MVP only requires a subset.

Minimum events:

```text
system.ready

session.started
session.ended

student.text.submitted

tutor.response.started
tutor.response.completed
tutor.response.failed

speech.synthesis.started
speech.synthesis.completed
speech.playback.started
speech.playback.completed

avatar.state.changed
avatar.behavior.started
avatar.behavior.completed

system.error
```

---

# 107. MVP Runtime Sequence

```text
session.started
       ↓
student.text.submitted
       ↓
tutor.response.started
       ↓
tutor.response.completed
       ↓
speech.synthesis.started
       ↓
speech.synthesis.completed
       ↓
avatar.behavior.started
       ↓
speech.playback.started
       ↓
speech.playback.completed
       ↓
avatar.behavior.completed
```

This constitutes the first event-driven vertical slice.

---

# 108. Phase 2 Events

Phase 2 introduces:

```text
student.speech.*
speech.transcription.*
lesson.*
pedagogy.*
assessment.*
competency.*
knowledge.*
```

---

# 109. Phase 3 Events

Phase 3 introduces:

```text
lab.*
tool.*
security.*
advanced telemetry
multi-agent events
distributed runtime events
```

---

# 110. Event Model Invariants

The following rules are architectural invariants:

1. Events SHALL be immutable.
2. Event names SHALL be semantic.
3. Commands SHALL remain distinct from events.
4. A command SHALL NOT be treated as evidence of success.
5. Important workflows SHALL propagate correlation IDs.
6. Distributed workflows SHOULD propagate trace IDs.
7. Consumers SHOULD tolerate duplicate delivery.
8. Important side effects SHOULD be idempotent.
9. High-frequency media streams MAY use optimized side channels.
10. Large binary payloads SHALL use references.
11. Event schemas SHALL be versioned.
12. The event bus SHALL not contain business logic.
13. Domain systems SHOULD use strongly typed events internally.
14. Learner-private data SHALL be minimized.
15. Secrets SHALL NOT be placed on the event bus.
16. Important learning events SHOULD be persistable.
17. Event replay SHOULD be supported.
18. Runtime behavior SHOULD be observable through events.
19. NBP SHALL remain distinct from the general event model.
20. The architecture SHALL allow an in-process bus to evolve into a distributed bus without changing domain semantics.

---

# 111. Recommended First Implementation

The first implementation SHOULD consist of:

```text
nexa-events
    │
    ├── Event<T>
    ├── Command<T>
    ├── EventId
    ├── CorrelationId
    ├── TraceId
    ├── EventType
    ├── Subscription
    └── InProcessEventBus
```

followed by typed events for:

```text
Session
Student
Tutor
Speech
Avatar
```

---

# 112. First Integration Test

The first integration test should validate:

```text
StudentTextSubmitted
        ↓
Tutor subscriber receives event
        ↓
TutorResponseCompleted
        ↓
Speech subscriber receives event
        ↓
SpeechSynthesisCompleted
        ↓
Behavior planner receives event
        ↓
NBP message emitted
        ↓
AvatarBehaviorCompleted
```

All events SHALL maintain:

```text
session_id
correlation_id
trace_id
```

throughout the interaction.

---

# 113. Relationship to Existing Specifications

This specification depends conceptually on:

**NEXA-CBS-001 — Nexa Character & Behavior Specification v1.0**

and integrates with:

**NEXA-NBP-001 — Nexa Behavior Protocol v1.0**

The relationship is:

```text
Character Specification
        │
        ▼
Behavior semantics
        │
        ▼
NBP
        │
        ├─────────────┐
        │             │
        ▼             ▼
 Avatar Runtime    Event Model
                       │
                       ▼
                  Entire Platform
```

---

# 114. Next Specification

The next architecture specification should be:

# NEXA-DOM-001 — Nexa Core Domain Model & Type System v1.0

That document should formally define the shared concepts every subsystem uses, including:

```text
Student
Session
Course
Lesson
LessonStep
LearningObjective
Concept
Competency
Evidence
Question
Answer
Assessment
Attempt
Hint
Misconception
TutorResponse
BehaviorIntent
Tool
ToolExecution
Lab
Artifact
KnowledgeSource
Memory
Message
Identifiers
timestamps
state enums
result types
error types
```

This is the point where the architecture begins transitioning from conceptual design into **strongly typed Rust implementation contracts**.
