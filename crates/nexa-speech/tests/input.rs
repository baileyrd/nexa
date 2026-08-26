use nexa_domain::{ProtocolVersion, SpeechId, SpeechInputOperationId};
use nexa_speech::*;
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    task::Poll,
};
use uuid::Uuid;

fn request(seed: u128) -> SpeechInputRequest {
    SpeechInputRequest::new(
        SpeechId::new(Uuid::from_u128(seed)).unwrap(),
        SpeechInputOperationId::new(Uuid::from_u128(seed + 1)).unwrap(),
    )
}
fn evidence(r: &SpeechInputRequest, text: &str) -> SpeechInputEvidence {
    SpeechInputEvidence::new(r, text).unwrap()
}
fn signal() -> (
    Arc<ManualSpeechInputCancellation>,
    Arc<dyn SpeechInputCancellationSignal>,
) {
    let concrete = Arc::new(ManualSpeechInputCancellation::default());
    (Arc::clone(&concrete), concrete)
}
fn with_extra<T: serde::Serialize>(value: &T) -> Value {
    let mut value = serde_json::to_value(value).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("extra".into(), Value::Bool(true));
    value
}
fn with_version<T: serde::Serialize>(value: &T) -> Value {
    let mut value = serde_json::to_value(value).unwrap();
    value["contract_version"] = json!("2.0");
    value
}
fn with_nil<T: serde::Serialize>(value: &T, field: &str) -> Value {
    let mut value = serde_json::to_value(value).unwrap();
    value[field] = json!(Uuid::nil());
    value
}

#[test]
fn every_closed_outcome_variant_round_trips() {
    let r = request(1);
    for outcome in [
        SpeechInputOutcome::Success(evidence(&r, "private transcript")),
        SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence::for_request(&r)),
        SpeechInputOutcome::Failure(SpeechInputFailure::Unavailable),
        SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure),
    ] {
        let json = serde_json::to_string(&outcome).unwrap();
        assert_eq!(
            serde_json::from_str::<SpeechInputOutcome>(&json).unwrap(),
            outcome
        );
    }
}

#[test]
fn every_input_wire_rejects_unknown_fields() {
    let r = request(10);
    let success = evidence(&r, "private transcript");
    let cancelled = SpeechInputCancellationEvidence::for_request(&r);
    assert!(serde_json::from_value::<SpeechInputRequest>(with_extra(&r)).is_err());
    assert!(serde_json::from_value::<SpeechInputEvidence>(with_extra(&success)).is_err());
    assert!(
        serde_json::from_value::<SpeechInputCancellationEvidence>(with_extra(&cancelled)).is_err()
    );
    for outcome in [
        SpeechInputOutcome::Success(success),
        SpeechInputOutcome::Cancelled(cancelled),
        SpeechInputOutcome::Failure(SpeechInputFailure::Unavailable),
    ] {
        assert!(serde_json::from_value::<SpeechInputOutcome>(with_extra(&outcome)).is_err());
    }
}

#[test]
fn every_version_bearing_wire_rejects_unsupported_versions() {
    let r = request(20);
    let success = evidence(&r, "private transcript");
    let cancelled = SpeechInputCancellationEvidence::for_request(&r);
    assert!(serde_json::from_value::<SpeechInputRequest>(with_version(&r)).is_err());
    assert!(serde_json::from_value::<SpeechInputEvidence>(with_version(&success)).is_err());
    assert!(
        serde_json::from_value::<SpeechInputCancellationEvidence>(with_version(&cancelled))
            .is_err()
    );

    for mut outcome in [
        serde_json::to_value(SpeechInputOutcome::Success(success)).unwrap(),
        serde_json::to_value(SpeechInputOutcome::Cancelled(cancelled)).unwrap(),
    ] {
        outcome["evidence"]["contract_version"] = json!("2.0");
        assert!(serde_json::from_value::<SpeechInputOutcome>(outcome).is_err());
    }
}

#[test]
fn every_identity_bearing_wire_rejects_nil_identities() {
    let r = request(30);
    let success = evidence(&r, "private transcript");
    let cancelled = SpeechInputCancellationEvidence::for_request(&r);
    for field in ["speech_id", "operation_id"] {
        assert!(serde_json::from_value::<SpeechInputRequest>(with_nil(&r, field)).is_err());
        assert!(serde_json::from_value::<SpeechInputEvidence>(with_nil(&success, field)).is_err());
        assert!(
            serde_json::from_value::<SpeechInputCancellationEvidence>(with_nil(&cancelled, field))
                .is_err()
        );
        for mut outcome in [
            serde_json::to_value(SpeechInputOutcome::Success(success.clone())).unwrap(),
            serde_json::to_value(SpeechInputOutcome::Cancelled(cancelled)).unwrap(),
        ] {
            outcome["evidence"][field] = json!(Uuid::nil());
            assert!(serde_json::from_value::<SpeechInputOutcome>(outcome).is_err());
        }
    }
}

