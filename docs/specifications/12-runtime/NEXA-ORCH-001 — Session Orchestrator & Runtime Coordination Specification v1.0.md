# NEXA-ORCH-001 — Session Orchestrator & Runtime Coordination Specification v1.0

**Specification ID:** NEXA-ORCH-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Purpose:** Define the runtime orchestration layer that coordinates Nexa’s tutoring, pedagogy, knowledge, speech, avatar, tools, lessons, assessments, memory, and event-driven workflows.

---

# 1. Purpose

The **Session Orchestrator** is the runtime coordination authority for an active Nexa tutoring session.

It does not replace the Tutor Engine, Pedagogy Engine, Speech Engine, Behavior Engine, or Lesson Engine.

Its purpose is to coordinate them.

The orchestrator SHALL:

* receive student input;
* maintain session workflow state;
* construct tutor requests;
* gather relevant context;
* invoke pedagogy;
* initiate knowledge retrieval;
* invoke tutor reasoning;
* coordinate tools and labs;
* translate tutor output into executable actions;
* coordinate speech and avatar behavior;
* manage interruption and cancellation;
* enforce timing and failure-recovery policies;
* publish lifecycle events;
* persist relevant session history.

The orchestrator is therefore the component that turns separate Nexa services into one coherent tutor.

---

# 2. Architectural Role

```text
                           STUDENT
                              │
                              ▼
                        INPUT SYSTEM
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                    SESSION ORCHESTRATOR                       │
│                                                               │
│ session state │ workflow │ routing │ timing │ coordination    │
└───────────┬───────────────┬──────────────┬────────────────────┘
            │               │              │
            ▼               ▼              ▼
       Pedagogy          Knowledge       Memory
            │               │              │
            └───────┬───────┴──────┬───────┘
                    ▼              ▼
                 Tutor Engine    Tools/Labs
                    │
                    ▼
             Structured Response
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       Speech     Behavior   Canvas
          │         │
          ▼         ▼
        Audio      NBP
                    │
                    ▼
                  Nexa
```

---

# 3. Core Principle

The orchestrator SHALL coordinate work but SHOULD NOT contain domain-specific reasoning that belongs elsewhere.

Examples:

The orchestrator MAY decide:

```text
"Invoke the pedagogy engine."
```

It SHOULD NOT decide:

```text
"The learner needs Socratic instruction because TCP mastery is 0.42."
```

That decision belongs to the Pedagogy Engine.

Likewise, the orchestrator MAY invoke retrieval but SHOULD NOT implement retrieval ranking algorithms.

---

# 4. Primary Responsibilities

The orchestrator SHALL own:

1. session lifecycle coordination;
2. workflow lifecycle coordination;
3. cross-service request routing;
4. execution ordering;
5. timeout policies;
6. cancellation;
7. correlation and tracing;
8. response execution;
9. concurrency boundaries;
10. runtime recovery;
11. external capability awareness;
12. synchronization of speech, avatar, and canvas;
13. session-level observability.

---

# 5. Responsibilities Explicitly Outside the Orchestrator

The orchestrator SHALL NOT own:

* LLM reasoning;
* competency scoring algorithms;
* lesson authoring;
* animation implementation;
* speech synthesis algorithms;
* speech recognition algorithms;
* RAG ranking;
* vector search implementation;
* lab execution internals;
* persistence engine implementation;
* UI rendering.

---

# 6. Session Lifecycle

The orchestrator SHALL implement the canonical session lifecycle.

```text
CREATED
   ↓
INITIALIZING
   ↓
READY
   ↓
ACTIVE
   ├────► PAUSED
   │        │
   │        └────► ACTIVE
   │
   ├────► DEGRADED
   │        │
   │        └────► ACTIVE
   │
   ▼
ENDING
   ↓
COMPLETED
```

Failure path:

```text
ANY ACTIVE STATE
       ↓
     FAILED
```

---

# 7. Session Runtime State

```rust
pub enum RuntimeSessionState {
    Created,
    Initializing,
    Ready,
    Active,
    Paused,
    Degraded,
    Ending,
    Completed,
    Failed,
}
```

This runtime state MAY supplement the core domain `SessionState`.

---

# 8. Session Initialization

Initialization SHOULD perform:

```text
Load student
      ↓
Load requested course / lesson
      ↓
Load learner progress
      ↓
Load competency projection
      ↓
Load active misconceptions
      ↓
Resolve available tools
      ↓
Resolve speech capabilities
      ↓
Resolve avatar capabilities
      ↓
Initialize session memory
      ↓
Publish session.ready
```

A session SHALL NOT become `ACTIVE` until mandatory dependencies are available.

---

# 9. Capability Classification

Dependencies SHOULD be classified as:

```text
required
preferred
optional
```

Example:

```yaml
capabilities:
  tutor_engine: required
  text_input: required
  speech_tts: preferred
  avatar_runtime: preferred
  lab_runtime: optional
```

This permits graceful degraded operation.

---

# 10. Degraded Session Example

If speech synthesis fails but text output remains available:

```text
ACTIVE
  ↓
speech unavailable
  ↓
DEGRADED
  ↓
text response continues
```

Nexa SHOULD remain usable where meaningful fallback exists.

---

# 11. Interaction Workflow

