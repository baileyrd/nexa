//! Headless binding for cooperative Retrieval cancellation.

use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_knowledge::RetrievalQuery;
use nexa_knowledge_runtime::{
    retrieve, RetrievalCancellation, RetrievalService, RetrievalServiceOutcome,
};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget, CancellationSemantics,
    CancellationTarget, InteractionWorkflow, WorkflowCancellationPlan, WorkflowState,
};
use nexa_orchestrator_runtime::{WorkflowCancellationExecution, WorkflowTaskGroup};
use std::sync::{Arc, Mutex};

/// Closed, content-free failures at the headless Retrieval boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalCancellationCompositionError {
    InvalidWorkflow,
    AssociationMismatch,
    InvalidQuery,
    ServiceFailure,
    RuntimeFailure,
    ConflictingExecution,
}
impl std::fmt::Display for RetrievalCancellationCompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorkflow => "invalid cancelled workflow",
            Self::AssociationMismatch => "retrieval cancellation association mismatch",
            Self::InvalidQuery => "invalid retrieval query",
            Self::ServiceFailure => "retrieval cancellation service failure",
            Self::RuntimeFailure => "retrieval cancellation runtime failure",
            Self::ConflictingExecution => "conflicting retrieval cancellation execution",
        })
    }
}
impl std::error::Error for RetrievalCancellationCompositionError {}

/// Immutable proof that the exact service future observed cancellation and joined.
///
/// This does not prove that an external provider, database, vector engine, or network
/// request stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalCancellationEvidence {
    runtime: WorkflowCancellationExecution,
    cancellation: RetrievalCancellation,
}
impl RetrievalCancellationEvidence {
    pub const fn runtime(&self) -> &WorkflowCancellationExecution {
        &self.runtime
    }
    pub const fn cancellation(&self) -> &RetrievalCancellation {
        &self.cancellation
    }
}

enum ExecutionState {
    Ready,
    Running,
    Succeeded(Box<RetrievalCancellationEvidence>),
    Failed,
}

