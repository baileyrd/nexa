//! Deterministic, synchronous session and interaction-workflow lifecycle contracts.
#![forbid(unsafe_code)]

use nexa_domain::{CorrelationId, ProtocolVersion, SessionId, TraceId, WorkflowId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Wire protocol version for the lifecycle foundation accepted by ADR-0051.
pub const ORCHESTRATOR_LIFECYCLE_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

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
