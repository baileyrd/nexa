NEXA-LAB-001 — Interactive Lab, Sandbox, Tool Execution & Observable Practice Architecture v1.0

Specification ID: NEXA-LAB-001
System: Nexa AI Training Tutor
Version: 1.0
Status: Baseline Draft
Depends On: NEXA-DOM-001, NEXA-EVT-001, NEXA-STU-001, NEXA-PED-001, NEXA-TUTOR-001, NEXA-KNOW-001, NEXA-ORCH-001, NEXA-LESSON-001, NEXA-ASMT-001
Purpose: Define the secure execution environment in which learners can perform real technical work while Nexa observes, teaches, evaluates, assists, and converts observable activity into learning evidence.

1. Purpose

The Lab subsystem answers:

“How can a learner safely perform real work while Nexa observes what happened, assists when permitted, and determines whether the intended outcome was achieved?”

Nexa SHALL not be limited to explaining technical concepts.

The platform SHOULD allow learners to:

write code
compile code
execute programs
use terminals
manipulate files
configure systems
inspect processes
debug failures
query databases
interact with APIs
analyze networks
operate simulated infrastructure
perform troubleshooting

within controlled environments.

2. Architectural Role
                   NEXA
                     │
                     ▼
               Lesson Engine
                     │
                     ▼
                 Lab Engine
                     │
       ┌─────────────┼─────────────┐
       ▼             ▼             ▼
   Environment     Tools       Observers
       │             │             │
       └─────────────┼─────────────┘
                     ▼
                Student Action
                     │
                     ▼
               Environment State
                     │
                     ▼
                Lab Evaluation
                     │
                     ▼
              Learning Evidence
                     │
                     ▼
                Student Model
3. Design Principle

A lab SHALL be treated as a controlled capability environment, not merely a shell embedded in the UI.

The architecture must answer:

What can the learner do?
Where can they do it?
What can Nexa do?
What can Nexa observe?
What is forbidden?
What constitutes success?
How can the environment be reset?
How can the activity be reproduced?
4. Core Responsibilities

The Lab subsystem SHALL own or coordinate:

lab definitions;
environment provisioning;
lifecycle management;
tool capabilities;
command execution;
process execution;
code execution;
filesystem operations;
network access;
resource quotas;
environment reset;
snapshots;
student-action observation;
objective evaluation;
artifact collection;
security boundaries;
policy enforcement;
timeout enforcement;
cancellation;
evidence generation;
lab versioning;
reproducibility.
5. Explicit Non-Responsibilities

The Lab subsystem SHALL NOT own:

long-term learner mastery;
curriculum sequencing;
general pedagogy;
final tutor wording;
avatar animation;
assessment grading outside lab-specific observations;
unrestricted host operating-system control.
6. Lab Definition
pub struct Lab {
    pub id: LabId,
    pub key: String,
    pub title: String,
    pub description: String,


    pub version: LabVersion,


    pub environment: EnvironmentDefinition,
    pub objectives: Vec<LabObjective>,
    pub tools: Vec<ToolCapabilityRequirement>,


    pub policy: LabPolicy,
    pub completion_policy: LabCompletionPolicy,
}
7. Lab Types
pub enum LabType {
    Terminal,
    Programming,
    Debugging,
    Networking,
    Database,
    OperatingSystem,
    DevOps,
    CyberRange,
    Simulation,
    Composite,
}

The architecture SHALL permit additional domain-specific lab types.

8. Lab Session

A definition describes the lab.

A session represents one learner's execution of that definition.

pub struct LabSession {
    pub id: LabSessionId,
    pub lab_id: LabId,
    pub lab_version: LabVersion,
    pub student_id: StudentId,


    pub environment_id: EnvironmentInstanceId,


    pub state: LabSessionState,


    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}
9. Session States
pub enum LabSessionState {
    Created,
    Provisioning,
    Ready,
    Active,
    Paused,
    Evaluating,
    Completed,
    Failed,
    Resetting,
    Terminated,
    Expired,
}
10. Lab Lifecycle
CREATE
   ↓
PROVISION
   ↓
READY
   ↓
START
   ↓
ACTIVE
   ├──── PAUSE
   │       ↓
   │     ACTIVE
   │
   ├──── RESET
   │       ↓
   │     ACTIVE
   │
   └──── COMPLETE
           ↓
        EVALUATE
           ↓
        DESTROY
11. Environment Abstraction
pub trait LabEnvironment {
    async fn provision(
        &self,
        request: ProvisionRequest,
    ) -> LabResult<EnvironmentInstance>;


    async fn snapshot(
        &self,
        environment: EnvironmentInstanceId,
    ) -> LabResult<EnvironmentSnapshot>;


    async fn restore(
        &self,
        snapshot: EnvironmentSnapshotId,
    ) -> LabResult<()>;


    async fn destroy(
        &self,
        environment: EnvironmentInstanceId,
    ) -> LabResult<()>;
}
12. Environment Providers

Nexa SHOULD support interchangeable providers.

pub enum EnvironmentProviderKind {
    LocalProcess,
    Container,
    VirtualMachine,
    RemoteSandbox,
    Simulator,
}

Future providers MAY include:

Kubernetes
cloud training environments
browser-based WASM sandboxes
network emulators
hardware simulators
13. Local-First Principle

The architecture SHOULD support local execution as a first-class deployment mode.

Example:

Nexa Desktop
    │
    ├── local container runtime
    ├── local compiler
    ├── local terminal
    └── local lab artifacts

Internet connectivity SHALL not be a fundamental architectural requirement.

14. Environment Definition
pub struct EnvironmentDefinition {
    pub provider: EnvironmentProviderKind,
    pub image: Option<EnvironmentImage>,
    pub resources: ResourceLimits,
    pub filesystem: FilesystemPolicy,
    pub network: NetworkPolicy,
}
15. Environment Image
pub struct EnvironmentImage {
    pub name: String,
    pub version: String,
    pub digest: Option<String>,
}

Production labs SHOULD prefer immutable digests where practical.

16. Reproducibility

The same lab version SHOULD resolve to a known environment.

Record:

lab version
environment image
image digest
tool versions
seed
configuration
17. Resource Limits
pub struct ResourceLimits {
    pub cpu_limit: Option<CpuLimit>,
    pub memory_limit: Option<ByteSize>,
    pub storage_limit: Option<ByteSize>,
    pub process_limit: Option<u32>,
    pub execution_timeout: Option<Duration>,
}
18. Why Resource Limits Matter

Learner code can accidentally create:

infinite loops
fork bombs
memory exhaustion
huge files
runaway processes

The lab SHALL contain such failures.

19. Filesystem Policy
pub struct FilesystemPolicy {
    pub writable_paths: Vec<PathPattern>,
    pub readonly_paths: Vec<PathPattern>,
    pub hidden_paths: Vec<PathPattern>,
    pub maximum_storage: Option<ByteSize>,
}
20. Host Filesystem Isolation

A lab SHALL NOT receive unrestricted access to the user's host filesystem.

Access SHALL be explicitly mounted or capability-authorized.

21. Workspace

Each lab SHOULD expose a dedicated learner workspace.

Example:

/workspace
├── README.md
├── src/
├── data/
└── output/
22. Persistent Versus Ephemeral Files
pub enum LabStorageClass {
    Ephemeral,
    SessionPersistent,
    CoursePersistent,
    Exportable,
}
23. Network Policy
pub enum NetworkPolicy {
    Disabled,
    InternalOnly,
    AllowList(Vec<NetworkTarget>),
    Internet,
}

Network access SHALL default to the least capability required by the lab.

24. Network Simulation

Networking courses SHOULD support isolated virtual networks.

client
  │
virtual network
  │
server

These networks SHOULD not require exposure to the host LAN.

25. Multi-Node Lab
pub struct LabTopology {
    pub nodes: Vec<LabNode>,
    pub links: Vec<LabLink>,
}

Example:

workstation
    │
 switch
    │
 router
   /   \
server  attacker-simulator
26. Lab Node
pub struct LabNode {
    pub id: LabNodeId,
    pub role: String,
    pub environment: EnvironmentDefinition,
}
27. Tool Capability Model

The learner and Nexa SHALL interact with labs through explicit capabilities.

pub struct ToolCapability {
    pub tool_id: ToolId,
    pub operations: Vec<ToolOperation>,
}
28. Tool Examples
terminal.execute
filesystem.read
filesystem.write
filesystem.list
process.list
process.kill
code.compile
code.execute
database.query
network.capture
network.inspect
http.request
29. Capability Principle

Possessing access to a lab SHALL NOT imply access to every lab operation.

Capabilities SHALL be independently authorized.

30. Tool Registry
pub struct ToolRegistry {
    pub tools: HashMap<ToolId, Arc<dyn Tool>>,
}
31. Tool Interface
#[async_trait]
pub trait Tool: Send + Sync {
    fn descriptor(&self) -> ToolDescriptor;


    async fn execute(
        &self,
        context: ToolExecutionContext,
        request: ToolRequest,
    ) -> ToolResult<ToolResponse>;
}
32. Tool Descriptor
pub struct ToolDescriptor {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub operations: Vec<ToolOperationDescriptor>,
}
33. Typed Operations

Tool operations SHOULD use typed schemas.

Example:

terminal.execute


input:
  command: string
  working_directory?: path
  timeout?: duration


output:
  exit_code: integer
  stdout: string
  stderr: string
34. No Arbitrary Tool Invocation

Tutor-generated text SHALL NOT directly become privileged execution.

Instead:

Tutor Intent
    ↓
Tool Request
    ↓
Schema Validation
    ↓
Authorization
    ↓
Execution
35. Student Versus Tutor Capabilities

The learner and Nexa MAY have different capability sets.

pub struct LabActorCapabilities {
    pub student: CapabilitySet,
    pub tutor: CapabilitySet,
}
36. Example

The student may have:

terminal.execute
filesystem.write

while Nexa has:

filesystem.read
process.inspect

but NOT:

terminal.execute

unless assistance policy permits it.

37. Tutor Assistance Levels
pub enum LabTutorAssistance {
    ObserveOnly,
    ExplainOnly,
    SuggestActions,
    PrepareAction,
    ExecuteWithApproval,
    AutonomousApproved,
}
38. Observe Only