#[test]
fn outcome_rejects_unknown_variants_and_evidence_rejects_invalid_content() {
    assert!(serde_json::from_value::<SpeechInputOutcome>(json!({
        "kind": "future_variant",
        "evidence": null
    }))
    .is_err());
    let r = request(40);
    assert!(SpeechInputEvidence::new(&r, "").is_err());
    assert!(SpeechInputEvidence::new(&r, "x".repeat(MAX_SPEECH_INPUT_TEXT_BYTES + 1)).is_err());
}

#[derive(Default)]
struct CustomState {
    calls: usize,
    active: usize,
    received: Vec<SpeechInputRequest>,
}
struct CustomService {
    outcome: SpeechInputOutcome,
    state: Arc<Mutex<CustomState>>,
}
impl CustomService {
    fn new(outcome: SpeechInputOutcome) -> Self {
        Self {
            outcome,
            state: Arc::default(),
        }
    }
    fn counts(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.calls, state.active)
    }
    fn received(&self) -> Vec<SpeechInputRequest> {
        self.state.lock().unwrap().received.clone()
    }
}
struct CustomActive(Arc<Mutex<CustomState>>);
impl Drop for CustomActive {
    fn drop(&mut self) {
        self.0.lock().unwrap().active -= 1;
    }
}
impl SpeechInputService for CustomService {
    fn input(
        &self,
        request: SpeechInputRequest,
        _cancellation: Arc<dyn SpeechInputCancellationSignal>,
    ) -> SpeechInputFuture<'_> {
        let outcome = self.outcome.clone();
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            {
                let mut state = state.lock().unwrap();
                state.calls += 1;
                state.active += 1;
                state.received.push(request);
            }
            let _active = CustomActive(state);
            outcome
        })
    }
}

fn call_custom(
    r: SpeechInputRequest,
    outcome: SpeechInputOutcome,
) -> Result<SpeechInputOutcome, SpeechInputError> {
    let service = CustomService::new(outcome);
    let (_, cancellation) = signal();
    let result = complete(
        Box::pin(request_speech_input(
            &service,
            r,
            r.speech_id,
            r.operation_id,
            cancellation,
        ))
        .as_mut(),
    );
    assert_eq!(service.counts(), (1, 0));
    assert_eq!(service.received(), vec![r]);
    result
}

#[test]
fn custom_service_exact_success_and_cancellation_are_accepted_unchanged() {
    let r = request(50);
    for expected in [
        SpeechInputOutcome::Success(evidence(&r, "exact private transcript")),
        SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence::for_request(&r)),
    ] {
        assert_eq!(call_custom(r, expected.clone()).unwrap(), expected);
    }
}

#[test]
fn custom_service_evidence_is_never_reassociated_and_errors_are_exact() {
    let r = request(60);
    let wrong = request(70);
    let mut success_version = evidence(&r, "private transcript");
    success_version.contract_version = ProtocolVersion::new(2, 0);
    let mut cancellation_version = SpeechInputCancellationEvidence::for_request(&r);
    cancellation_version.contract_version = ProtocolVersion::new(2, 0);

    let cases = [
        (
            {
                let mut value = evidence(&r, "wrong speech");
                value.speech_id = wrong.speech_id;
                SpeechInputOutcome::Success(value)
            },
            SpeechInputError::AssociationMismatch,
        ),
        (
            {
                let mut value = evidence(&r, "wrong operation");
                value.operation_id = wrong.operation_id;
                SpeechInputOutcome::Success(value)
            },
            SpeechInputError::AssociationMismatch,
        ),
        (
            SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence {
                speech_id: wrong.speech_id,
                ..SpeechInputCancellationEvidence::for_request(&r)
            }),
            SpeechInputError::AssociationMismatch,
        ),
        (
            SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence {
                operation_id: wrong.operation_id,
                ..SpeechInputCancellationEvidence::for_request(&r)
            }),
            SpeechInputError::AssociationMismatch,
        ),
        (
            SpeechInputOutcome::Success(success_version),
            SpeechInputError::UnsupportedVersion,
        ),
        (
            SpeechInputOutcome::Cancelled(cancellation_version),
            SpeechInputError::UnsupportedVersion,
        ),
    ];
    for (dependency_outcome, expected_error) in cases {
        let preserved = dependency_outcome.clone();
        assert_eq!(
            call_custom(r, dependency_outcome.clone()).unwrap_err(),
            expected_error
        );
        assert_eq!(dependency_outcome, preserved);
    }
}

