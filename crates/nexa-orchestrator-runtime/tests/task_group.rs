use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_orchestrator::InteractionWorkflow;
use nexa_orchestrator_runtime::{
    WorkflowTaskCompletionKind, WorkflowTaskGroup, WorkflowTaskGroupError, WorkflowTaskGroupState,
};
use std::future::Future;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::task::{Context, Poll, Waker};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

fn workflow() -> InteractionWorkflow {
    InteractionWorkflow::new(
        WorkflowId::new(Uuid::from_u128(1)).unwrap(),
        SessionId::new(Uuid::from_u128(2)).unwrap(),
        CorrelationId::new(Uuid::from_u128(3)).unwrap(),
        TraceId::new(Uuid::from_u128(4)).unwrap(),
    )
}

#[tokio::test]
async fn cancellation_preserves_all_identities_and_waits_for_cooperative_task() {
    let expected_workflow_id = WorkflowId::new(Uuid::from_u128(1)).unwrap();
    let expected_session_id = SessionId::new(Uuid::from_u128(2)).unwrap();
    let expected_correlation_id = CorrelationId::new(Uuid::from_u128(3)).unwrap();
    let expected_trace_id = TraceId::new(Uuid::from_u128(4)).unwrap();
    let expected = InteractionWorkflow::new(
        expected_workflow_id,
        expected_session_id,
        expected_correlation_id,
        expected_trace_id,
    );
    let mut group = WorkflowTaskGroup::new(expected);
    let (stopped_tx, stopped_rx) = oneshot::channel();
    let spawn_result: Result<(), WorkflowTaskGroupError> = group.spawn(|token| async move {
        token.cancelled().await;
        stopped_tx.send(()).unwrap();
    });
    spawn_result.unwrap();

    let evidence = group.cancel_and_wait().await.unwrap();
    assert_eq!(stopped_rx.await, Ok(()));
    assert_eq!(evidence.workflow().workflow_id(), expected_workflow_id);
    assert_eq!(evidence.workflow().session_id(), expected_session_id);
    assert_eq!(
        evidence.workflow().correlation_id(),
        expected_correlation_id
    );
    assert_eq!(evidence.workflow().trace_id(), expected_trace_id);
    assert_eq!(evidence.workflow(), expected);
    assert_eq!(evidence.kind(), WorkflowTaskCompletionKind::Cancelled);
    assert_eq!(group.task_count(), 0);
}

#[tokio::test]
async fn every_owned_task_stops_and_repeat_cancellation_is_idempotent() {
    let mut group = WorkflowTaskGroup::new(workflow());
    let stops = Arc::new(AtomicUsize::new(0));
    for _ in 0..3 {
        let stops = Arc::clone(&stops);
        group
            .spawn(move |token| async move {
                token.cancelled().await;
                stops.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
    }
    let first = group.cancel_and_wait().await.unwrap();
    let second = group.cancel_and_wait().await.unwrap();
    assert_eq!(first, second);
    assert_eq!(stops.load(Ordering::SeqCst), 3);
    assert!(group.is_cancellation_requested());
    assert_eq!(group.state(), WorkflowTaskGroupState::Cancelled);
}

#[tokio::test]
async fn rejects_spawning_after_cancellation_or_drain_begins() {
    let mut cancelling = WorkflowTaskGroup::new(workflow());
    cancelling.spawn(|_| std::future::pending::<()>()).unwrap();
    let mut cancellation = Box::pin(cancelling.cancel_and_wait());
    let mut context = Context::from_waker(Waker::noop());
    assert_eq!(cancellation.as_mut().poll(&mut context), Poll::Pending);
    drop(cancellation);
    assert_eq!(cancelling.state(), WorkflowTaskGroupState::Cancelling);
    assert_eq!(
        cancelling.spawn(|_| async {}),
        Err(WorkflowTaskGroupError::NotAcceptingTasks)
    );

    let mut draining = WorkflowTaskGroup::new(workflow());
    draining.spawn(|_| std::future::pending::<()>()).unwrap();
    let mut drain = Box::pin(draining.drain());
    let mut context = Context::from_waker(Waker::noop());
    assert_eq!(drain.as_mut().poll(&mut context), Poll::Pending);
    drop(drain);
    assert_eq!(draining.state(), WorkflowTaskGroupState::Draining);
    assert_eq!(
        draining.spawn(|_| async {}),
        Err(WorkflowTaskGroupError::NotAcceptingTasks)
    );

    let mut cancelled = WorkflowTaskGroup::new(workflow());
    cancelled.cancel_and_wait().await.unwrap();
    assert_eq!(
        cancelled.spawn(|_| async {}),
        Err(WorkflowTaskGroupError::NotAcceptingTasks)
    );

    let mut drained = WorkflowTaskGroup::new(workflow());
    drained.drain().await.unwrap();
    assert_eq!(
        drained.spawn(|_| async {}),
        Err(WorkflowTaskGroupError::NotAcceptingTasks)
    );
}

#[tokio::test]
async fn natural_drain_does_not_claim_or_request_cancellation() {
    let mut group = WorkflowTaskGroup::new(workflow());
    group.spawn(|_| async {}).unwrap();
    let evidence = group.drain().await.unwrap();
    assert_eq!(evidence.kind(), WorkflowTaskCompletionKind::Drained);
    assert!(!group.is_cancellation_requested());
    assert_eq!(group.state(), WorkflowTaskGroupState::Drained);
    assert_eq!(
        group.cancel_and_wait().await,
        Err(WorkflowTaskGroupError::ConflictingCompletion)
    );
}

#[tokio::test]
async fn panic_is_normalized_without_leaking_text_and_other_tasks_are_drained() {
    let mut group = WorkflowTaskGroup::new(workflow());
    let (release_tx, release_rx) = oneshot::channel();
    group
        .spawn(|_| async { panic!("private panic payload") })
        .unwrap();
    group
        .spawn(|_| async move {
            release_rx.await.unwrap();
        })
        .unwrap();
    release_tx.send(()).unwrap();
    let error = group.drain().await.unwrap_err();
    assert_eq!(error, WorkflowTaskGroupError::TaskJoinFailure);
    assert!(!format!("{error:?} {error}").contains("private panic payload"));
    assert_eq!(group.task_count(), 0);
}

struct DropSignal(Option<oneshot::Sender<()>>);
impl Drop for DropSignal {
    fn drop(&mut self) {
        let _ = self.0.take().unwrap().send(());
    }
}

#[tokio::test]
async fn dropping_owner_aborts_outstanding_work_instead_of_detaching_it() {
    let (started_tx, mut started_rx) = mpsc::channel(1);
    let (dropped_tx, dropped_rx) = oneshot::channel();
    let continued = Arc::new(AtomicUsize::new(0));
    let continued_in_task = Arc::clone(&continued);
    let mut group = WorkflowTaskGroup::new(workflow());
    group
        .spawn(move |_| async move {
            let _guard = DropSignal(Some(dropped_tx));
            started_tx.send(()).await.unwrap();
            std::future::pending::<()>().await;
            continued_in_task.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    started_rx.recv().await.unwrap();
    drop(group);
    dropped_rx.await.unwrap();
    tokio::task::yield_now().await;
    assert_eq!(continued.load(Ordering::SeqCst), 0);
}
