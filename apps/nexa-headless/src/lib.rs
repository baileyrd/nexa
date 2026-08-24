//! Minimal headless composition of workflow-owned Behavior cancellation.
#![forbid(unsafe_code)]

use nexa_avatar::{AvatarPort, AvatarReport, AvatarRequest};
use nexa_domain::{CorrelationId, MessageId, SessionId, TraceId, WorkflowId};
use nexa_nbp::{AvatarCapability, BehaviorCancel, RuntimeStatus};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget, CancellationSemantics,
    CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan,
};
use nexa_orchestrator_runtime::{
    WorkflowCancellationExecution, WorkflowTaskGroup, WorkflowTaskGroupError,
};
use std::sync::{Arc, Mutex};

/// Closed, content-free failures at the application composition boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BehaviorCancellationError {
    InvalidWorkflow,
    AssociationMismatch,
    CapabilityUnavailable,
    InvalidPreview,
    RuntimeFailure,
    ConflictingExecution,
}

impl std::fmt::Display for BehaviorCancellationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorkflow => "invalid cancelled workflow",
            Self::AssociationMismatch => "behavior cancellation association mismatch",
            Self::CapabilityUnavailable => "avatar cancellation capability unavailable",
            Self::InvalidPreview => "invalid avatar cancellation preview",
            Self::RuntimeFailure => "behavior cancellation runtime failure",
            Self::ConflictingExecution => "conflicting behavior cancellation execution",
        })
    }
}

impl std::error::Error for BehaviorCancellationError {}

#[derive(Clone, Debug, PartialEq)]
pub struct AvatarCancellationEvidence {
    request: AvatarRequest,
    report: AvatarReport,
}

impl AvatarCancellationEvidence {
    pub const fn request(&self) -> &AvatarRequest {
        &self.request
    }
    pub const fn report(&self) -> &AvatarReport {
        &self.report
    }
    pub const fn cancellation_request_count(&self) -> usize {
        1
    }
    pub const fn submit_request_count(&self) -> usize {
        0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BehaviorCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    avatar: AvatarCancellationEvidence,
}

impl BehaviorCancellationEvidence {
    pub const fn runtime(&self) -> &WorkflowCancellationExecution {
        &self.runtime
    }
    pub const fn avatar(&self) -> &AvatarCancellationEvidence {
        &self.avatar
    }
}

/// Owns one adapter and one exact Behavior-cancellation operation.
pub struct BehaviorCancellationComposition<A> {
    workflow: InteractionWorkflow,
    request: AvatarRequest,
    plan: WorkflowCancellationPlan,
    adapter: Arc<Mutex<A>>,
    terminal: Option<BehaviorCancellationEvidence>,
}

impl<A: AvatarPort + Send + 'static> BehaviorCancellationComposition<A> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow: InteractionWorkflow,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
        message_id: MessageId,
        cancellation: BehaviorCancel,
        adapter: A,
    ) -> Result<Self, BehaviorCancellationError> {
        if workflow.state() != nexa_orchestrator::WorkflowState::Cancelled {
            return Err(BehaviorCancellationError::InvalidWorkflow);
        }
        if (
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
        ) != (workflow_id, session_id, correlation_id, trace_id)
        {
            return Err(BehaviorCancellationError::AssociationMismatch);
        }
        if !adapter
            .capabilities()
            .supports(AvatarCapability::Cancellation)
        {
            return Err(BehaviorCancellationError::CapabilityUnavailable);
        }
        let request = AvatarRequest::Cancel {
            message_id,
            cancellation,
        };
        let preview = adapter.preview(&request);
        let behavior_id = match &request {
            AvatarRequest::Cancel { cancellation, .. } => cancellation.behavior_id,
            AvatarRequest::Submit { .. } => unreachable!(),
        };
        if preview.message_id != message_id
            || preview.behavior_id != behavior_id
            || preview.terminal_status() != RuntimeStatus::Cancelled
        {
            return Err(BehaviorCancellationError::InvalidPreview);
        }
        let plan = plan_workflow_cancellation(
            &workflow,
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            &[ActiveCancellationTarget::new(
                CancellationTarget::Behavior,
                CancellationSemantics::Cancellable,
            )],
        )
        .map_err(|_| BehaviorCancellationError::InvalidWorkflow)?;
        Ok(Self {
            workflow,
            request,
            plan,
            adapter: Arc::new(Mutex::new(adapter)),
            terminal: None,
        })
    }

    pub const fn plan(&self) -> &WorkflowCancellationPlan {
        &self.plan
    }

    pub async fn execute(
        &mut self,
        message_id: MessageId,
        cancellation: BehaviorCancel,
    ) -> Result<BehaviorCancellationEvidence, BehaviorCancellationError> {
        let requested = AvatarRequest::Cancel {
            message_id,
            cancellation,
        };
        if requested != self.request {
            return Err(BehaviorCancellationError::ConflictingExecution);
        }
        if let Some(evidence) = &self.terminal {
            return Ok(evidence.clone());
        }

        let mut tasks = WorkflowTaskGroup::new(self.workflow);
        let adapter = Arc::clone(&self.adapter);
        let request = self.request.clone();
        let report = Arc::new(Mutex::new(None));
        let task_report = Arc::clone(&report);
        tasks
            .spawn_for_target(CancellationTarget::Behavior, move |token| async move {
                token.cancelled().await;
                let result = adapter
                    .lock()
                    .expect("avatar adapter mutex poisoned")
                    .handle(request);
                *task_report.lock().expect("avatar report mutex poisoned") = Some(result);
            })
            .map_err(map_runtime_error)?;
        let runtime = tasks
            .execute_cancellation_plan(&self.plan)
            .await
            .map_err(map_runtime_error)?;
        let avatar_report = report
            .lock()
            .expect("avatar report mutex poisoned")
            .clone()
            .ok_or(BehaviorCancellationError::RuntimeFailure)?;
        let evidence = BehaviorCancellationEvidence {
            runtime,
            avatar: AvatarCancellationEvidence {
                request: self.request.clone(),
                report: avatar_report,
            },
        };
        self.terminal = Some(evidence.clone());
        Ok(evidence)
    }
}

