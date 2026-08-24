//! Deterministic, synchronous session and interaction-workflow lifecycle contracts.
#![forbid(unsafe_code)]

use nexa_domain::{CorrelationId, ProtocolVersion, SessionId, TraceId, WorkflowId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Wire protocol version for the lifecycle foundation accepted by ADR-0051.
pub const ORCHESTRATOR_LIFECYCLE_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
/// Wire protocol version for cancellation propagation plans accepted by ADR-0052.
pub const CANCELLATION_PROPAGATION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

/// The closed runtime session-state vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// A rejected session lifecycle transition. It carries no session content.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("illegal runtime session state transition")]
pub struct SessionTransitionError;

impl RuntimeSessionState {
    /// Applies an expressly supported lifecycle transition without side effects.
    pub fn transition_to(self, next: Self) -> Result<Self, SessionTransitionError> {
        use RuntimeSessionState::*;
        let ordinary = matches!(
            (self, next),
            (Created, Initializing)
                | (Initializing, Ready)
                | (Ready, Active)
                | (Active, Paused)
                | (Paused, Active)
                | (Active, Degraded)
                | (Degraded, Active)
                | (Active, Ending)
                | (Paused, Ending)
                | (Degraded, Ending)
                | (Ending, Completed)
        );
        let failure = !matches!(self, Completed | Failed) && next == Failed;
        (ordinary || failure)
            .then_some(next)
            .ok_or(SessionTransitionError)
    }
}

/// The closed interaction-workflow state vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// A closed, content-free workflow lifecycle failure category.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WorkflowLifecycleError {
    #[error("unsupported workflow lifecycle version")]
    UnsupportedVersion,
    #[error("illegal workflow lifecycle transition")]
    IllegalTransition,
    #[error("workflow lifecycle identity association mismatch")]
    AssociationMismatch,
}

/// Reference-only identity and current state for one interaction workflow.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InteractionWorkflow {
    workflow_id: WorkflowId,
    session_id: SessionId,
    correlation_id: CorrelationId,
    trace_id: TraceId,
    state: WorkflowState,
}

impl InteractionWorkflow {
    pub const fn new(
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
    ) -> Self {
        Self {
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            state: WorkflowState::Created,
        }
    }
    pub const fn workflow_id(&self) -> WorkflowId {
        self.workflow_id
    }
    pub const fn session_id(&self) -> SessionId {
        self.session_id
    }
    pub const fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }
    pub const fn trace_id(&self) -> TraceId {
        self.trace_id
    }
    pub const fn state(&self) -> WorkflowState {
        self.state
    }

    /// Validates the aggregate's complete identity association against trusted references.
    pub fn validate_association(
        &self,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
    ) -> Result<(), WorkflowLifecycleError> {
        if (
            self.workflow_id,
            self.session_id,
            self.correlation_id,
            self.trace_id,
        ) != (workflow_id, session_id, correlation_id, trace_id)
        {
            return Err(WorkflowLifecycleError::AssociationMismatch);
        }
        Ok(())
    }

    /// Advances through one transition selected by ADR-0051.
    pub fn advance(self, next: WorkflowState) -> Result<Self, WorkflowLifecycleError> {
        use WorkflowState::*;
        let legal = matches!(
            (self.state, next),
            (Created, NormalizingInput)
                | (NormalizingInput, PreparingContext)
                | (PreparingContext, SelectingPedagogy)
                | (SelectingPedagogy, RetrievingKnowledge)
                | (RetrievingKnowledge, GeneratingTutorResponse)
                | (GeneratingTutorResponse, ExecutingTools)
                | (GeneratingTutorResponse, PlanningResponse)
                | (ExecutingTools, PlanningResponse)
                | (PlanningResponse, Speaking)
                | (PlanningResponse, WaitingForStudent)
                | (Speaking, WaitingForStudent)
                | (WaitingForStudent, Completed)
        );
        if !legal {
            return Err(WorkflowLifecycleError::IllegalTransition);
        }
        Ok(Self {
            state: next,
            ..self
        })
    }

    /// Requests lifecycle cancellation. Repeating it after cancellation is idempotent.
    pub fn cancel(self) -> Result<Self, WorkflowLifecycleError> {
        match self.state {
            WorkflowState::Completed | WorkflowState::Failed => {
                Err(WorkflowLifecycleError::IllegalTransition)
            }
            WorkflowState::Cancelled => Ok(self),
            _ => Ok(Self {
                state: WorkflowState::Cancelled,
                ..self
            }),
        }
    }

    /// Moves any nonterminal workflow to its closed failure state.
    pub fn fail(self) -> Result<Self, WorkflowLifecycleError> {
        match self.state {
            WorkflowState::Completed | WorkflowState::Cancelled | WorkflowState::Failed => {
                Err(WorkflowLifecycleError::IllegalTransition)
            }
            _ => Ok(Self {
                state: WorkflowState::Failed,
                ..self
            }),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowWire {
    version: ProtocolVersion,
    workflow_id: WorkflowId,
    session_id: SessionId,
    correlation_id: CorrelationId,
    trace_id: TraceId,
    state: WorkflowState,
}

impl Serialize for InteractionWorkflow {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WorkflowWire {
            version: ORCHESTRATOR_LIFECYCLE_V1,
            workflow_id: self.workflow_id,
            session_id: self.session_id,
            correlation_id: self.correlation_id,
            trace_id: self.trace_id,
            state: self.state,
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for InteractionWorkflow {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = WorkflowWire::deserialize(deserializer)?;
        if wire.version != ORCHESTRATOR_LIFECYCLE_V1 {
            return Err(serde::de::Error::custom(
                WorkflowLifecycleError::UnsupportedVersion,
            ));
        }
        Ok(Self {
            workflow_id: wire.workflow_id,
            session_id: wire.session_id,
            correlation_id: wire.correlation_id,
            trace_id: wire.trace_id,
            state: wire.state,
        })
    }
}

/// A closed workflow-owned subsystem category, in canonical propagation order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CancellationTarget {
    Retrieval,
    TutorGeneration,
    Speech,
    Behavior,
    ToolExecution,
}

/// The cancellation capability declared for an active target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CancellationSemantics {
    Cancellable,
    NonCancellable,
}

/// The single planning outcome emitted for an active target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CancellationDirective {
    RequestCancellation,
    ReportNonCancellable,
}