Nexa can inspect activity but cannot perform actions.

This is particularly appropriate for assessment labs.

39. Suggest Actions

Nexa may say:

“Check whether the service is listening on the expected port.”

But SHALL not run the command herself.

40. Prepare Action

Nexa may construct:

ss -ltnp

for learner review.

The learner still executes it.

41. Execute With Approval

Flow:

Nexa proposes action
      ↓
student approves
      ↓
authorization
      ↓
tool executes
42. Autonomous Approved

Only explicitly permitted instructional contexts SHOULD allow Nexa to execute actions without per-action approval.

43. Assessment Override

NEXA-ASMT-001 SHALL be capable of reducing assistance.

Example:

lesson policy:
    SuggestActions


assessment policy:
    ObserveOnly

Assessment policy wins.

44. Tool Request
pub struct ToolRequest {
    pub id: ToolRequestId,
    pub tool_id: ToolId,
    pub operation: String,
    pub arguments: JsonValue,
}
45. Tool Execution Context
pub struct ToolExecutionContext {
    pub session_id: SessionId,
    pub lab_session_id: LabSessionId,
    pub actor: Actor,
    pub capabilities: CapabilitySet,
    pub policy: EffectiveToolPolicy,
}
46. Actor
pub enum Actor {
    Student,
    Tutor,
    System,
    Evaluator,
    Instructor,
}
47. Authorization

Before execution:

Request
   ↓
Tool exists?
   ↓
Operation exists?
   ↓
Actor authorized?
   ↓
Lab permits it?
   ↓
Assessment permits it?
   ↓
Arguments valid?
   ↓
Execute
48. Denied Action

Denied requests SHALL produce structured results.

pub struct ToolDenied {
    pub reason: ToolDenialReason,
}
49. Denial Reasons
pub enum ToolDenialReason {
    CapabilityMissing,
    PolicyRestricted,
    AssessmentRestricted,
    InvalidArguments,
    ResourceLimit,
    EnvironmentUnavailable,
}
50. Terminal Tool
pub struct TerminalExecuteRequest {
    pub command: String,
    pub working_directory: Option<String>,
    pub timeout: Option<Duration>,
}
51. Terminal Response
pub struct TerminalExecuteResponse {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub timed_out: bool,
}
52. Streaming Output

Long-running commands SHOULD support streaming.

command starts
    ↓
stdout chunk
stdout chunk
stderr chunk
...
    ↓
command exits
53. Process Handle
pub struct ProcessHandle {
    pub process_id: ProcessId,
    pub started_at: Timestamp,
}

The runtime SHOULD avoid exposing raw host PIDs where unnecessary.

54. Cancellation

Long-running tool actions SHALL be cancellable.

async fn cancel(
    &self,
    execution_id: ToolExecutionId,
) -> ToolResult<()>;
55. Timeouts

Every executable capability SHOULD support an upper-bound timeout.

Lab-wide policies MAY impose a stricter maximum.

56. Output Limits

Tool output SHALL be bounded.

Example:

maximum stdout bytes
maximum stderr bytes
maximum event rate

Large outputs MAY be persisted as artifacts instead.

57. Artifact Capture
pub struct LabArtifact {
    pub id: ArtifactId,
    pub lab_session_id: LabSessionId,
    pub kind: LabArtifactKind,
    pub location: ArtifactLocation,
    pub hash: ContentHash,
}
58. Artifact Types
pub enum LabArtifactKind {
    SourceCode,
    Binary,
    TextFile,
    Log,
    PacketCapture,
    Screenshot,
    Database,
    Report,
    Archive,
    Other,
}
59. Artifact Integrity

Important lab artifacts SHOULD be hashed.

This supports:

evaluation
reproducibility
audit
change detection
60. Observation System

Nexa SHALL observe structured lab events rather than rely only on terminal text.

pub enum LabObservation {
    CommandExecuted,
    FileCreated,
    FileModified,
    ProcessStarted,
    ProcessExited,
    NetworkConnection,
    TestExecuted,
    CompilerDiagnostic,
    EnvironmentStateChanged,
}
61. Observation Record
pub struct Observation {
    pub id: ObservationId,
    pub lab_session_id: LabSessionId,
    pub actor: Actor,
    pub kind: LabObservation,
    pub timestamp: Timestamp,
    pub data: JsonValue,
}
62. Why Structured Observation Matters

Suppose the learner executes:

cargo test

Nexa SHOULD receive more than terminal text.

Ideally:

operation = test
framework = cargo
passed = 18
failed = 2
exit_code = 101

This enables reliable tutoring.

63. Observer Architecture
Environment
    │
    ├── Terminal Observer
    ├── Filesystem Observer
    ├── Process Observer
    ├── Network Observer
    └── Tool Observer
            │
            ▼
      Observation Bus
64. Observation Bus

The Lab subsystem SHOULD publish observations through NEXA-EVT-001.

65. Observation Filtering

Not every low-level event belongs in TutorContext.

raw observations
      ↓
filter / aggregate
      ↓
pedagogically relevant state
      ↓