Each meaningful learner interaction SHALL create a workflow.

Example:

```text
Student asks question
        ↓
InteractionWorkflow created
        ↓
Input normalization
        ↓
Context assembly
        ↓
Pedagogy decision
        ↓
Knowledge retrieval
        ↓
Tutor request
        ↓
Tutor response
        ↓
Response planning
        ↓
Speech + avatar + canvas execution
        ↓
Workflow completed
```

---

# 12. Interaction Workflow Type

```rust
pub struct InteractionWorkflow {
    pub id: WorkflowId,
    pub session_id: SessionId,
    pub correlation_id: CorrelationId,
    pub trace_id: TraceId,
    pub state: WorkflowState,
    pub input: StudentInput,
    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}
```

---

# 13. Workflow States

```rust
pub enum WorkflowState {
    Created,
    NormalizingInput,
    PreparingContext,
    SelectingPedagogy,
    RetrievingKnowledge,
    GeneratingTutorResponse,
    ExecutingTools,
    PlanningResponse,
    Speaking,
    WaitingForStudent,
    Completed,
    Cancelled,
    Failed,
}
```

---

# 14. Workflow Correlation

One user interaction SHOULD share a single correlation ID across:

```text
student input
pedagogy
retrieval
tutor inference
tool calls
speech synthesis
avatar behavior
canvas activity
```

This enables complete reconstruction of the interaction.

---

# 15. Input Sources

The orchestrator SHALL support input from:

```text
text
speech
answer submission
tool action
lab action
UI control
system event
```

All forms SHALL normalize to `StudentInput`.

---

# 16. Input Normalization

Examples:

```text
keyboard text
      ↓
StudentInput::Text
```

```text
speech audio
      ↓
STT
      ↓
StudentInput::SpeechTranscript
```

```text
quiz submission
      ↓
StudentInput::Answer
```

The Tutor Engine should not require knowledge of UI-specific controls.

---

# 17. Input Concurrency

The orchestrator SHALL define policy for simultaneous inputs.

Potential conflicts include:

```text
student speaking while Nexa speaks
student submits answer while tool running
pause request while tutor generating
new question before previous response completes
```

These SHALL not be left to individual components to resolve independently.

---

# 18. Input Priority

Recommended priority order:

```text
emergency / stop
pause
student interruption
assessment submission
normal student input
background actions
```

---

# 19. Student Interruption

The orchestrator SHALL support barge-in behavior.

```text
Nexa speaking
     ↓
student.speech.started
     ↓
interrupt policy evaluated
     ↓
speech.cancel
     ↓
behavior.cancel
     ↓
avatar → listening
     ↓
STT continues
```

---

# 20. Interruption Policies

```rust
pub enum InterruptionPolicy {
    Disabled,
    Immediate,
    WordBoundary,
    PhraseBoundary,
    SentenceBoundary,
    Adaptive,
}
```

`Adaptive` SHOULD become the normal long-term mode.

---

# 21. Adaptive Interruption

Adaptive interruption MAY consider:

```text
response importance
current sentence
warning state
assessment mode
student speech confidence
background noise
behavior state
```

Example:

A safety warning MAY finish the current sentence before interruption.

A general explanation MAY stop at a phrase boundary.

---

# 22. Context Assembly

The orchestrator SHALL request a purpose-built `TutorContext`.

Context assembly SHOULD be performed by a dedicated context service.

```text
Student
   │
   ├── competencies
   ├── misconceptions
   ├── lesson state
   ├── recent conversation
   ├── retrieved knowledge
   ├── available tools
   └── pedagogy decision
          │
          ▼
      TutorContext
```

---

# 23. Context Minimization

The orchestrator SHALL NOT blindly send all historical state to the Tutor Engine.

Context SHOULD be:

```text
relevant
bounded
structured
source-aware
purpose-specific
```

This improves cost, latency, correctness, and privacy.

---

# 24. Context Builder Contract

```rust
#[async_trait]
pub trait TutorContextBuilder {
    async fn build(
        &self,
        request: &ContextRequest,
    ) -> OrchestratorResult<TutorContext>;
}
```

---

# 25. Context Request

```rust
pub struct ContextRequest {
    pub student_id: StudentId,
    pub session_id: SessionId,
    pub input: StudentInput,
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
    pub workflow_id: WorkflowId,
}
```

---

# 26. Pedagogy Invocation

The orchestrator SHOULD invoke pedagogy before final tutor generation when instruction is involved.

```text
Student input
     ↓
Current learning state
     ↓
Pedagogy Engine
     ↓
PedagogyDecision
     ↓
Tutor Context
```

The tutor then generates within that pedagogical frame.

---

# 27. Pedagogy Bypass

Not all interactions require pedagogy.

Examples:

```text
"Repeat that."
"Pause."
"What lesson are we on?"
"Turn your voice down."
```

The orchestrator MAY route such control interactions directly.

---

# 28. Knowledge Retrieval Decision

Retrieval SHOULD be initiated when:

* the tutor requires external grounding;
* lesson material is relevant;
* current documentation is required;
* a source-specific answer is requested;
* policy requires grounding.

Retrieval MAY be skipped for:

* greetings;
* obvious session control;
* purely reflective prompts;
* known lesson metadata.

---

# 29. Retrieval Workflow

