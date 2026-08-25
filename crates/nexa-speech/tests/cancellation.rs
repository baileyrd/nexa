use nexa_domain::{ProtocolVersion, SpeechId};
use nexa_speech::{
    request_speech_cancellation, ScriptedSpeechCancellationOutcome as ScriptedOutcome,
    ScriptedSpeechCancellationService as Service, SpeechCancellationAcknowledgement as Ack,
    SpeechCancellationError as Error, SpeechCancellationRequest as Request,
    SpeechCancellationServiceOutcome as ServiceOutcome,
};
use serde_json::json;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use uuid::Uuid;

fn speech_id(value: u128) -> SpeechId {
    SpeechId::new(Uuid::from_u128(value)).unwrap()
}

fn request(value: u128) -> Request {
    Request::new(speech_id(value))
}

fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    future.poll(&mut Context::from_waker(Waker::noop()))
}

fn complete<F: Future>(mut future: Pin<&mut F>) -> F::Output {
    match poll_once(future.as_mut()) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("future unexpectedly pending"),
    }
}

#[test]
fn exact_v1_wire_is_strict_and_round_trips() {
    let request = request(1);
    let acknowledgement = Ack::for_request(&request);
    let exact = r#"{"contract_version":"1.0","speech_id":"00000000-0000-0000-0000-000000000001"}"#;
    assert_eq!(serde_json::to_string(&request).unwrap(), exact);
    assert_eq!(serde_json::to_string(&acknowledgement).unwrap(), exact);
    assert_eq!(serde_json::from_str::<Request>(exact).unwrap(), request);
    assert_eq!(serde_json::from_str::<Ack>(exact).unwrap(), acknowledgement);

    for invalid in [
        json!({"contract_version":"2.0","speech_id":speech_id(1)}),
        json!({"contract_version":"1.0","speech_id":Uuid::nil()}),
        json!({"contract_version":"1.0"}),
        json!({"contract_version":"1.0","speech_id":speech_id(1),"extra":true}),
    ] {
        assert!(serde_json::from_value::<Request>(invalid.clone()).is_err());
        assert!(serde_json::from_value::<Ack>(invalid).is_err());
    }
    assert!(serde_json::from_value::<ServiceOutcome>(json!({"kind":"provider_secret"})).is_err());
}

#[test]
fn acknowledgement_construction_uses_v1_and_preserves_the_exact_id() {
    let mut request = request(1);
    request.contract_version = ProtocolVersion::new(2, 0);

    let acknowledgement = Ack::for_request(&request);

    assert_eq!(
        acknowledgement.contract_version,
        nexa_speech::SPEECH_CANCELLATION_V1
    );
    assert_eq!(acknowledgement.speech_id, request.speech_id);
}