TutorContext
66. Student Action History
pub struct StudentAction {
    pub id: StudentActionId,
    pub lab_session_id: LabSessionId,
    pub action_type: StudentActionType,
    pub timestamp: Timestamp,
    pub outcome: ActionOutcome,
}
67. Action Types
pub enum StudentActionType {
    Command,
    FileEdit,
    CodeRun,
    TestRun,
    DebugAction,
    NetworkAction,
    DatabaseAction,
    ToolAction,
}
68. Observable Reasoning

Nexa SHOULD infer instructional needs from observable actions.

Example:

student runs:
ping
ping
ping
ping

Nexa may recognize:

repetitive diagnostic behavior

and suggest broadening the troubleshooting approach.

69. Do Not Require Hidden Thought

Evaluation SHALL use observable evidence:

commands
files
results
explanations
hypotheses explicitly entered
environment changes

not inaccessible private reasoning.

70. Lab Objective
pub struct LabObjective {
    pub id: LabObjectiveId,
    pub learning_objective_id: LearningObjectiveId,
    pub competency_ids: Vec<CompetencyId>,
    pub evaluation: LabObjectiveEvaluation,
}
71. Objective Evaluation
pub enum LabObjectiveEvaluation {
    FinalState(StatePredicate),
    ObservationSequence(SequencePredicate),
    Artifact(ArtifactPredicate),
    CommandOutcome(CommandPredicate),
    Tests(TestSuiteId),
    Rubric(RubricId),
    Composite(Vec<LabObjectiveEvaluation>),
}
72. Final-State Evaluation

Example:

Configure the service to listen on port 8080.

Evaluation:

service running
AND
port 8080 listening

The exact commands used need not matter.

73. Process Evaluation

Sometimes the method matters.

Example:

Diagnose the problem without restarting the server.

Then evaluation MAY inspect whether:

restart action occurred
74. Observation Sequence
pub struct SequencePredicate {
    pub required_events: Vec<ObservationPattern>,
    pub ordering: SequenceOrdering,
}
75. Sequence Ordering
pub enum SequenceOrdering {
    Exact,
    Relative,
    Any,
}
76. Avoid Over-Specification

A lab SHOULD not require an exact command sequence unless that sequence itself is the competency.

Otherwise multiple valid approaches SHOULD pass.

77. State Predicate
pub struct StatePredicate {
    pub checks: Vec<StateCheck>,
}

Examples:

file exists
file contains value
process running
port open
database row exists
test passes
78. Objective Result
pub struct LabObjectiveResult {
    pub objective_id: LabObjectiveId,
    pub outcome: EvaluationOutcome,
    pub score: Score,
    pub evidence: Vec<ObservationId>,
}
79. Lab Completion
pub enum LabCompletionPolicy {
    AllRequiredObjectives,
    ScoreThreshold(Score),
    RequiredObjectives(Vec<LabObjectiveId>),
    AssessmentControlled,
}
80. Lab Evaluation
pub struct LabEvaluationResult {
    pub lab_session_id: LabSessionId,
    pub objectives: Vec<LabObjectiveResult>,
    pub score: Option<Score>,
    pub completed: bool,
}
81. Evidence Generation
Lab Objective Result
       +
Assistance Level
       +
Observation History
       +
Evaluator Confidence
       ↓
Learning Evidence
       ↓
Student Model
82. Assistance Matters

If Nexa performed most of the task, successful final state SHOULD not imply independent mastery.

Example:

student performed task independently
    → strong evidence


Nexa suggested command
    → moderate evidence


Nexa executed command
    → weak evidence
83. Assistance Trace
pub struct AssistanceTrace {
    pub tutor_suggestions: u32,
    pub tutor_prepared_actions: u32,
    pub tutor_executed_actions: u32,
}
84. Evidence Independence

Lab evidence SHOULD record an independence classification compatible with NEXA-STU-001.

85. Environment Snapshot
pub struct EnvironmentSnapshot {
    pub id: EnvironmentSnapshotId,
    pub environment_id: EnvironmentInstanceId,
    pub created_at: Timestamp,
    pub provider_reference: String,
}
86. Snapshot Use Cases

Snapshots support:

lab reset
checkpoint
assessment review
debugging replay
instructor inspection
87. Baseline Snapshot

Every resettable lab SHOULD have a known baseline.

baseline
   ↓
student activity
   ↓
reset
   ↓
baseline
88. Checkpoints

Some labs MAY permit learner-created checkpoints.

pub struct LabCheckpoint {
    pub id: LabCheckpointId,
    pub snapshot_id: EnvironmentSnapshotId,
    pub label: Option<String>,
}
89. Reset Policy
pub enum LabResetPolicy {
    Disabled,
    Unlimited,
    Limited(u32),
    InstructorOnly,
}
90. Assessment Reset

Assessment mode MAY override the normal lab reset policy.

91. Pause

Pause semantics depend on environment provider.

A pause MAY:

freeze environment

or:

persist state and destroy runtime

with later restoration.

92. Expiration

Lab environments SHOULD have lifecycle expiration.

pub struct LabLease {
    pub expires_at: Timestamp,
}
93. Lease Extension

Long lessons MAY renew active leases according to policy.

94. Cleanup

Environment cleanup SHALL be robust against:

client crash
application crash
network loss
student abandonment
95. Orphan Detection