macro_rules! versioned_enum_wire {
    ($ty:ty, $wire:ident, {$($variant:ident => $name:literal),+ $(,)?}) => {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct $wire { version: ProtocolVersion, kind: String }
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let kind = match self { $(Self::$variant => $name,)+ };
                $wire { version: CANCELLATION_PROPAGATION_V1, kind: kind.into() }.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let wire = $wire::deserialize(deserializer)?;
                if wire.version != CANCELLATION_PROPAGATION_V1 {
                    return Err(serde::de::Error::custom("unsupported cancellation propagation version"));
                }
                match wire.kind.as_str() { $($name => Ok(Self::$variant),)+ _ => Err(serde::de::Error::custom("unknown cancellation propagation variant")) }
            }
        }
    };
}

versioned_enum_wire!(CancellationTarget, CancellationTargetWire, {
    Retrieval => "retrieval", TutorGeneration => "tutor_generation", Speech => "speech",
    Behavior => "behavior", ToolExecution => "tool_execution"
});
versioned_enum_wire!(CancellationSemantics, CancellationSemanticsWire, {
    Cancellable => "cancellable", NonCancellable => "non_cancellable"
});
versioned_enum_wire!(CancellationDirective, CancellationDirectiveWire, {
    RequestCancellation => "request_cancellation", ReportNonCancellable => "report_non_cancellable"
});

/// One currently active workflow-owned target supplied to the planner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveCancellationTarget {
    version: ProtocolVersion,
    target: CancellationTarget,
    semantics: CancellationSemantics,
}

impl ActiveCancellationTarget {
    pub const fn new(target: CancellationTarget, semantics: CancellationSemantics) -> Self {
        Self {
            version: CANCELLATION_PROPAGATION_V1,
            target,
            semantics,
        }
    }

    /// Validates an explicitly versioned target received outside the wire decoder.
    pub fn try_new(
        version: ProtocolVersion,
        target: CancellationTarget,
        semantics: CancellationSemantics,
    ) -> Result<Self, CancellationPlanningError> {
        if version != CANCELLATION_PROPAGATION_V1 {
            return Err(CancellationPlanningError::UnsupportedVersion);
        }
        Ok(Self::new(target, semantics))
    }

    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub const fn target(&self) -> CancellationTarget {
        self.target
    }

    pub const fn semantics(&self) -> CancellationSemantics {
        self.semantics
    }
}

impl<'de> Deserialize<'de> for ActiveCancellationTarget {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            version: ProtocolVersion,
            target: CancellationTarget,
            semantics: CancellationSemantics,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.version != CANCELLATION_PROPAGATION_V1 {
            return Err(serde::de::Error::custom(
                CancellationPlanningError::UnsupportedVersion,
            ));
        }
        Ok(Self::new(wire.target, wire.semantics))
    }
}

/// One canonical, content-free cancellation directive.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedCancellationDirective {
    version: ProtocolVersion,
    target: CancellationTarget,
    directive: CancellationDirective,
}

impl PlannedCancellationDirective {
    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub const fn target(&self) -> CancellationTarget {
        self.target
    }

    pub const fn directive(&self) -> CancellationDirective {
        self.directive
    }
}

impl<'de> Deserialize<'de> for PlannedCancellationDirective {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            version: ProtocolVersion,
            target: CancellationTarget,
            directive: CancellationDirective,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.version != CANCELLATION_PROPAGATION_V1 {
            return Err(serde::de::Error::custom(
                CancellationPlanningError::UnsupportedVersion,
            ));
        }
        Ok(Self {
            version: wire.version,
            target: wire.target,
            directive: wire.directive,
        })
    }
}