```text
Tutor request candidate
        ↓
Retrieval required?
   ┌────┴────┐
  yes       no
   │         │
   ▼         ▼
query       continue
   │
   ▼
rank results
   │
   ▼
context assembly
```

---

# 30. Retrieval Failure

Failure policies SHOULD include:

```text
retry
fallback source
continue without retrieval
ask tutor to acknowledge uncertainty
fail interaction
```

Policy depends on the importance of grounding.

---

# 31. Tutor Invocation

The orchestrator SHALL invoke the Tutor Engine with a typed request.

```rust
#[async_trait]
pub trait TutorEngine {
    async fn respond(
        &self,
        request: TutorRequest,
    ) -> TutorResult<TutorResponse>;
}
```

---

# 32. Streaming Tutor Responses

The architecture SHOULD support streaming.

```text
Tutor Engine
    │
    ├── response.started
    ├── response.partial
    ├── response.partial
    ├── response.partial
    └── response.completed
```

This can reduce perceived latency.

---

# 33. Streaming Boundary

The orchestrator SHALL distinguish:

```text
display streaming
speech streaming
final structured response
```

Partial model tokens SHOULD NOT automatically become unvalidated NBP instructions.

---

# 34. Structured Response Requirement

Before executing behavior or tools, the orchestrator SHALL obtain a validated structured response.

```text
LLM output
   ↓
parse
   ↓
schema validation
   ↓
domain validation
   ↓
TutorResponse
```

Malformed output SHALL not directly drive runtime systems.

---

# 35. Tool Invocation

The TutorResponse MAY contain tool requests.

The orchestrator SHALL route them through a tool execution service.

```text
TutorResponse
      ↓
ToolRequest
      ↓
Authorization
      ↓
Tool Runtime
      ↓
ToolResult
      ↓
Tutor continuation
```

---

# 36. Tool Authorization

Tool execution SHALL occur only after policy evaluation.

```text
ToolRequest
     ↓
Policy Engine
     ↓
allow / deny / confirm
```

The Tutor Engine SHALL NOT bypass this boundary.

---

# 37. Tool Continuation Loop

Some interactions require multiple reasoning passes.

```text
Tutor
  ↓
tool request
  ↓
tool executes
  ↓
result returned
  ↓
Tutor continues
  ↓
final response
```

The orchestrator SHALL support this loop.

---

# 38. Tool Loop Limit

The orchestrator SHOULD enforce a configurable maximum number of tool iterations.

Example:

```text
max_tool_iterations = 8
```

This prevents runaway execution loops.

---

# 39. Tool Timeout

Each tool call SHALL have:

```text
timeout
cancellation
status
trace context
```

A timed-out tool SHALL generate a structured failure result.

---

# 40. Lab Integration

Lab sessions MAY span multiple tutor interactions.

The orchestrator SHALL preserve lab context independently from one response.

```text
Session
  │
  └── active_lab
        ├── environment
        ├── objectives
        ├── observations
        └── policy
```

---

# 41. Lab Observation Routing

```text
Student runs command
        ↓
Lab Runtime
        ↓
lab.command.executed
        ↓
Orchestrator
        ↓
Lab Observer
        ↓
Tutor / Pedagogy
```

Nexa may react to real execution outcomes.

---

# 42. Response Planning

A `TutorResponse` is semantic.

The orchestrator SHOULD pass it through a Response Planner.

```text
TutorResponse
     ↓
Response Planner
     ├── speech plan
     ├── behavior plan
     ├── canvas plan
     └── follow-up plan
```

---

# 43. Response Plan Type

```rust
pub struct ResponsePlan {
    pub response_id: TutorResponseId,
    pub speech: Option<SpeechPlan>,
    pub behavior: BehaviorPlan,
    pub canvas_actions: Vec<CanvasAction>,
    pub tools: Vec<ToolExecutionPlan>,
    pub follow_up: Option<FollowUpPlan>,
}
```

---

# 44. Speech Plan

```rust
pub struct SpeechPlan {
    pub text: String,
    pub style: SpeechStyle,
    pub streaming: bool,
    pub interruption_policy: InterruptionPolicy,
}
```

---

# 45. Behavior Plan

```rust
pub struct BehaviorPlan {
    pub intent: BehaviorIntent,
    pub nbp_command: NbpBehaviorCommand,
}
```

The NBP command SHOULD be built by a dedicated adapter.

---

# 46. Response Execution

The orchestrator SHALL coordinate parallelizable operations.

Example:

```text
               ResponsePlan
                    │
         ┌──────────┼──────────┐
         ▼          ▼          ▼
      Canvas       TTS       Behavior
         │          │          │
         └──────────┼──────────┘
                    ▼
              synchronized
                playback
```

---

# 47. Parallel Execution

The following MAY start concurrently:

```text
canvas rendering
speech synthesis
avatar preparation
nonverbal gesture preparation
```

The orchestrator SHALL avoid unnecessary sequential latency.

---

# 48. Synchronization Barrier

Some actions require synchronization.

Example:

```text
gaze at diagram
     ↓
200 ms
     ↓
point gesture
     ↓
speech references object
```

The orchestrator MAY delegate fine animation timing to the Behavior Engine/NBP runtime.

---

# 49. Speech Readiness