/// Owns one supplied service and one exact Retrieval cancellation operation.
pub struct RetrievalCancellationComposition<S> {
    workflow: InteractionWorkflow,
    query: RetrievalQuery,
    plan: WorkflowCancellationPlan,
    service: Arc<S>,
    terminal: ExecutionState,
    #[cfg(test)]
    fault: Option<TestFault>,
    #[cfg(test)]
    barrier: Option<TestBarrier>,
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum TestFault {
    Join,
    Coverage,
}
#[cfg(test)]
struct TestBarrier {
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
}

impl<S: RetrievalService + 'static> RetrievalCancellationComposition<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workflow: InteractionWorkflow,
        workflow_id: WorkflowId,
        session_id: SessionId,
        correlation_id: CorrelationId,
        trace_id: TraceId,
        query: RetrievalQuery,
        service: S,
    ) -> Result<Self, RetrievalCancellationCompositionError> {
        if workflow.state() != WorkflowState::Cancelled {
            return Err(RetrievalCancellationCompositionError::InvalidWorkflow);
        }
        if (
            workflow.workflow_id(),
            workflow.session_id(),
            workflow.correlation_id(),
            workflow.trace_id(),
        ) != (workflow_id, session_id, correlation_id, trace_id)
        {
            return Err(RetrievalCancellationCompositionError::AssociationMismatch);
        }
        query
            .validate()
            .map_err(|_| RetrievalCancellationCompositionError::InvalidQuery)?;
        let plan = plan_workflow_cancellation(
            &workflow,
            workflow_id,
            session_id,
            correlation_id,
            trace_id,
            &[ActiveCancellationTarget::new(
                CancellationTarget::Retrieval,
                CancellationSemantics::Cancellable,
            )],
        )
        .map_err(|_| RetrievalCancellationCompositionError::InvalidWorkflow)?;
        Ok(Self {
            workflow,
            query,
            plan,
            service: Arc::new(service),
            terminal: ExecutionState::Ready,
            #[cfg(test)]
            fault: None,
            #[cfg(test)]
            barrier: None,
        })
    }

    pub const fn plan(&self) -> &WorkflowCancellationPlan {
        &self.plan
    }

    /// Bounded read-only inspection of deterministic service evidence.
    pub fn inspect_service<R>(&self, inspect: impl FnOnce(&S) -> R) -> R {
        inspect(&self.service)
    }

    pub async fn execute(
        &mut self,
        query: RetrievalQuery,
    ) -> Result<RetrievalCancellationEvidence, RetrievalCancellationCompositionError> {
        if query != self.query {
            return Err(RetrievalCancellationCompositionError::ConflictingExecution);
        }
        match &self.terminal {
            ExecutionState::Succeeded(value) => return Ok((**value).clone()),
            ExecutionState::Running | ExecutionState::Failed => {
                return Err(RetrievalCancellationCompositionError::RuntimeFailure)
            }
            ExecutionState::Ready => {}
        }
        // Terminalize before spawn: caller drop aborts owned work and retry cannot duplicate it.
        self.terminal = ExecutionState::Running;
        let mut tasks = WorkflowTaskGroup::new(self.workflow);
        let service = Arc::clone(&self.service);
        let exact = self.query.clone();
        let observed = Arc::new(Mutex::new(None));
        let task_observed = Arc::clone(&observed);
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        #[cfg(test)]
        let fault = self.fault;
        tasks
            .spawn_for_target(CancellationTarget::Retrieval, move |token| async move {
                #[cfg(test)]
                let task_token = token.clone();
                let mut future = Box::pin(retrieve(&*service, exact, token));
                let mut started_tx = Some(started_tx);
                #[cfg(test)]
                if matches!(fault, Some(TestFault::Join)) {
                    std::future::poll_fn(|cx| {
                        let _ = std::future::Future::poll(future.as_mut(), cx);
                        if let Some(started_tx) = started_tx.take() {
                            let _ = started_tx.send(());
                        }
                        std::task::Poll::Ready(())
                    })
                    .await;
                    task_token.cancelled().await;
                    drop(future);
                    panic!("injected join failure during cancellation");
                }
                let outcome = std::future::poll_fn(|cx| {
                    let poll = std::future::Future::poll(future.as_mut(), cx);
                    if let Some(started_tx) = started_tx.take() {
                        let _ = started_tx.send(());
                    }
                    poll
                })
                .await;
                if let Ok(mut slot) = task_observed.lock() {
                    *slot = Some(outcome);
                }
            })
            .map_err(|_| {
                self.terminal = ExecutionState::Failed;
                RetrievalCancellationCompositionError::RuntimeFailure
            })?;
        if started_rx.await.is_err() {
            self.terminal = ExecutionState::Failed;
            return Err(RetrievalCancellationCompositionError::RuntimeFailure);
        }
        #[cfg(test)]
        if let Some(barrier) = self.barrier.take() {
            let _ = barrier.reached.send(());
            if barrier.release.await.is_err() {
                self.terminal = ExecutionState::Failed;
                return Err(RetrievalCancellationCompositionError::RuntimeFailure);
            }
        }
        #[cfg(test)]
        if matches!(self.fault, Some(TestFault::Coverage)) {
            tasks
                .spawn_for_target(CancellationTarget::Behavior, |_| async {})
                .unwrap();
        }
        let runtime = tasks
            .execute_cancellation_plan(&self.plan)
            .await
            .map_err(|_| {
                self.terminal = ExecutionState::Failed;
                RetrievalCancellationCompositionError::RuntimeFailure
            })?;
        let cancellation = match observed.lock().ok().and_then(|mut slot| slot.take()) {
            Some(Ok(RetrievalServiceOutcome::Cancelled(value))) => value,
            _ => {
                self.terminal = ExecutionState::Failed;
                return Err(RetrievalCancellationCompositionError::ServiceFailure);
            }
        };
        let evidence = RetrievalCancellationEvidence {
            runtime,
            cancellation,
        };
        self.terminal = ExecutionState::Succeeded(Box::new(evidence.clone()));
        Ok(evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_domain::{RetrievalQueryId, RetrievalResultId};
    use nexa_knowledge::{Audience, RetrievalFilters, RetrievalResult, LEXICAL_RETRIEVAL_V1, V1};
    use nexa_knowledge_runtime::{
        RetrievalFuture, ScriptedRetrievalOutcome as Outcome, ScriptedRetrievalService as Service,
    };
    use nexa_orchestrator::CancellationDirective;
    use nexa_orchestrator_runtime::CancellationTargetExecutionOutcome;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    fn id<T>(n: u128, make: impl Fn(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
        make(Uuid::from_u128(n)).unwrap()
    }
    fn ids() -> (WorkflowId, SessionId, CorrelationId, TraceId) {
        (
            id(1, WorkflowId::new),
            id(2, SessionId::new),
            id(3, CorrelationId::new),
            id(4, TraceId::new),
        )
    }
    fn workflow(cancelled: bool) -> InteractionWorkflow {
        let (w, s, c, t) = ids();
        let x = InteractionWorkflow::new(w, s, c, t);
        if cancelled {
            x.cancel().unwrap()
        } else {
            x
        }
    }
    fn query(seed: u128, text: &str) -> RetrievalQuery {
        RetrievalQuery {
            contract_version: V1,
            retrieval_policy_version: LEXICAL_RETRIEVAL_V1,
            query_id: id(seed, RetrievalQueryId::new),
            result_id: id(seed + 1, RetrievalResultId::new),
            text: text.into(),
            filters: RetrievalFilters {
                audience: Audience::StudentLearning,
                course_id: None,
                lesson_id: None,
            },
            maximum_results: 1,
        }
    }
    fn result(q: &RetrievalQuery) -> RetrievalResult {
        RetrievalResult {
            contract_version: V1,
            retrieval_policy_version: LEXICAL_RETRIEVAL_V1,
            query_id: q.query_id,
            result_id: q.result_id,
            candidates: vec![],
            exclusions: vec![],
        }
    }
    fn composition(
        outcomes: impl IntoIterator<Item = Outcome>,
    ) -> RetrievalCancellationComposition<Service> {
        let (w, s, c, t) = ids();
        RetrievalCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            query(10, "private query"),
            Service::new(outcomes),
        )
        .unwrap()
    }

    #[test]
    fn preflight_is_side_effect_free_and_plan_is_exact() {
        let c = composition([Outcome::WaitForCancellation]);
        assert_eq!(c.plan().directives().len(), 1);
        assert_eq!(
            c.plan().directives()[0].target(),
            CancellationTarget::Retrieval
        );
        assert_eq!(
            c.plan().directives()[0].directive(),
            CancellationDirective::RequestCancellation
        );
        let (w, s, corr, t) = ids();
        for changed in 0..6 {
            let service = Service::new([Outcome::WaitForCancellation]);
            let observable = service.clone();
            let (mut wi, mut si, mut ci, mut ti) = (w, s, corr, t);
            let mut wf = workflow(true);
            let mut q = query(10, "private query");
            match changed {
                0 => wf = workflow(false),
                1 => wi = id(30, WorkflowId::new),
                2 => si = id(30, SessionId::new),
                3 => ci = id(30, CorrelationId::new),
                4 => ti = id(30, TraceId::new),
                _ => q.text.clear(),
            }
            assert!(RetrievalCancellationComposition::new(wf, wi, si, ci, ti, q, service).is_err());
            assert_eq!(
                (
                    observable.received_queries().len(),
                    observable.consumed_outcome_count(),
                    observable.active_operation_count()
                ),
                (0, 0, 0)
            );
        }
    }

    #[tokio::test]
    async fn waiting_service_is_started_cancelled_joined_and_exactly_consumed() {
        let mut c = composition([Outcome::WaitForCancellation]);
        let observable = c.inspect_service(Clone::clone);
        let (reached_tx, reached_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        c.barrier = Some(TestBarrier {
            reached: reached_tx,
            release: release_rx,
        });
        let mut execution = Box::pin(c.execute(query(10, "private query")));
        tokio::select! {
            result = &mut execution => panic!("execution completed before cancellation barrier: {result:?}"),
            result = reached_rx => result.expect("cancellation barrier must be reached"),
        }
        assert_eq!(
            (
                observable.received_queries(),
                observable.consumed_outcome_count(),
                observable.remaining_outcome_count(),
                observable.active_operation_count()
            ),
            (vec![query(10, "private query")], 1, 0, 1)
        );
        release_tx.send(()).unwrap();
        let evidence = execution.as_mut().await.unwrap();
        drop(execution);
        assert_eq!(
            evidence.cancellation(),
            &RetrievalCancellation::from_query(&query(10, "private query")).unwrap()
        );
        assert_eq!(
            evidence.runtime().target_outcomes()[0].target(),
            CancellationTarget::Retrieval
        );
        assert_eq!(
            evidence.runtime().target_outcomes()[0].outcome(),
            CancellationTargetExecutionOutcome::Stopped
        );
        assert_eq!(
            c.inspect_service(|s| (
                s.received_queries(),
                s.consumed_outcome_count(),
                s.remaining_outcome_count(),
                s.active_operation_count()
            )),
            (vec![query(10, "private query")], 1, 0, 0)
        );
    }

    #[tokio::test]
    async fn success_failures_exhaustion_and_association_mismatch_fail_closed() {
        let q = query(10, "private query");
        let mut wrong = result(&q);
        wrong.result_id = id(99, RetrievalResultId::new);
        for outcomes in [
            vec![Outcome::Success(result(&q))],
            vec![Outcome::DependencyFailure],
            vec![],
            vec![Outcome::Success(wrong)],
        ] {
            let mut c = composition(outcomes);
            assert_eq!(
                c.execute(q.clone()).await,
                Err(RetrievalCancellationCompositionError::ServiceFailure)
            );
            let before =
                c.inspect_service(|s| (s.received_queries().len(), s.consumed_outcome_count()));
            assert_eq!(
                c.execute(q.clone()).await,
                Err(RetrievalCancellationCompositionError::RuntimeFailure)
            );
            assert_eq!(
                c.inspect_service(|s| (s.received_queries().len(), s.consumed_outcome_count())),
                before
            );
        }
    }

    #[tokio::test]
    async fn repeat_is_idempotent_and_conflict_does_not_duplicate_work() {
        let mut c = composition([Outcome::WaitForCancellation]);
        let first = c.execute(query(10, "private query")).await.unwrap();
        assert_eq!(c.execute(query(10, "private query")).await.unwrap(), first);
        assert_eq!(
            c.execute(query(20, "other secret")).await,
            Err(RetrievalCancellationCompositionError::ConflictingExecution)
        );
        assert_eq!(
            c.inspect_service(|s| (s.received_queries().len(), s.consumed_outcome_count())),
            (1, 1)
        );
    }

    struct StartedService {
        inner: Service,
        started: std::sync::mpsc::Sender<()>,
    }

    struct ReassociatedCancellationService(RetrievalCancellation);
    impl RetrievalService for ReassociatedCancellationService {
        fn retrieve(
            &self,
            _query: RetrievalQuery,
            _token: CancellationToken,
        ) -> RetrievalFuture<'_> {
            Box::pin(async move { RetrievalServiceOutcome::Cancelled(self.0) })
        }
    }

    #[tokio::test]
    async fn reassociated_cancellation_evidence_fails_closed() {
        let (w, s, c, t) = ids();
        let mismatched = RetrievalCancellation::from_query(&query(40, "other secret")).unwrap();
        let mut composition = RetrievalCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            query(10, "private query"),
            ReassociatedCancellationService(mismatched),
        )
        .unwrap();
        assert_eq!(
            composition.execute(query(10, "private query")).await,
            Err(RetrievalCancellationCompositionError::ServiceFailure)
        );
    }
    impl RetrievalService for StartedService {
        fn retrieve(&self, q: RetrievalQuery, t: CancellationToken) -> RetrievalFuture<'_> {
            let f = self.inner.retrieve(q, t);
            let started = self.started.clone();
            Box::pin(async move {
                let _ = started.send(());
                f.await
            })
        }
    }
    async fn receive(rx: &std::sync::mpsc::Receiver<()>) {
        loop {
            match rx.try_recv() {
                Ok(()) => return,
                Err(std::sync::mpsc::TryRecvError::Empty) => tokio::task::yield_now().await,
                Err(_) => panic!("probe disconnected"),
            }
        }
    }

    #[tokio::test]
    async fn dropping_started_caller_aborts_service_and_forbids_retry() {
        let (tx, rx) = std::sync::mpsc::channel();
        let service = StartedService {
            inner: Service::new([Outcome::WaitForCancellation]),
            started: tx,
        };
        let observable = service.inner.clone();
        let (w, s, c, t) = ids();
        let mut composition = RetrievalCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            query(10, "private query"),
            service,
        )
        .unwrap();
        let mut future = Box::pin(composition.execute(query(10, "private query")));
        std::future::poll_fn(|cx| {
            let _ = std::future::Future::poll(future.as_mut(), cx);
            std::task::Poll::Ready(())
        })
        .await;
        receive(&rx).await;
        assert_eq!(observable.active_operation_count(), 1);
        drop(future);
        for _ in 0..100 {
            if observable.active_operation_count() == 0 {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(observable.active_operation_count(), 0);
        assert_eq!(
            composition.execute(query(10, "private query")).await,
            Err(RetrievalCancellationCompositionError::RuntimeFailure)
        );
    }

    #[tokio::test]
    async fn runtime_failures_are_closed_and_terminal() {
        for fault in [TestFault::Join, TestFault::Coverage] {
            let mut c = composition([Outcome::WaitForCancellation]);
            let observable = c.inspect_service(Clone::clone);
            c.fault = Some(fault);
            assert_eq!(
                c.execute(query(10, "private query")).await,
                Err(RetrievalCancellationCompositionError::RuntimeFailure)
            );
            if matches!(fault, TestFault::Join) {
                assert_eq!(
                    (
                        observable.received_queries(),
                        observable.consumed_outcome_count(),
                        observable.remaining_outcome_count(),
                        observable.active_operation_count()
                    ),
                    (vec![query(10, "private query")], 1, 0, 0)
                );
            }
            assert_eq!(
                c.execute(query(10, "private query")).await,
                Err(RetrievalCancellationCompositionError::RuntimeFailure)
            );
            if matches!(fault, TestFault::Join) {
                assert_eq!(
                    (
                        observable.received_queries().len(),
                        observable.consumed_outcome_count(),
                        observable.active_operation_count()
                    ),
                    (1, 1, 0)
                );
            }
        }
    }

    #[tokio::test]
    async fn public_surfaces_redact_content() {
        let q = query(10, "ultra-private-query-text");
        assert!(!format!("{q:?}").contains("ultra-private-query-text"));
        let e = RetrievalCancellation::from_query(&q).unwrap();
        assert!(!format!("{e:?} {e}").contains("ultra-private-query-text"));
        let (w, s, c, t) = ids();
        let mut composition = RetrievalCancellationComposition::new(
            workflow(true),
            w,
            s,
            c,
            t,
            q.clone(),
            Service::new([Outcome::WaitForCancellation]),
        )
        .unwrap();
        let evidence = composition.execute(q).await.unwrap();
        let public_debug = format!("{evidence:?}");
        assert!(!public_debug.contains("ultra-private-query-text"));
        assert!(!public_debug.contains("StudentLearning"));
        assert!(!public_debug.contains(&LEXICAL_RETRIEVAL_V1.to_string()));

        for (error, debug, display) in [
            (
                RetrievalCancellationCompositionError::InvalidWorkflow,
                "InvalidWorkflow",
                "invalid cancelled workflow",
            ),
            (
                RetrievalCancellationCompositionError::AssociationMismatch,
                "AssociationMismatch",
                "retrieval cancellation association mismatch",
            ),
            (
                RetrievalCancellationCompositionError::InvalidQuery,
                "InvalidQuery",
                "invalid retrieval query",
            ),
            (
                RetrievalCancellationCompositionError::ServiceFailure,
                "ServiceFailure",
                "retrieval cancellation service failure",
            ),
            (
                RetrievalCancellationCompositionError::RuntimeFailure,
                "RuntimeFailure",
                "retrieval cancellation runtime failure",
            ),
            (
                RetrievalCancellationCompositionError::ConflictingExecution,
                "ConflictingExecution",
                "conflicting retrieval cancellation execution",
            ),
        ] {
            assert_eq!(format!("{error:?}"), debug);
            assert_eq!(error.to_string(), display);
            assert!(!format!("{error:?} {error}").contains("ultra-private-query-text"));
        }
    }
}