#[test]
fn exact_success_and_fifo_accounting() {
    let first = request(1);
    let second = request(2);
    let first_ack = Ack::for_request(&first);
    let second_ack = Ack::for_request(&second);
    let service = Service::new([
        ScriptedOutcome::Acknowledged(first_ack),
        ScriptedOutcome::DependencyFailure,
        ScriptedOutcome::Acknowledged(second_ack),
    ]);

    assert_eq!(
        complete(
            Box::pin(request_speech_cancellation(
                &service,
                first,
                first.speech_id
            ))
            .as_mut()
        ),
        Ok(first_ack)
    );
    assert_eq!(
        complete(
            Box::pin(request_speech_cancellation(
                &service,
                second,
                second.speech_id
            ))
            .as_mut()
        ),
        Err(Error::DependencyFailure)
    );
    assert_eq!(
        complete(
            Box::pin(request_speech_cancellation(
                &service,
                second,
                second.speech_id
            ))
            .as_mut()
        ),
        Ok(second_ack)
    );
    assert_eq!(service.received_requests(), vec![first, second, second]);
    assert_eq!(service.consumed_outcome_count(), 3);
    assert_eq!(service.remaining_outcome_count(), 0);
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn preflight_failures_do_not_invoke_or_consume() {
    let valid = request(1);
    let service = Service::new([ScriptedOutcome::Acknowledged(Ack::for_request(&valid))]);
    let mut unsupported = valid;
    unsupported.contract_version = ProtocolVersion::new(2, 0);

    assert_eq!(
        complete(
            Box::pin(request_speech_cancellation(
                &service,
                unsupported,
                valid.speech_id
            ))
            .as_mut()
        ),
        Err(Error::UnsupportedVersion)
    );
    assert_eq!(
        complete(Box::pin(request_speech_cancellation(&service, valid, speech_id(2))).as_mut()),
        Err(Error::AssociationMismatch)
    );
    assert!(service.received_requests().is_empty());
    assert_eq!(service.consumed_outcome_count(), 0);
    assert_eq!(service.remaining_outcome_count(), 1);
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn dependency_failure_exhaustion_and_acknowledgement_mismatch_fail_closed() {
    let request = request(1);
    let mut wrong_id = Ack::for_request(&request);
    wrong_id.speech_id = speech_id(2);
    let mut wrong_version = Ack::for_request(&request);
    wrong_version.contract_version = ProtocolVersion::new(2, 0);
    let service = Service::new([
        ScriptedOutcome::DependencyFailure,
        ScriptedOutcome::Acknowledged(wrong_id),
        ScriptedOutcome::Acknowledged(wrong_version),
    ]);
    for expected in [
        Error::DependencyFailure,
        Error::AcknowledgementMismatch,
        Error::AcknowledgementMismatch,
        Error::DependencyFailure,
    ] {
        assert_eq!(
            complete(
                Box::pin(request_speech_cancellation(
                    &service,
                    request,
                    request.speech_id
                ))
                .as_mut()
            ),
            Err(expected)
        );
    }
    assert_eq!(service.received_requests(), vec![request; 4]);
    assert_eq!(service.consumed_outcome_count(), 3);
    assert_eq!(service.remaining_outcome_count(), 0);
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn dropping_pending_host_future_removes_all_active_work() {
    let request = request(1);
    let service = Service::new([ScriptedOutcome::Pending]);
    let mut future = Box::pin(request_speech_cancellation(
        &service,
        request,
        request.speech_id,
    ));
    assert!(poll_once(future.as_mut()).is_pending());
    assert_eq!(service.received_requests(), vec![request]);
    assert_eq!(service.consumed_outcome_count(), 1);
    assert_eq!(service.active_future_count(), 1);
    drop(future);
    assert_eq!(service.active_future_count(), 0);
}

#[test]
fn every_public_diagnostic_is_content_free() {
    let request = request(1);
    let acknowledgement = Ack::for_request(&request);
    let surfaces = [
        format!("{request:?}"),
        format!("{acknowledgement:?}"),
        format!(
            "{:?} {}",
            ServiceOutcome::Acknowledged(acknowledgement),
            ServiceOutcome::Acknowledged(acknowledgement)
        ),
        format!(
            "{:?} {}",
            ServiceOutcome::DependencyFailure,
            ServiceOutcome::DependencyFailure
        ),
        format!(
            "{:?} {}",
            ScriptedOutcome::Pending,
            ScriptedOutcome::Pending
        ),
        format!(
            "{:?} {}",
            Error::UnsupportedVersion,
            Error::UnsupportedVersion
        ),
        format!(
            "{:?} {}",
            Error::AssociationMismatch,
            Error::AssociationMismatch
        ),
        format!(
            "{:?} {}",
            Error::DependencyFailure,
            Error::DependencyFailure
        ),
        format!(
            "{:?} {}",
            Error::AcknowledgementMismatch,
            Error::AcknowledgementMismatch
        ),
    ];
    for surface in surfaces {
        for secret in [
            "learner-private",
            "synthesized-private",
            "audio-private",
            "provider-private",
            "endpoint-private",
            "credential-private",
        ] {
            assert!(!surface.contains(secret));
        }
    }
}