The orchestrator SHOULD support:

```text
prebuffered speech
fully synthesized speech
streamed speech
```

For interactive conversation, streamed speech is preferred where reliable.

---

# 50. First-Audio Latency

An important runtime metric is:

```text
student finishes input
        ↓
first Nexa audio begins
```

This SHOULD be measured continuously.

---

# 51. Speech Failure Fallback

If TTS fails:

```text
speech synthesis failed
       ↓
publish failure event
       ↓
display text response
       ↓
avatar uses silent explaining behavior
```

The interaction SHOULD continue where possible.

---

# 52. Avatar Failure Fallback

If avatar rendering fails:

```text
avatar unavailable
      ↓
text + speech continue
```

Avatar presence SHOULD enhance tutoring but SHOULD NOT become a single point of failure for basic instruction.

---

# 53. Canvas Failure Fallback

If a diagram cannot be rendered:

Nexa SHOULD adapt speech.

Instead of:

> "Look at the highlighted SYN packet."

the system should allow:

> "The first packet is the SYN sent from the client."

This requires response-plan awareness of capability failures.

---

# 54. Runtime Capability Registry

The orchestrator SHOULD maintain current capabilities.

```rust
pub struct RuntimeCapabilities {
    pub tutor: TutorCapabilities,
    pub speech: SpeechCapabilities,
    pub avatar: AvatarCapabilities,
    pub canvas: CanvasCapabilities,
    pub tools: Vec<ToolCapabilityDescriptor>,
    pub labs: Vec<LabCapabilityDescriptor>,
}
```

---

# 55. Capability Changes

Capabilities MAY change during a session.

Examples:

```text
microphone disconnected
TTS provider unavailable
lab runtime restarted
avatar renderer crashed
```

Capability changes SHALL publish events.

---

# 56. Orchestrator Command Queue

Commands SHOULD be prioritized.

Conceptually:

```text
┌────────────────────┐
│ Immediate          │ stop / emergency
├────────────────────┤
│ Interactive High   │ interruption / pause
├────────────────────┤
│ Interactive Normal │ questions / answers
├────────────────────┤
│ Background         │ memory updates
└────────────────────┘
```

---

# 57. Concurrency Model

The orchestrator SHOULD use structured concurrency.

Every workflow SHOULD own its spawned operations.

```text
InteractionWorkflow
      │
      ├── retrieval task
      ├── tutor task
      ├── speech task
      └── behavior task
```

When the workflow is cancelled, owned tasks SHOULD be cancelled appropriately.

---

# 58. No Detached Runtime Work

Core interaction tasks SHOULD NOT become untracked background tasks.

Untracked work makes:

* cancellation;
* recovery;
* testing;
* shutdown;
* observability

unreliable.

---

# 59. Cancellation Token

Each workflow SHOULD expose a cancellation token.

Conceptually:

```rust
pub struct WorkflowContext {
    pub workflow_id: WorkflowId,
    pub cancellation: CancellationToken,
    pub correlation_id: CorrelationId,
    pub trace_id: TraceId,
}
```

---

# 60. Cancellation Propagation

```text
student.stop
    ↓
workflow cancellation
    ↓
Tutor generation cancelled
    ↓
TTS cancelled
    ↓
NBP behavior cancelled
    ↓
Tool calls cancelled where safe
```

---

# 61. Non-Cancellable Actions

Some operations MAY not be safely cancellable.

Examples:

```text
database commit
external destructive command already executing
hardware operation
```

Such operations SHALL report their cancellation semantics explicitly.

---

# 62. Timeout Hierarchy

Timeouts SHOULD exist at multiple levels.

```text
workflow timeout
    │
    ├── retrieval timeout
    ├── tutor timeout
    ├── tool timeout
    ├── TTS timeout
    └── behavior timeout
```

A child timeout does not necessarily require the entire workflow to fail.

---

# 63. Timeout Configuration

```rust
pub struct OrchestratorTimeouts {
    pub retrieval: Duration,
    pub tutor_response: Duration,
    pub tool_execution: Duration,
    pub speech_synthesis: Duration,
    pub avatar_ack: Duration,
    pub interaction: Duration,
}
```

---

# 64. Retry Policy

Retries SHALL be explicit and bounded.

```rust
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff: BackoffStrategy,
}
```

---

# 65. Retryable Failures

Typical retry candidates:

```text
temporary network failure
provider timeout
transient database failure
temporary service unavailable
```

Non-retry candidates:

```text
invalid request
permission denied
schema failure
unsupported action
```

---

# 66. Circuit Breakers

External services SHOULD eventually support circuit-breaker behavior.

```text
TTS fails repeatedly
      ↓
circuit opens
      ↓
requests stop temporarily
      ↓
fallback used
      ↓
health probe
      ↓
circuit closes
```

---

# 67. Failure Classification

Failures SHOULD be classified as:

```text
interaction-local
component-local
session-degrading
session-fatal
system-fatal
```

---

# 68. Interaction-Local Failure

Example:

One diagram cannot render.

Response:

```text
fallback
continue interaction
```

---

# 69. Component-Local Failure

Example:

TTS unavailable.

Response:

```text
disable speech
continue using text
```

---

# 70. Session-Fatal Failure

Example:

Student session state cannot be recovered.