The runtime SHOULD detect abandoned environments and reclaim resources.

96. Container Provider

Containers are a strong baseline provider for programming and terminal labs.

Lab Engine
   ↓
Container Provider
   ↓
isolated container
97. Container Security

Container labs SHOULD use:

non-root user
limited capabilities
resource quotas
filesystem restrictions
network policy
process limits

where supported.

98. Containers Are Not Absolute Isolation

The architecture SHALL NOT assume that containers provide the strongest possible isolation.

Higher-risk workloads MAY require VM-backed environments.

99. VM Provider

Virtual machines MAY be appropriate for:

kernel training
operating-system administration
network appliances
higher-isolation workloads
100. Simulator Provider

Some lessons SHOULD use simulated environments rather than real systems.

Examples:

router configuration simulator
packet-routing simulator
embedded hardware simulator
cloud architecture simulator
101. Browser/WASM Provider

Future programming labs MAY execute suitable workloads using WASM-based sandboxing.

This can provide lightweight local/offline labs.

102. Remote Provider
pub trait RemoteLabProvider {
    async fn allocate(
        &self,
        request: RemoteLabRequest,
    ) -> LabResult<RemoteLab>;
}

Remote providers SHALL still obey the same logical capability and observation contracts.

103. Provider Independence

Lessons SHALL target lab capabilities, not Docker-specific or cloud-specific internals where avoidable.

104. Lab Manifest
lab:
  id: rust-ownership-001
  version: 1.0.0
  type: programming


environment:
  provider: container
  image: nexa/rust-lab:1.0
  memory: 512MiB
  cpu: 1


network:
  policy: disabled


workspace:
  template: workspace/


tools:
  - terminal.execute
  - filesystem.read
  - filesystem.write
  - code.execute


objectives:
  - ownership.compile
  - ownership.explain
105. Networking Lab Manifest
lab:
  id: tcp-handshake-debug
  version: 1.0.0
  type: networking


topology:
  nodes:
    - client
    - server


network:
  external_access: false


tools:
  - terminal.execute
  - network.capture
  - network.inspect
106. Lab Package

Recommended structure:

labs/
└── tcp-handshake-debug/
    ├── lab.yaml
    ├── environment/
    ├── workspace/
    ├── objectives/
    ├── tests/
    ├── fixtures/
    └── protected/
107. Protected Lab Assets

The following MAY be protected:

hidden tests
expected state
solution files
assessment scripts
grading rules

They SHALL not be exposed to learner tools.

108. Lab Compiler
pub trait LabCompiler {
    fn compile(
        &self,
        source: LabSource,
    ) -> LabResult<CompiledLab>;
}
109. Validation

Before publication validate:

manifest schema
environment availability
tool requirements
objective references
resource limits
network policy
reset capability
protected asset isolation
110. Capability Validation

A course SHOULD fail validation if a required lab demands capabilities unavailable in the target deployment.

Unless the capability is explicitly optional.

111. Lab Test

Each lab SHOULD support automated validation.

Example:

provision
   ↓
verify baseline
   ↓
execute reference solution
   ↓
evaluate objectives
   ↓
reset
   ↓
verify baseline restored
112. Reference Solution

A protected reference solution MAY exist for automated lab testing.

It SHALL not be made available to the learner runtime.

113. Lab Linter

Future CLI:

nexa lab validate ./labs/tcp-handshake-debug
114. Lab Smoke Test
nexa lab test ./labs/tcp-handshake-debug

SHOULD verify provisioning, tooling, objectives, and cleanup.

115. Lab Events

Canonical events include:

lab.created
lab.provisioning
lab.ready
lab.started
lab.paused
lab.resumed
lab.reset
lab.completed
lab.failed
lab.expired
lab.destroyed


lab.tool.requested
lab.tool.approved
lab.tool.denied
lab.tool.started
lab.tool.output
lab.tool.completed
lab.tool.failed
lab.tool.cancelled


lab.observation.created
lab.objective.satisfied
lab.objective.failed


lab.snapshot.created
lab.snapshot.restored


lab.artifact.created
116. Tool Requested Event
{
  "event_type": "lab.tool.requested",
  "payload": {
    "lab_session_id": "lab-session-42",
    "actor": "student",
    "tool": "terminal",
    "operation": "execute"
  }
}
117. Tool Completed Event
{
  "event_type": "lab.tool.completed",
  "payload": {
    "execution_id": "exec-9",
    "exit_code": 0,
    "duration_ms": 221
  }
}
118. Sensitive Output

Tool output MAY contain:

credentials
tokens
private paths
environment variables

Observation and logging layers SHOULD support redaction.

119. Secret Model
pub struct SecretReference {
    pub id: SecretId,
}

Labs SHOULD consume secret references rather than embed plaintext secrets in manifests.

120. Secret Injection

Where required:

Secret Store
    ↓
authorized lab
    ↓
temporary environment injection

Secrets SHALL NOT automatically become learner-visible.

121. No Secret Logging

Tools and event handlers SHOULD avoid logging secrets.

122. Environment Variables

Lab manifests MAY define ordinary environment variables separately from secrets.

123. Network Credentials

