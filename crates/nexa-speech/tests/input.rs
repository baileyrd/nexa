use nexa_domain::{ProtocolVersion, SpeechId, SpeechInputOperationId};
use nexa_speech::*;
use std::{sync::Arc, task::Poll};
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

#[test]
fn strict_v1_wire_rejects_unknown_versions_fields_variants_nil_and_invalid_evidence() {
    let r = request(1);
    let json = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::from_str::<SpeechInputRequest>(&json).unwrap(),
        r
    );
    for bad in [
        json.replace("1.0", "2.0"),
        json.replace('}', ",\"extra\":true}"),
        json.replace(&r.speech_id.to_string(), &Uuid::nil().to_string()),
        json.replace(&r.operation_id.to_string(), &Uuid::nil().to_string()),
    ] {
        assert!(
            serde_json::from_str::<SpeechInputRequest>(&bad).is_err(),
            "{bad}"
        );
    }
    let e = evidence(&r, "private transcript");
    let ej = serde_json::to_string(&e).unwrap();
    assert_eq!(serde_json::from_str::<SpeechInputEvidence>(&ej).unwrap(), e);
    assert!(
        serde_json::from_str::<SpeechInputEvidence>(&ej.replace("private transcript", "")).is_err()
    );
    assert!(serde_json::from_str::<SpeechInputOutcome>(
        r#"{"kind":"future_variant","evidence":null}"#
    )
    .is_err());
    let cancelled = SpeechInputCancellationEvidence::for_request(&r);
    let cj = serde_json::to_string(&cancelled).unwrap();
    assert!(
        serde_json::from_str::<SpeechInputCancellationEvidence>(&cj.replace("1.0", "2.0")).is_err()
    );
    assert!(SpeechInputEvidence::new(&r, "x".repeat(MAX_SPEECH_INPUT_TEXT_BYTES + 1)).is_err());
}

#[test]
fn fifo_success_failures_exhaustion_and_exact_consumption() {
    let a = request(10);
    let b = request(20);
    let service = ScriptedSpeechInputService::new([
        ScriptedSpeechInputOutcome::Success(evidence(&a, "alpha private")),
        ScriptedSpeechInputOutcome::Unavailable,
        ScriptedSpeechInputOutcome::DependencyFailure,
    ]);
    let (_, s) = signal();
    assert!(matches!(
        complete(
            Box::pin(request_speech_input(
                &service,
                a,
                a.speech_id,
                a.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap(),
        SpeechInputOutcome::Success(_)
    ));
    let (_, s) = signal();
    assert_eq!(
        complete(
            Box::pin(request_speech_input(
                &service,
                b,
                b.speech_id,
                b.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap(),
        SpeechInputOutcome::Failure(SpeechInputFailure::Unavailable)
    );
    let (_, s) = signal();
    assert_eq!(
        complete(
            Box::pin(request_speech_input(
                &service,
                b,
                b.speech_id,
                b.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap(),
        SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure)
    );
    let (_, s) = signal();
    assert_eq!(
        complete(
            Box::pin(request_speech_input(
                &service,
                b,
                b.speech_id,
                b.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap(),
        SpeechInputOutcome::Failure(SpeechInputFailure::DependencyFailure)
    );
    assert_eq!(service.consumed_outcome_count(), 3);
    assert_eq!(service.received_requests(), vec![a, b, b, b]);
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn all_preflight_failures_consume_nothing_and_reassociation_is_rejected() {
    let good = request(30);
    let wrong = request(40);
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
    let mismatch = ScriptedSpeechInputService::new([ScriptedSpeechInputOutcome::Success(
        evidence(&wrong, "other secret"),
    )]);
    let (_, s) = signal();
    assert_eq!(
        complete(
            Box::pin(request_speech_input(
                &mismatch,
                good,
                good.speech_id,
                good.operation_id,
                s
            ))
            .as_mut()
        )
        .unwrap_err(),
        SpeechInputError::AssociationMismatch
    );
    assert_eq!(mismatch.consumed_outcome_count(), 1);
}

#[test]
fn pre_cancel_waiting_cancel_and_caller_drop_are_owned() {
    let r = request(50);
    let service = ScriptedSpeechInputService::new([ScriptedSpeechInputOutcome::Success(evidence(
        &r, "unused",
    ))]);
    let (c, s) = signal();
    c.cancel();
    assert!(matches!(
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
        SpeechInputOutcome::Cancelled(_)
    ));
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
    let out = complete(future.as_mut()).unwrap();
    assert_eq!(
        out,
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
fn diagnostics_redact_content_and_internal_reasons() {
    let r = request(60);
    let e = evidence(
        &r,
        "transcript-secret authorization-secret audio-bytes-secret",
    );
    for diagnostic in [
        format!("{e:?}"),
        format!("{e}"),
        format!("{:?}", SpeechInputOutcome::Success(e.clone())),
        format!("{}", SpeechInputOutcome::Success(e)),
        format!(
            "{:?} {}",
            SpeechInputError::InvalidEvidence,
            SpeechInputError::InvalidEvidence
        ),
    ] {
        for secret in [
            "transcript-secret",
            "authorization-secret",
            "audio-bytes-secret",
            "internal reason",
        ] {
            assert!(!diagnostic.contains(secret));
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