Response:

```text
terminate session cleanly
persist failure information
```

---

# 71. Recovery Manager

The orchestrator SHOULD use a dedicated recovery policy service rather than large nested error branches.

```rust
pub trait RecoveryPolicy {
    fn decide(
        &self,
        failure: &RuntimeFailure,
        context: &WorkflowContext,
    ) -> RecoveryAction;
}
```

---

# 72. Recovery Actions

```rust
pub enum RecoveryAction {
    Retry,
    RetryWithFallback,
    ContinueDegraded,
    SkipOptionalStep,
    CancelWorkflow,
    EndSession,
    Escalate,
}
```

---

# 73. Orchestrator Event Inputs

The orchestrator SHOULD subscribe to:

```text
student.*
session.*
speech.*
avatar.*
tool.*
lab.*
lesson.*
assessment.*
system.*
security.*
```

Not every event requires action.

---

# 74. Orchestrator Event Outputs

Typical outputs:

```text
orchestrator.workflow.started
orchestrator.workflow.completed
orchestrator.workflow.failed
session.state.changed
tutor.request.received
speech.synthesis.requested
avatar.behavior.requested
tool.execution.requested
```

---

# 75. Workflow Event Example

```json
{
  "event_type": "orchestrator.workflow.started",
  "payload": {
    "workflow_id": "wf-0193f...",
    "workflow_type": "student_interaction",
    "input_type": "text"
  }
}
```

---

# 76. Event-Driven Versus Direct Calls

Within the same process, direct typed calls MAY be used for efficiency.

Example:

```text
orchestrator → context_builder.build()
```

The meaningful lifecycle SHOULD still emit events.

This provides:

```text
performance + observability
```

without forcing every internal function through a message broker.

---

# 77. Command Routing

The orchestrator SHOULD route commands through typed service interfaces.

```rust
pub trait SpeechService { ... }
pub trait BehaviorService { ... }
pub trait ToolService { ... }
pub trait RetrievalService { ... }
```

The orchestrator SHALL depend on interfaces, not concrete providers.

---

# 78. Session Actor Model

A strong implementation approach is one logical actor per active session.

```text
Session A ──► Orchestrator Actor A
Session B ──► Orchestrator Actor B
Session C ──► Orchestrator Actor C
```

This simplifies:

* mutable session state;
* ordering;
* cancellation;
* concurrent users.

---

# 79. Session Isolation

One session's mutable workflow state SHALL NOT leak into another session.

Shared services MAY include:

```text
LLM runtime
knowledge database
TTS engine
avatar asset cache
```

but session state remains isolated.

---

# 80. Session Mailbox

Each session actor MAY receive:

```rust
pub enum SessionMessage {
    StudentInput(StudentInput),
    RuntimeEvent(EventEnvelope),
    Control(SessionControl),
    Timeout(TimeoutKind),
    Shutdown,
}
```

---

# 81. Session Control

```rust
pub enum SessionControl {
    Pause,
    Resume,
    Stop,
    ResetInteraction,
    ChangeMode(SessionMode),
}
```

---

# 82. Single Active Conversational Workflow

For the MVP, each session SHOULD allow only one primary conversational workflow at a time.

This prevents:

```text
two tutor responses speaking simultaneously
conflicting avatar behaviors
out-of-order pedagogy decisions
```

Background operations MAY remain concurrent.

---

# 83. Future Multi-Workflow Support

Later Nexa may support:

```text
conversation workflow
lab monitoring workflow
background retrieval
progress projection
content prefetching
```

These require explicit coordination priorities.

---

# 84. Background Prefetching

The orchestrator MAY prefetch likely future resources.

Example:

```text
current TCP lesson
      ↓
next step likely congestion control
      ↓
retrieve diagrams/content in background
```

Prefetching SHALL NOT alter learning state until content is actually used.

---

# 85. Follow-Up Planning

A TutorResponse MAY include a next pedagogical action.

Example:

```text
explain concept
     ↓
ask verification question
```

The orchestrator SHOULD schedule the follow-up after speech completion.

---

# 86. Follow-Up Type

```rust
pub enum FollowUpPlan {
    AskQuestion(QuestionId),
    WaitForStudent,
    ContinueLesson,
    StartAssessment(AssessmentId),
    StartLab(LabId),
    None,
}
```

---

# 87. Wait State

After Nexa asks a question:

```text
QUESTIONING
     ↓
speech.completed
     ↓
WAITING
```

The orchestrator SHALL explicitly represent this state.

---

# 88. Silence Timeout

While waiting for an answer, a configurable silence timeout MAY occur.

Possible response:

```text
wait
  ↓
timeout
  ↓
Nexa gently prompts student
```

This should be pedagogically controlled rather than hardcoded.

---

# 89. Memory Updates

Memory writes SHOULD generally occur after meaningful interaction milestones.

Examples:

```text
lesson completed
misconception identified
preference explicitly changed
session summary generated
```

The orchestrator MAY schedule these as lower-priority work.

---

# 90. Session Summary

At session end, the system MAY generate:

```text
topics covered
competencies affected
misconceptions observed
unfinished objectives
recommended next actions
```

This summary may feed future context.

---

# 91. Persistence Strategy

The orchestrator SHOULD persist:

```text
session lifecycle
student inputs
TutorResponses
assessment results
competency evidence
lab actions
important errors
```

Transient animation details MAY be optionally retained for diagnostics.

---

# 92. Transaction Boundaries

Learning-state changes SHOULD use explicit transactional boundaries.

Example:

```text
assessment result
      ↓
evidence creation
      ↓
competency update
      ↓
progress update
      ↓
commit
```

Partial updates SHOULD be avoided.

---

# 93. Exactly-Once Illusion

The orchestrator SHALL NOT assume exactly-once message delivery.

State changes SHOULD use:

```text
idempotency key
event ID
operation ID
```

to prevent duplicate side effects.

---

# 94. Idempotency Example

If:

```text
assessment.completed
```

is observed twice, the competency update SHALL not be applied twice.

---

# 95. State Reconciliation

After component failure or reconnect, the orchestrator SHOULD be able to reconcile state.

Example:

```text
avatar reconnects
      ↓
runtime.capabilities
      ↓
runtime.state
      ↓
orchestrator determines expected state
      ↓
synchronization command
```

---

# 96. Speech/Avatar Reconciliation

If the avatar reconnects during active speech, policy MAY choose:

```text
join current speech
wait until next response
restart behavior
```

The default SHOULD avoid jarring visual resets.

---

# 97. Health Monitoring

The orchestrator SHOULD observe health of critical dependencies.

```text
Tutor Engine       healthy
Event Bus          healthy
Persistence        healthy
Speech             degraded
Avatar             healthy
Knowledge          healthy
```

---

# 98. Health States

```rust
pub enum HealthState {
    Healthy,
    Degraded,
    Unavailable,
    Unknown,
}
```

---

# 99. Service Registry

The runtime SHOULD maintain a service registry.

```rust
pub struct ServiceRegistry {
    pub tutor: Arc<dyn TutorEngine>,
    pub pedagogy: Arc<dyn PedagogyEngine>,
    pub retrieval: Arc<dyn RetrievalService>,
    pub speech: Arc<dyn SpeechService>,
    pub behavior: Arc<dyn BehaviorService>,
    pub tools: Arc<dyn ToolService>,
}
```

---

# 100. Dependency Injection

Services SHOULD be supplied to the orchestrator rather than constructed internally.

This improves:

* testing;
* provider replacement;
* modularity;
* local/cloud switching.

---

# 101. Local-First Provider Selection

The registry MAY choose providers based on deployment configuration.

Example:

```text
Tutor:
  local model

Speech:
  local TTS

Knowledge:
  local vector store

Avatar:
  local desktop runtime
```

A cloud configuration can use the same interfaces.

---

# 102. Provider Failover

The architecture MAY support ordered providers.

Example:

```text
TTS
 ├── local_primary
 ├── local_fallback
 └── cloud_fallback
```

Provider changes SHOULD be observable through events.

---

# 103. Orchestrator State Machine

At the top level:

```text
                 ┌─────────────┐
                 │ INITIALIZE  │
                 └──────┬──────┘
                        ▼
                   ┌─────────┐
          ┌───────►│  READY  │◄────────┐
          │        └────┬────┘         │
          │             ▼              │
          │        ┌──────────┐        │
          │        │ PROCESS  │        │
          │        │  INPUT   │        │
          │        └────┬─────┘        │
          │             ▼              │
          │        ┌──────────┐        │
          │        │ EXECUTE  │        │
          │        │ RESPONSE │        │
          │        └────┬─────┘        │
          │             ▼              │
          └───────── WAITING ──────────┘
```

---

# 104. Orchestrator API

Conceptually:

```rust
#[async_trait]
pub trait SessionOrchestrator {
    async fn start_session(
        &self,
        request: StartSessionRequest,
    ) -> OrchestratorResult<SessionHandle>;

    async fn submit_input(
        &self,
        session_id: SessionId,
        input: StudentInput,
    ) -> OrchestratorResult<WorkflowId>;

    async fn control(
        &self,
        session_id: SessionId,
        command: SessionControl,
    ) -> OrchestratorResult<()>;

    async fn end_session(
        &self,
        session_id: SessionId,
    ) -> OrchestratorResult<()>;
}
```

---

# 105. Session Handle

```rust
pub struct SessionHandle {
    pub session_id: SessionId,
    pub state: RuntimeSessionState,
    pub capabilities: RuntimeCapabilities,
}
```

---

# 106. Start Session Request

```rust
pub struct StartSessionRequest {
    pub student_id: StudentId,
    pub course_id: Option<CourseId>,
    pub lesson_id: Option<LessonId>,
    pub mode: SessionMode,
    pub preferred_capabilities: CapabilityPreferences,
}
```

---

# 107. Observability Context

Each workflow SHOULD generate:

```text
workflow_id
session_id
correlation_id
trace_id
student interaction index
```

These SHOULD be included in structured logs.

---

# 108. Performance Metrics

The orchestrator SHOULD collect:

```text
input_to_first_token
input_to_first_audio
retrieval_latency
tutor_latency
TTS_latency
avatar_ack_latency
total_interaction_latency
tool_latency
workflow_failure_rate
interrupt_response_latency
```

---

# 109. User-Perceived Latency

The most important latency is not necessarily total completion time.

For voice interaction:

```text
student stops speaking
      ↓
Nexa visibly reacts
      ↓
Nexa begins speaking
```

Visual reaction can reduce perceived delay even while inference continues.

---

# 110. Thinking State Timing

The orchestrator SHOULD transition Nexa to `THINKING` when nontrivial processing begins.

The thinking animation SHOULD end when useful response output begins.

It SHALL NOT artificially extend system latency for theatrical effect.

---

# 111. Early Behavior

Before speech is ready:

```text
student finishes speaking
      ↓
Nexa → thinking
      ↓
small processing gesture
      ↓
speech becomes ready
      ↓
Nexa → explaining
```

This creates natural feedback.

---

# 112. Streaming Speech Pipeline

Future preferred flow:

```text
Tutor response tokens
        ↓
sentence chunker
        ↓
TTS streaming
        ↓
audio chunks
        ↓
viseme stream
        ↓
avatar
```

The orchestrator coordinates lifecycle and cancellation.

---

# 113. Sentence Chunking

Speech streaming SHOULD normally operate on meaningful language chunks rather than isolated tokens.

This improves:

* prosody;
* coherence;
* lip synchronization;
* interruption behavior.

---

# 114. Response Mutation Rule

Once a sentence begins speech playback, later model output SHOULD NOT silently rewrite that sentence.

Streaming requires a commitment boundary.

---

# 115. Commit Boundary

Conceptually:

```text
draft tokens
    ↓
sentence stabilized
    ↓
sentence committed
    ↓
TTS
```

Committed text becomes part of session history.

---

# 116. Assessment Coordination

In assessment mode, the orchestrator SHALL enforce assessment policy.

The Tutor Engine cannot simply decide to reveal answers.

```text
Tutor proposal
      ↓
Assessment Policy
      ↓
allowed?
```

---

# 117. Assessment Restrictions

Examples:

```text
hints disabled
tool access limited
knowledge retrieval restricted
feedback delayed
```

The orchestrator ensures these runtime constraints are honored.

---

# 118. Lesson Coordination

For guided lessons:

```text
Lesson Engine
      ↓
current step
      ↓
Orchestrator
      ↓
Tutor
      ↓
student interaction
      ↓
completion evidence
      ↓
Lesson Engine
```

The orchestrator does not decide authored lesson progression itself.

---

# 119. Freeform Mode

In freeform tutoring:

```text
student question
      ↓
context + pedagogy
      ↓
Tutor response
```

Course and lesson state MAY be absent.

---

# 120. Debugging Mode

Debugging workflows MAY include iterative tool loops.

```text
Observe error
     ↓
Tutor forms diagnostic action
     ↓
tool
     ↓
result
     ↓
Tutor evaluates
     ↓
next diagnostic action
```

The orchestrator SHALL preserve one correlation trace across the entire debugging sequence.

---

# 121. Security Policy Integration

Before executing privileged actions:

```text
request
   ↓
Policy Engine
   ↓
allow / deny / confirm
```

Security decisions SHALL be outside LLM authority.

---

# 122. Confirmation Workflow

When policy requires student confirmation:

```text
ToolRequest
     ↓
confirmation required
     ↓
Nexa explains consequence
     ↓
student confirms
     ↓
operation executes
```

Confirmation state SHALL be explicit.

---

# 123. Confirmation Expiry

Confirmations SHOULD expire after:

```text
timeout
session change
request mutation
context mutation
```

A prior confirmation should not authorize a materially different action.

---

# 124. Shutdown

Graceful shutdown SHOULD:

```text
stop accepting new sessions
      ↓
cancel or drain active workflows
      ↓
persist session state
      ↓
flush important events
      ↓
stop runtime services
```

---

# 125. Crash Recovery

After unexpected termination, the system SHOULD be capable of identifying:

```text
sessions active at crash
unfinished workflows
pending persistence operations
active labs
```

Recovery MAY either resume or cleanly close them according to policy.

---

# 126. Testability

The orchestrator SHALL be testable with mock services.

Mocks SHOULD exist for:

```text
Tutor Engine
Pedagogy Engine
Retrieval
Speech
Behavior
Tools
Persistence
Clock
```

---

# 127. Deterministic Workflow Test

Example:

```text
Given:
  StudentInput::Text("What is TCP?")

Mock tutor returns:
  fixed TutorResponse

Expect:
  tutor.response.completed
  speech.synthesis.requested
  avatar.behavior.requested
  workflow.completed
```

---

# 128. Cancellation Test

```text
Given:
  tutor response in progress

When:
  student sends Stop

Expect:
  tutor cancelled
  TTS cancelled
  behavior cancelled
  workflow cancelled
  session remains active
```

---

# 129. Failure Recovery Test

```text
Given:
  TTS unavailable

When:
  TutorResponse generated

Expect:
  text displayed
  avatar behavior still executes if available
  session enters degraded state
  workflow completes
```

---

# 130. Concurrency Test

```text
Given:
  Nexa is speaking

When:
  student begins speaking

Expect:
  active behavior cancellation policy invoked
  speech stops at configured boundary
  avatar enters listening state
  new interaction workflow begins
```

---

# 131. Recommended Crate