Training systems SHOULD prefer synthetic credentials and isolated training resources.

124. Destructive Actions

A lab MAY intentionally teach destructive commands.

Example:

rm
DROP TABLE
kill
firewall changes

These SHALL remain contained within the lab environment.

125. Destructive Action Classification
pub enum ActionRisk {
    ReadOnly,
    Mutating,
    Destructive,
    Privileged,
}
126. Risk-Aware Authorization

Higher-risk operations MAY require additional authorization even inside a lab.

127. Host Boundary

No lab policy SHALL grant access outside the designated environment merely because an instructional task requests it.

128. Tool Input Validation

Arguments SHALL be validated before execution.

This is especially important for structured tools such as:

filesystem.write
database.query
http.request
process.kill
129. Terminal Exception

A terminal intentionally permits general command syntax.

Therefore isolation SHALL be enforced at the environment boundary rather than attempting to parse every possible command safely.

130. Lab State Model
pub struct LabState {
    pub environment: EnvironmentState,
    pub objectives: Vec<LabObjectiveState>,
    pub assistance: AssistanceTrace,
    pub artifacts: Vec<ArtifactId>,
}
131. Objective State
pub enum LabObjectiveState {
    NotStarted,
    InProgress,
    Satisfied,
    Failed,
}
132. Continuous Objective Detection

Some objectives MAY be evaluated continuously.

Example:

student starts required service
      ↓
observer detects state
      ↓
objective satisfied
133. Completion Confirmation

For significant objectives, the system MAY require state stability or explicit final evaluation before marking complete.

134. Example Programming Lab

Objective:

Fix the Rust program so all tests pass.

Flow:

Nexa presents broken project
      ↓
student opens source
      ↓
student edits code
      ↓
cargo test
      ↓
2 failures
      ↓
Nexa observes diagnostics
      ↓
student edits
      ↓
cargo test
      ↓
all tests pass
      ↓
objective satisfied
135. Example Networking Lab

Objective:

Determine why the TCP connection cannot be established.

Environment:

client ───── server

Hidden defect:

server port closed

Learner MAY use:

ping
ss
tcpdump
nc

The evaluator scores:

correct root cause
appropriate evidence
verified resolution
136. Example Database Lab

Objective:

Correct a broken SQL query and return the required rows.

Evaluation SHOULD preferably inspect:

query result

rather than requiring an exact SQL statement.

137. Example Debugging Lab

Broken application:

service fails at startup

Nexa observes:

student checks status
student reads logs
student identifies config error
student corrects file
student restarts service
student verifies healthy state

This produces richer evidence than a multiple-choice question.

138. Nexa's Lab Awareness

TutorContext SHOULD include a summarized lab state.

pub struct TutorLabContext {
    pub lab_id: LabId,
    pub current_objective: Option<LabObjectiveId>,
    pub recent_actions: Vec<ActionSummary>,
    pub relevant_observations: Vec<ObservationSummary>,
    pub assistance_policy: LabTutorAssistance,
}
139. Context Compression

Long lab histories SHALL be summarized.

Do not continually send thousands of terminal lines into the tutor model.

140. Relevant Observation Selection

Selection SHOULD prioritize:

errors
state transitions
failed tests
objective-related events
recent student actions
repeated behavior
141. Tutor Intervention

Pedagogy determines whether Nexa should intervene.

Observation
    ↓
Student Model / Pedagogy
    ↓
intervene?
   /       \
 no        yes
           ↓
       Tutor Engine
142. Avoid Over-Tutoring

Nexa SHOULD not interrupt after every failed command.

Productive struggle is part of learning.

143. Stuck Detection

Possible indicators:

repeated same command
repeated same error
long inactivity
cycling between ineffective actions
many unsuccessful attempts

These signals SHALL be advisory, not proof of confusion.

144. Lab Hints

Hints MAY be progressively disclosed.

Example:

Hint 1:
Inspect whether the service is running.


Hint 2:
Check listening sockets.


Hint 3:
Try `ss -ltnp`.
145. Hint Evidence

Lab hint usage SHALL be included in assistance/evidence metadata.

146. Solution Reveal

Full solutions SHOULD require explicit policy.

Practice lab:

may allow after sufficient attempts

Assessment lab:

normally prohibited
147. Lab Assessment Mode

A lab MAY operate under Assessment Engine control.

Lab Engine
    │
    ▼
Assessment Policy
    │
    ├── disable tutor execution
    ├── disable hints
    ├── restrict network
    └── protect solution assets
148. Assessment Lab Completion

The Lab Engine provides observations and objective results.

The Assessment Engine determines:

score
pass/fail
feedback disclosure
149. Replay

Lab activity SHOULD be replayable at the semantic event level.

12:01 command executed
12:02 file changed
12:03 test failed
12:05 file changed
12:06 test passed
150. Replay Is Not Necessarily Environment Reexecution

Semantic replay and actual deterministic reexecution are different capabilities.

Both MAY eventually be supported.

151. Environment Export

Some labs MAY permit students to export:

source files
reports
packet captures
logs

according to policy.

152. Import

A lab MAY allow importing learner files.

Imported artifacts SHALL be treated as untrusted input.

153. Malware and Untrusted Code Boundary