#[test]
fn fifo_success_failures_exhaustion_and_exact_consumption() {
    let a = request(80);
    let b = request(90);
    let service = ScriptedSpeechInputService::new([
        ScriptedSpeechInputOutcome::Success(evidence(&a, "alpha private")),
        ScriptedSpeechInputOutcome::Unavailable,
        ScriptedSpeechInputOutcome::DependencyFailure,
    ]);
    let expected = [
        SpeechInputOutcome::Success(evidence(&a, "alpha private")),
        SpeechInputOutcome::Failure(SpeechInputFailure::Unavailable),
        SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure),
        SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure),
    ];
    for (index, expected) in expected.into_iter().enumerate() {
        let r = if index == 0 { a } else { b };
        let (_, s) = signal();
        assert_eq!(
            complete(
                Box::pin(request_speech_input(
                    &service,
                    r,
                    r.speech_id,
                    r.operation_id,
                    s
                ))
                .as_mut()
            )
            .unwrap(),
            expected
        );
    }
    assert_eq!(service.consumed_outcome_count(), 3);
    assert_eq!(service.received_requests(), vec![a, b, b, b]);
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn all_preflight_failures_consume_nothing() {
    let good = request(100);
    let wrong = request(110);
    let service = ScriptedSpeechInputService::new([ScriptedSpeechInputOutcome::Success(evidence(
        &good, "secret",
    ))]);
    let mut bad = good;
    bad.contract_version = ProtocolVersion::new(9, 0);
    let (_, s) = signal();
    assert_eq!(
        complete(
            Box::pin(request_speech_input(
                &service,
                bad,
                good.speech_id,
                good.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap_err(),
        SpeechInputError::UnsupportedVersion
    );
    for (speech, operation) in [
        (wrong.speech_id, good.operation_id),
        (good.speech_id, wrong.operation_id),
    ] {
        let (_, s) = signal();
        assert_eq!(
            complete(Box::pin(request_speech_input(&service, good, speech, operation, s)).as_mut())
                .unwrap_err(),
            SpeechInputError::AssociationMismatch
        );
    }
    assert_eq!(service.consumed_outcome_count(), 0);
    assert!(service.received_requests().is_empty());
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn pre_cancel_waiting_cancel_and_caller_drop_are_owned() {
    let r = request(120);
    let service = ScriptedSpeechInputService::new([ScriptedSpeechInputOutcome::Success(evidence(
        &r, "unused",
    ))]);
    let (c, s) = signal();
    c.cancel();
    assert_eq!(
        complete(
            Box::pin(request_speech_input(
                &service,
                r,
                r.speech_id,
                r.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap(),
        SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence::for_request(&r))
    );
    assert_eq!(service.consumed_outcome_count(), 0);
    assert_eq!(service.remaining_outcome_count(), 1);

    let service =
        ScriptedSpeechInputService::new([ScriptedSpeechInputOutcome::WaitForCancellation]);
    let (c, s) = signal();
    let mut future = Box::pin(request_speech_input(
        &service,
        r,
        r.speech_id,
        r.operation_id,
        s,
    ));
    assert!(matches!(poll_once(future.as_mut()), Poll::Pending));
    assert_eq!(service.active_future_count(), 1);
    c.cancel();
    assert_eq!(
        complete(future.as_mut()).unwrap(),
        SpeechInputOutcome::Cancelled(SpeechInputCancellationEvidence::for_request(&r))
    );
    assert_eq!(service.active_future_count(), 0);

    let service =
        ScriptedSpeechInputService::new([ScriptedSpeechInputOutcome::WaitForCancellation]);
    let (_, s) = signal();
    let mut future = Box::pin(request_speech_input(
        &service,
        r,
        r.speech_id,
        r.operation_id,
        s,
    ));
    assert!(matches!(poll_once(future.as_mut()), Poll::Pending));
    assert_eq!(service.active_future_count(), 1);
    drop(future);
    assert_eq!(service.active_future_count(), 0);
    assert_eq!(service.consumed_outcome_count(), 1);
}

#[test]
fn all_exercised_public_diagnostics_are_content_free() {
    let r = request(130);
    let e = evidence(
        &r,
        "transcript-secret authorization-secret audio-bytes-secret",
    );
    let cancelled = SpeechInputCancellationEvidence::for_request(&r);
    let diagnostics = [
        format!("{r:?}"),
        format!("{e:?}"),
        format!("{e}"),
        format!("{cancelled:?}"),
        format!("{:?}", SpeechInputFailure::Unavailable),
        format!("{}", SpeechInputOutcome::Success(e.clone())),
        format!("{:?}", SpeechInputOutcome::Success(e)),
        format!("{}", SpeechInputOutcome::Cancelled(cancelled)),
        format!("{:?}", SpeechInputOutcome::Cancelled(cancelled)),
        format!(
            "{}",
            SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure)
        ),
        format!(
            "{:?}",
            SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure)
        ),
        format!("{}", SpeechInputError::UnsupportedVersion),
        format!("{:?}", SpeechInputError::AssociationMismatch),
        format!("{}", SpeechInputError::InvalidEvidence),
    ];
    for diagnostic in diagnostics {
        for secret in [
            "transcript-secret",
            "authorization-secret",
            "audio-bytes-secret",
            "internal reason",
        ] {
            assert!(!diagnostic.contains(secret), "{diagnostic}");
        }
    }
}

fn complete<F: std::future::Future>(mut f: std::pin::Pin<&mut F>) -> F::Output {
    match poll_once(f.as_mut()) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("future unexpectedly pending"),
    }
}
fn poll_once<F: std::future::Future>(mut f: std::pin::Pin<&mut F>) -> Poll<F::Output> {
    let w = std::task::Waker::noop();
    f.as_mut().poll(&mut std::task::Context::from_waker(w))
}