```text
crates/
└── nexa-orchestrator/
    ├── src/
    │   ├── lib.rs
    │   ├── orchestrator.rs
    │   ├── session_actor.rs
    │   ├── workflow.rs
    │   ├── context.rs
    │   ├── response_plan.rs
    │   ├── cancellation.rs
    │   ├── timeout.rs
    │   ├── retry.rs
    │   ├── recovery.rs
    │   ├── capability.rs
    │   ├── service_registry.rs
    │   ├── errors.rs
    │   └── adapters/
    │       ├── nbp.rs
    │       ├── events.rs
    │       └── tutor.rs
    └── tests/
        ├── session.rs
        ├── workflow.rs
        ├── interruption.rs
        ├── recovery.rs
        └── integration.rs
```

---

# 132. Dependency Direction

Recommended dependency relationship:

```text
             nexa-domain
             /    |    \
            ▼     ▼     ▼
      nexa-events │  nexa-nbp
             \    |    /
              \   |   /
               ▼  ▼  ▼
          nexa-orchestrator
                 │
        ┌────────┼─────────┐
        ▼        ▼         ▼
      tutor   speech    behavior
```

The orchestrator depends on contracts.

Runtime implementations depend outward from those contracts.

---

# 133. MVP Orchestrator Scope

The first implementation SHOULD support only:

```text
one student
one active session
text input
one Tutor Engine
one TTS provider
one avatar runtime
no labs
no assessments
no distributed messaging
```

But the public interfaces SHOULD preserve the architecture defined here.

---

# 134. MVP Vertical Slice

The first executable flow:

```text
start_session()
      ↓
student.text.submitted
      ↓
InteractionWorkflow
      ↓
TutorContext
      ↓
TutorEngine.respond()
      ↓
TutorResponse
      ↓
BehaviorIntent
      ↓
NBP command
      ↓
TTS
      ↓
Avatar behavior
      ↓
workflow.completed
```

---

# 135. MVP Acceptance Scenario

Student enters:

> "Explain the TCP three-way handshake."

Expected sequence:

```text
session active
      ↓
student input accepted
      ↓
Nexa → thinking
      ↓
TutorRequest constructed
      ↓
TutorResponse generated
      ↓
TCP canvas content displayed
      ↓
Nexa → explaining
      ↓
speech begins
      ↓
Nexa points to SYN
      ↓
Nexa points to SYN-ACK
      ↓
Nexa points to ACK
      ↓
speech completes
      ↓
Nexa asks verification question
      ↓
Nexa → waiting
```

All major transitions SHALL emit events.

---

# 136. Orchestrator Invariants

NEXA-ORCH-001 establishes the following invariants:

1. The orchestrator coordinates but does not replace domain engines.
2. Every meaningful student interaction SHALL have a workflow identity.
3. Workflows SHOULD propagate correlation and trace identifiers.
4. One MVP session SHALL have at most one primary conversational workflow.
5. All long-running work SHOULD support cancellation where feasible.
6. Tool operations SHALL pass through authorization.
7. Tutor output SHALL be validated before runtime execution.
8. The LLM SHALL NOT directly drive low-level animation.
9. Response execution SHOULD parallelize independent operations.
10. Capability failures SHOULD degrade gracefully where possible.
11. Session state SHALL be isolated between learners.
12. Important state changes SHALL be observable as events.
13. External service retries SHALL be bounded.
14. Timeout policies SHALL be explicit.
15. Runtime failures SHALL be classified.
16. Streaming output SHALL have clear commit boundaries.
17. Assessment policy SHALL override tutor preferences.
18. Security policy SHALL override tutor preferences.
19. Important side effects SHALL be idempotent.
20. The MVP design SHALL permit future distributed execution without changing core domain semantics.

---

# 137. Foundation Status

With this specification, the initial Nexa runtime foundation becomes:

```text
NEXA-CBS-001
Character & Behavior
       │
       ▼
NEXA-DOM-001
Core Domain Model
       │
       ├──────────────┐
       ▼              ▼
NEXA-EVT-001     NEXA-NBP-001
Event Model      Behavior Protocol
       │              │
       └───────┬──────┘
               ▼
        NEXA-ORCH-001
     Session Orchestrator
               │
               ▼
      Executable Nexa Runtime
```

We are now past purely conceptual architecture.

The next documents can define the actual **intelligence layer**.

---

# 138. Next Specification

The recommended next specification is:

# NEXA-PED-001 — Adaptive Pedagogy Engine & Learning Strategy Specification v1.0

It should formally define:

```text
pedagogical strategies
instruction selection
adaptive difficulty
hint escalation
mastery thresholds
misconception handling
student confidence
review scheduling
retrieval practice
Socratic instruction
guided discovery
feedback policies
challenge selection
lesson adaptation
evidence consumption
pedagogical state machines
pedagogy scoring
decision contracts
```

After that should come:

```text
NEXA-TUTOR-001
Tutor Intelligence & Reasoning Contract

NEXA-STU-001
Student Model & Competency Engine

NEXA-KNOW-001
Knowledge/RAG Architecture

NEXA-SPCH-001
Speech, STT, TTS & Lip-Sync Pipeline

NEXA-AVTR-001
Avatar Runtime & Animation Architecture
```

At that point, Nexa's architecture will cover the complete path from **learning science → AI reasoning → voice → animation → student feedback**.