The architecture SHALL assume learner-provided code may behave unexpectedly.

Execution isolation SHALL not depend on learner intent.

154. Provider Health

The Lab Engine SHOULD monitor provider availability.

pub enum ProviderHealth {
    Healthy,
    Degraded,
    Unavailable,
}
155. Graceful Degradation

If a lab provider is unavailable:

required lab
    → lesson blocked or deferred


optional lab
    → alternate activity
156. Offline Capability Declaration
pub enum OfflineCapability {
    Full,
    Partial,
    None,
}

Course catalogs SHOULD expose this eventually.

157. Lab Analytics

Useful metrics include:

completion rate
average completion time
reset frequency
hint usage
failed command frequency
common failure state
objective failure rate
tool usage
assistance level
158. Learning Analytics

More interesting questions include:

Which actions correlate with successful learning?
Where do students become stuck?
Which hint resolves the issue?
Which lab objective is poorly designed?
Does the reference solution represent how learners actually solve it?
159. Lab Quality Feedback

Lab analytics SHOULD feed authoring improvements.

learner sessions
      ↓
analytics
      ↓
lab author
      ↓
new lab version
160. Lab Version Migration

Active sessions SHOULD normally remain pinned to the version on which they started.

New sessions use the updated version.

161. Emergency Lab Disable

A lab version SHALL be disableable if a security or correctness problem is discovered.

162. Lab Status
pub enum LabStatus {
    Draft,
    Review,
    Active,
    Suspended,
    Retired,
}
163. Lab Security Validation

Publication SHOULD verify:

no unsafe host mounts
no unintended privileged mode
network restrictions valid
protected assets inaccessible
resource limits present
secret handling valid
164. Security Test Suite

Lab infrastructure SHOULD eventually include adversarial tests for isolation boundaries.

165. Observability

Operational telemetry SHOULD include:

provision latency
tool execution latency
provider failures
resource exhaustion
cleanup failures
orphan environments
166. Trace Correlation

Every lab operation SHOULD carry:

session_id
lab_session_id
execution_id
trace_id

where applicable.

167. Error Model
pub enum LabError {
    LabNotFound,
    ProvisionFailed,
    EnvironmentUnavailable,
    ToolUnavailable,
    ToolDenied,
    ToolFailed,
    Timeout,
    ResourceExceeded,
    SnapshotFailed,
    RestoreFailed,
    EvaluationFailed,
    PolicyViolation,
    ProviderFailure,
}
168. Error Recovery

Recoverable infrastructure errors SHOULD remain distinct from learner mistakes.

Example:

compiler returns syntax error
    → learner outcome


container provider crashes
    → infrastructure error

Never grade infrastructure failure as learner failure.

169. First MVP

The initial Lab implementation SHOULD target:

Provider:
  local container


Lab types:
  terminal
  programming


Tools:
  terminal.execute
  filesystem.read
  filesystem.write
  filesystem.list


Observation:
  commands
  exit codes
  stdout/stderr
  file changes
  test results


Lifecycle:
  provision
  start
  reset
  complete
  destroy
170. MVP Security Baseline

The first implementation SHOULD enforce:

non-root container
no host network
no arbitrary host mounts
CPU quota
memory quota
process limit
storage quota
execution timeout
restricted environment variables
171. MVP Rust Lab
Course
  ↓
Rust Ownership Lesson
  ↓
Programming Lab
  ↓
Container:
  Rust toolchain
  broken source project
  tests

Student task:

Fix the ownership errors and make all tests pass.

172. MVP Interaction
Nexa:
"Compile the project and see what Rust tells us."


Student:
cargo test


Lab:
compiler errors


Nexa:
"Look at the borrow on line 17. What is still holding ownership there?"


Student edits code.


Student:
cargo test


Lab:
all tests pass


Nexa:
"Good. Now explain why your change fixed the ownership problem."

This combines:

execution
observation
tutoring
assessment
explanation
173. MVP Networking Evolution

Once container labs are stable:

single container
      ↓
multiple containers
      ↓
isolated virtual network
      ↓
packet capture
      ↓
network troubleshooting labs
174. Recommended Crate Structure
crates/
└── nexa-labs/
    ├── src/
    │   ├── lib.rs
    │   ├── engine.rs
    │   ├── lab.rs
    │   ├── session.rs
    │   ├── environment.rs
    │   ├── topology.rs
    │   ├── capability.rs
    │   ├── tool.rs
    │   ├── registry.rs
    │   ├── authorization.rs
    │   ├── observation.rs
    │   ├── objective.rs
    │   ├── evaluation.rs
    │   ├── evidence.rs
    │   ├── artifact.rs
    │   ├── snapshot.rs
    │   ├── policy.rs
    │   ├── manifest.rs
    │   ├── compiler.rs
    │   ├── validation.rs
    │   ├── errors.rs
    │   │
    │   ├── providers/
    │   │   ├── mod.rs
    │   │   ├── local.rs
    │   │   ├── container.rs
    │   │   ├── vm.rs
    │   │   └── remote.rs
    │   │
    │   ├── tools/
    │   │   ├── terminal.rs
    │   │   ├── filesystem.rs
    │   │   ├── process.rs
    │   │   ├── code.rs
    │   │   ├── database.rs
    │   │   └── network.rs
    │   │
    │   └── observers/
    │       ├── terminal.rs
    │       ├── filesystem.rs
    │       ├── process.rs
    │       └── network.rs
    │
    └── tests/
        ├── lifecycle.rs
        ├── capabilities.rs
        ├── authorization.rs
        ├── isolation.rs
        ├── reset.rs
        ├── objective.rs
        ├── evidence.rs
        └── cleanup.rs