/// A deterministic propagation plan preserving the cancelled workflow association.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCancellationPlan {
    version: ProtocolVersion,
    workflow_id: WorkflowId,
    session_id: SessionId,
    correlation_id: CorrelationId,
    trace_id: TraceId,
    directives: Vec<PlannedCancellationDirective>,
}

impl WorkflowCancellationPlan {
    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub const fn workflow_id(&self) -> WorkflowId {
        self.workflow_id
    }

    pub const fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub const fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }

    pub const fn trace_id(&self) -> TraceId {
        self.trace_id
    }

    pub fn directives(&self) -> &[PlannedCancellationDirective] {
        &self.directives
    }
}

impl<'de> Deserialize<'de> for WorkflowCancellationPlan {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            version: ProtocolVersion,
            workflow_id: WorkflowId,
            session_id: SessionId,
            correlation_id: CorrelationId,
            trace_id: TraceId,
            directives: Vec<PlannedCancellationDirective>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.version != CANCELLATION_PROPAGATION_V1 {
            return Err(serde::de::Error::custom(
                CancellationPlanningError::UnsupportedVersion,
            ));
        }
        if wire.directives.len() > CANCELLATION_TARGET_LIMIT {
            return Err(serde::de::Error::custom(
                CancellationPlanningError::TooManyTargets,
            ));
        }
        if wire
            .directives
            .windows(2)
            .any(|pair| pair[0].target >= pair[1].target)
        {
            return Err(serde::de::Error::custom(
                CancellationPlanningError::DuplicateTarget,
            ));
        }
        Ok(Self {
            version: wire.version,
            workflow_id: wire.workflow_id,
            session_id: wire.session_id,
            correlation_id: wire.correlation_id,
            trace_id: wire.trace_id,
            directives: wire.directives,
        })
    }
}

/// Closed, content-free propagation planning failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CancellationPlanningError {
    #[error("unsupported cancellation propagation version")]
    UnsupportedVersion,
    #[error("workflow is not cancelled")]
    WorkflowNotCancelled,
    #[error("cancellation propagation identity association mismatch")]
    AssociationMismatch,
    #[error("duplicate cancellation target")]
    DuplicateTarget,
    #[error("too many cancellation targets")]
    TooManyTargets,
}
versioned_enum_wire!(CancellationPlanningError, CancellationPlanningErrorWire, {
    UnsupportedVersion => "unsupported_version", WorkflowNotCancelled => "workflow_not_cancelled",
    AssociationMismatch => "association_mismatch", DuplicateTarget => "duplicate_target",
    TooManyTargets => "too_many_targets"
});

const CANCELLATION_TARGET_LIMIT: usize = 5;

/// Purely plans propagation for an already-cancelled workflow.
pub fn plan_workflow_cancellation(
    workflow: &InteractionWorkflow,
    workflow_id: WorkflowId,
    session_id: SessionId,
    correlation_id: CorrelationId,
    trace_id: TraceId,
    active_targets: &[ActiveCancellationTarget],
) -> Result<WorkflowCancellationPlan, CancellationPlanningError> {
    if workflow.state() != WorkflowState::Cancelled {
        return Err(CancellationPlanningError::WorkflowNotCancelled);
    }
    if (
        workflow.workflow_id(),
        workflow.session_id(),
        workflow.correlation_id(),
        workflow.trace_id(),
    ) != (workflow_id, session_id, correlation_id, trace_id)
    {
        return Err(CancellationPlanningError::AssociationMismatch);
    }
    if active_targets.len() > CANCELLATION_TARGET_LIMIT {
        return Err(CancellationPlanningError::TooManyTargets);
    }
    if active_targets
        .iter()
        .any(|target| target.version != CANCELLATION_PROPAGATION_V1)
    {
        return Err(CancellationPlanningError::UnsupportedVersion);
    }
    let mut targets = active_targets.to_vec();
    targets.sort_by_key(|target| target.target);
    if targets
        .windows(2)
        .any(|pair| pair[0].target == pair[1].target)
    {
        return Err(CancellationPlanningError::DuplicateTarget);
    }
    let directives = targets
        .into_iter()
        .map(|target| PlannedCancellationDirective {
            version: CANCELLATION_PROPAGATION_V1,
            target: target.target,
            directive: match target.semantics {
                CancellationSemantics::Cancellable => CancellationDirective::RequestCancellation,
                CancellationSemantics::NonCancellable => {
                    CancellationDirective::ReportNonCancellable
                }
            },
        })
        .collect();
    Ok(WorkflowCancellationPlan {
        version: CANCELLATION_PROPAGATION_V1,
        workflow_id,
        session_id,
        correlation_id,
        trace_id,
        directives,
    })
}
