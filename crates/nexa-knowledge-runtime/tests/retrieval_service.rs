use nexa_domain::{RetrievalQueryId, RetrievalResultId};
use nexa_knowledge::{
    Audience, RetrievalFilters, RetrievalQuery, RetrievalResult, LEXICAL_RETRIEVAL_V1, V1,
};
use nexa_knowledge_runtime::{
    retrieve, RetrievalServiceError, RetrievalServiceOutcome, ScriptedRetrievalOutcome,
    ScriptedRetrievalService,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

fn ids(seed: u128) -> (RetrievalQueryId, RetrievalResultId) {
    (
        RetrievalQueryId::new(Uuid::from_u128(seed)).unwrap(),
        RetrievalResultId::new(Uuid::from_u128(seed + 1)).unwrap(),
    )
}

fn query(seed: u128, text: &str) -> RetrievalQuery {
    let (query_id, result_id) = ids(seed);
    RetrievalQuery {
        contract_version: V1,
        retrieval_policy_version: LEXICAL_RETRIEVAL_V1,
        query_id,
        result_id,
        text: text.into(),
        filters: RetrievalFilters {
            audience: Audience::StudentLearning,
            course_id: None,
            lesson_id: None,
        },
        maximum_results: 1,
    }
}

fn result(query: &RetrievalQuery) -> RetrievalResult {
    RetrievalResult {
        contract_version: V1,
        retrieval_policy_version: LEXICAL_RETRIEVAL_V1,
        query_id: query.query_id,
        result_id: query.result_id,
        candidates: vec![],
        exclusions: vec![],
    }
}

#[tokio::test]
async fn success_preserves_exact_identity_and_consumes_fifo_once() {
    let first = query(1, "first private query");
    let second = query(3, "second private query");
    let service = ScriptedRetrievalService::new([
        ScriptedRetrievalOutcome::Success(result(&first)),
        ScriptedRetrievalOutcome::Success(result(&second)),
    ]);
    for expected in [&first, &second] {
        let outcome = retrieve(&service, expected.clone(), CancellationToken::new())
            .await
            .unwrap();
        let RetrievalServiceOutcome::Success(actual) = outcome else {
            panic!("expected success")
        };
        assert_eq!(
            (actual.query_id, actual.result_id),
            (expected.query_id, expected.result_id)
        );
    }
    assert_eq!(service.received_queries(), vec![first, second]);
    assert_eq!(service.consumed_outcome_count(), 2);
    assert_eq!(service.remaining_outcome_count(), 0);
    assert_eq!(service.active_operation_count(), 0);
}

#[tokio::test]
async fn waiting_operation_observes_cancellation_and_terminates() {
    let query = query(10, "never disclose this");
    let token = CancellationToken::new();
    let service = ScriptedRetrievalService::new([ScriptedRetrievalOutcome::WaitForCancellation]);
    let future = retrieve(&service, query.clone(), token.clone());
    tokio::pin!(future);
    assert!(matches!(
        futures_poll_once(&mut future),
        std::task::Poll::Pending
    ));
    assert_eq!(service.active_operation_count(), 1);
    token.cancel();
    let outcome = future.await.unwrap();
    let RetrievalServiceOutcome::Cancelled(evidence) = outcome else {
        panic!("expected cancellation")
    };
    assert_eq!(
        (evidence.query_id(), evidence.result_id()),
        (query.query_id, query.result_id)
    );
    assert_eq!(service.consumed_outcome_count(), 1);
    assert_eq!(service.active_operation_count(), 0);
}

#[tokio::test]
async fn cancellation_before_call_records_request_without_consuming_outcome() {
    let query = query(20, "cancelled sensitive query");
    let token = CancellationToken::new();
    token.cancel();
    let service =
        ScriptedRetrievalService::new([ScriptedRetrievalOutcome::Success(result(&query))]);
    assert!(matches!(
        retrieve(&service, query.clone(), token).await.unwrap(),
        RetrievalServiceOutcome::Cancelled(_)
    ));
    assert_eq!(service.received_queries(), vec![query]);
    assert_eq!(service.consumed_outcome_count(), 0);
    assert_eq!(service.remaining_outcome_count(), 1);
    assert_eq!(service.active_operation_count(), 0);
}

#[tokio::test]
async fn failures_exhaustion_and_identity_mismatch_are_closed() {
    let request = query(30, "secret phrase");
    let wrong = query(40, "different secret");
    for (service, expected) in [
        (
            ScriptedRetrievalService::new([ScriptedRetrievalOutcome::DependencyFailure]),
            RetrievalServiceError::DependencyFailure,
        ),
        (
            ScriptedRetrievalService::default(),
            RetrievalServiceError::DependencyFailure,
        ),
        (
            ScriptedRetrievalService::new([ScriptedRetrievalOutcome::Success(result(&wrong))]),
            RetrievalServiceError::AssociationMismatch,
        ),
    ] {
        let error = retrieve(&service, request.clone(), CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains("secret"));
        assert_eq!(service.active_operation_count(), 0);
    }
    assert!(!format!("{request:?}").contains("secret phrase"));
}

#[tokio::test]
async fn dropping_caller_future_terminates_all_adapter_work() {
    let query = query(50, "drop me");
    let service = ScriptedRetrievalService::new([ScriptedRetrievalOutcome::WaitForCancellation]);
    let mut future = Box::pin(retrieve(&service, query, CancellationToken::new()));
    assert!(matches!(
        futures_poll_once(&mut future.as_mut()),
        std::task::Poll::Pending
    ));
    assert_eq!(service.active_operation_count(), 1);
    drop(future);
    assert_eq!(service.active_operation_count(), 0);
    assert_eq!(service.consumed_outcome_count(), 1);
}

fn futures_poll_once<F: std::future::Future>(
    future: &mut std::pin::Pin<&mut F>,
) -> std::task::Poll<F::Output> {
    struct Noop;
    impl std::task::Wake for Noop {
        fn wake(self: std::sync::Arc<Self>) {}
    }
    let waker = std::task::Waker::from(std::sync::Arc::new(Noop));
    future
        .as_mut()
        .poll(&mut std::task::Context::from_waker(&waker))
}