175. Dependency Direction
                   nexa-domain
                        │
                        ▼
                    nexa-labs
                  /     │      \
                 ▼      ▼       ▼
             events   tools   providers
                 \      │      /
                  \     │     /
                   ▼    ▼    ▼
                 orchestrator
                       │
              ┌────────┴────────┐
              ▼                 ▼
           lessons          assessment
176. Lab System Invariants

NEXA-LAB-001 establishes the following invariants:

Labs SHALL execute inside defined environment boundaries.
Host-system access SHALL not be implicitly granted.
Tool access SHALL be capability-based.
Student and Tutor capabilities MAY differ.
Assessment policy SHALL be able to restrict lab assistance.
Tutor-generated text SHALL not directly become privileged execution.
Tool requests SHALL pass schema validation and authorization.
Executable operations SHOULD have resource limits.
Long-running operations SHALL support timeout and cancellation.
Tool output SHOULD be bounded.
Important lab activity SHALL produce structured observations.
Raw observations SHALL be filtered before entering TutorContext.
Lab evaluation SHALL prefer observable behavior and environment state.
Hidden learner reasoning SHALL not be required.
Multiple valid solution paths SHOULD be supported unless method is the competency.
Infrastructure failures SHALL not be graded as learner failures.
Lab definitions SHALL be versioned.
Active sessions SHALL remain pinned to their lab version.
Lab environments SHOULD be reproducible.
Important artifacts SHOULD support integrity hashing.
Protected solutions and hidden tests SHALL remain learner-inaccessible.
Network access SHALL follow explicit policy.
Secrets SHALL not be embedded in ordinary manifests or logs.
Destructive actions SHALL remain confined to the lab environment.
Resettable labs SHALL have a known baseline.
Abandoned environments SHALL be reclaimable.
Assistance SHALL influence learning-evidence independence.
Lab logic SHALL be testable without the avatar or UI.
Provider-specific infrastructure SHALL remain behind environment abstractions.
A successful final state SHALL not automatically imply independent mastery.
177. Architecture Status

With NEXA-LAB-001, Nexa's core learning loop now becomes:

                         STUDENT
                            │
                            ▼
                     ┌────────────┐
                     │    NEXA    │
                     └─────┬──────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
          Knowledge      Lesson      Student Model
              │            │            │
              └──────┬─────┴─────┬──────┘
                     ▼           ▼
                 Pedagogy     Assessment
                     │           │
                     └─────┬─────┘
                           ▼
                     Tutor Engine
                           │
                     Orchestrator
                    /      │       \
                   /       │        \
                  ▼        ▼         ▼
              Avatar     Speech      Lab
                                     │
                         ┌───────────┼───────────┐
                         ▼           ▼           ▼
                     Terminal      Code       Network
                         │           │           │
                         └───────────┼───────────┘
                                     ▼
                               Observation
                                     │
                                     ▼
                                  Evidence
                                     │
                                     ▼
                               Student Model

This is a significant architectural boundary.

Nexa can now teach a concept, demonstrate it, ask the learner to perform it, watch the learner work, recognize failure states, provide controlled assistance, evaluate the resulting system state, and convert that performance into mastery evidence.

178. What We Should Build Next

We have now defined what Nexa teaches, how she assesses it, and where learners practice it.

The next architectural gap is the layer the learner actually experiences.

NEXA-UX-001 — Tutor Workspace, Training Runtime & Human–Avatar Interaction Architecture v1.0

This should define the complete student-facing workspace:

┌─────────────────────────────────────────────────────────────┐
│ Course / Lesson                          Progress     Nexa   │
├──────────────────────┬──────────────────────────────────────┤
│                      │                                      │
│                      │             LESSON CANVAS            │
│        NEXA          │                                      │
│                      │    diagrams / code / instructions    │
│      animated        │                                      │
│       avatar         ├──────────────────────────────────────┤
│                      │                                      │
│                      │          INTERACTIVE LAB             │
│                      │                                      │
│                      │     terminal / editor / output       │
│                      │                                      │
├──────────────────────┴──────────────────────────────────────┤
│ 🎤 Ask Nexa...                              Send            │
└─────────────────────────────────────────────────────────────┘

That specification should define workspace composition, avatar placement, lesson canvas, terminal/editor panels, voice/text input, subtitles, pointing and gaze targets, focus management, panel orchestration, adaptive layouts, accessibility, keyboard control, progress visualization, notifications, assessment mode, fullscreen lab mode, responsive behavior, and the event contract between the UI and every architecture component we've designed.

After NEXA-UX-001, we should define NEXA-AUTHOR-001, the course/lab/assessment authoring system. That is where this architecture starts becoming a complete training platform, rather than only an AI tutor runtime.