fn map_runtime_error(error: WorkflowTaskGroupError) -> BehaviorCancellationError {
    match error {
        WorkflowTaskGroupError::ConflictingCompletion => {
            BehaviorCancellationError::ConflictingExecution
        }
        _ => BehaviorCancellationError::RuntimeFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_avatar::{AvatarCapabilities, FakeAvatarAdapter};
    use nexa_domain::{BehaviorId, SemanticKey};
    use nexa_nbp::CancellationMode;
    use nexa_orchestrator::CancellationDirective;
    use nexa_orchestrator_runtime::CancellationTargetExecutionOutcome;
    use std::str::FromStr;
    use uuid::Uuid;

    fn ids() -> (WorkflowId, SessionId, CorrelationId, TraceId) {
        (
            WorkflowId::new(Uuid::from_u128(1)).unwrap(),
            SessionId::new(Uuid::from_u128(2)).unwrap(),
            CorrelationId::new(Uuid::from_u128(3)).unwrap(),
            TraceId::new(Uuid::from_u128(4)).unwrap(),
        )
    }

    fn message_id() -> MessageId {
        MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap()
    }

    fn cancellation() -> BehaviorCancel {
        BehaviorCancel {
            behavior_id: BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap(),
            reason: "new turn".into(),
            transition: CancellationMode::Graceful,
        }
    }

    fn workflow(cancelled: bool) -> InteractionWorkflow {
        let (workflow, session, correlation, trace) = ids();
        let value = InteractionWorkflow::new(workflow, session, correlation, trace);
        if cancelled {
            value.cancel().unwrap()
        } else {
            value
        }
    }

    fn adapter(capable: bool) -> FakeAvatarAdapter {
        FakeAvatarAdapter::new(
            SemanticKey::new("headless-avatar").unwrap(),
            AvatarCapabilities::new(capable.then_some(AvatarCapability::Cancellation)),
        )
    }

    fn composition() -> BehaviorCancellationComposition<FakeAvatarAdapter> {
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        BehaviorCancellationComposition::new(
            workflow(true),
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            message_id(),
            cancellation(),
            adapter(true),
        )
        .unwrap()
    }

    #[test]
    fn canonical_behavior_plan_executes_exactly_once_and_preserves_identity() {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let mut composition = composition();
                assert_eq!(composition.plan().directives().len(), 1);
                assert_eq!(
                    composition.plan().directives()[0].target(),
                    CancellationTarget::Behavior
                );
                assert_eq!(
                    composition.plan().directives()[0].directive(),
                    CancellationDirective::RequestCancellation
                );

                let evidence = composition
                    .execute(message_id(), cancellation())
                    .await
                    .unwrap();
                assert_eq!(evidence.runtime().workflow(), workflow(true));
                assert_eq!(evidence.runtime().target_outcomes().len(), 1);
                assert_eq!(
                    evidence.runtime().target_outcomes()[0].outcome(),
                    CancellationTargetExecutionOutcome::Stopped
                );
                assert_eq!(evidence.avatar().cancellation_request_count(), 1);
                assert_eq!(evidence.avatar().submit_request_count(), 0);
                assert_eq!(evidence.avatar().report().message_id, message_id());
                assert_eq!(
                    evidence.avatar().report().behavior_id,
                    cancellation().behavior_id
                );
                assert_eq!(
                    evidence.avatar().report().terminal_status(),
                    RuntimeStatus::Cancelled
                );

                let repeat = composition
                    .execute(message_id(), cancellation())
                    .await
                    .unwrap();
                assert_eq!(repeat, evidence);
                let mut conflict = cancellation();
                conflict.reason = "different".into();
                assert_eq!(
                    composition.execute(message_id(), conflict).await,
                    Err(BehaviorCancellationError::ConflictingExecution)
                );
            });
    }

    #[test]
    fn workflow_capability_and_all_associations_fail_during_preflight() {
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        let build = |workflow, w, s, c, t, capable| {
            BehaviorCancellationComposition::new(
                workflow,
                w,
                s,
                c,
                t,
                message_id(),
                cancellation(),
                adapter(capable),
            )
            .map(|_| ())
        };
        assert_eq!(
            build(
                workflow(false),
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::InvalidWorkflow)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
                false
            ),
            Err(BehaviorCancellationError::CapabilityUnavailable)
        );
        let wrong = Uuid::from_u128(99);
        assert_eq!(
            build(
                workflow(true),
                WorkflowId::new(wrong).unwrap(),
                session_id,
                correlation_id,
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                SessionId::new(wrong).unwrap(),
                correlation_id,
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                session_id,
                CorrelationId::new(wrong).unwrap(),
                trace_id,
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
        assert_eq!(
            build(
                workflow(true),
                workflow_id,
                session_id,
                correlation_id,
                TraceId::new(wrong).unwrap(),
                true
            ),
            Err(BehaviorCancellationError::AssociationMismatch)
        );
    }

    #[test]
    fn errors_have_exact_content_free_debug_and_display() {
        let cases = [
            (
                BehaviorCancellationError::InvalidWorkflow,
                "InvalidWorkflow",
                "invalid cancelled workflow",
            ),
            (
                BehaviorCancellationError::AssociationMismatch,
                "AssociationMismatch",
                "behavior cancellation association mismatch",
            ),
            (
                BehaviorCancellationError::CapabilityUnavailable,
                "CapabilityUnavailable",
                "avatar cancellation capability unavailable",
            ),
            (
                BehaviorCancellationError::InvalidPreview,
                "InvalidPreview",
                "invalid avatar cancellation preview",
            ),
            (
                BehaviorCancellationError::RuntimeFailure,
                "RuntimeFailure",
                "behavior cancellation runtime failure",
            ),
            (
                BehaviorCancellationError::ConflictingExecution,
                "ConflictingExecution",
                "conflicting behavior cancellation execution",
            ),
        ];
        for (error, debug, display) in cases {
            assert_eq!(format!("{error:?}"), debug);
            assert_eq!(error.to_string(), display);
        }
    }

    struct InvalidPreviewAdapter {
        preview: AvatarReport,
    }

    impl AvatarPort for InvalidPreviewAdapter {
        fn capabilities(&self) -> AvatarCapabilities {
            AvatarCapabilities::new([AvatarCapability::Cancellation])
        }
        fn preview(&self, _: &AvatarRequest) -> AvatarReport {
            self.preview.clone()
        }
        fn submit(&mut self, _: MessageId, _: nexa_nbp::BehaviorCommand) -> AvatarReport {
            unreachable!()
        }
        fn cancel(&mut self, _: MessageId, _: BehaviorCancel) -> AvatarReport {
            unreachable!()
        }
    }

    #[test]
    fn invalid_preview_identity_or_status_fails_before_adapter_mutation() {
        let (workflow_id, session_id, correlation_id, trace_id) = ids();
        let previews = [
            AvatarReport::cancelled(
                MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap(),
                cancellation().behavior_id,
            ),
            AvatarReport::cancelled(
                message_id(),
                BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249099").unwrap(),
            ),
            AvatarReport::rejected(
                message_id(),
                cancellation().behavior_id,
                SemanticKey::new("invalid.preview").unwrap(),
                "must not escape".into(),
            ),
        ];
        for preview in previews {
            assert!(matches!(
                BehaviorCancellationComposition::new(
                    workflow(true),
                    workflow_id,
                    session_id,
                    correlation_id,
                    trace_id,
                    message_id(),
                    cancellation(),
                    InvalidPreviewAdapter { preview },
                ),
                Err(BehaviorCancellationError::InvalidPreview)
            ));
        }
    }
}
