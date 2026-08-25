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

use nexa_speech::{
    ScriptedSpeechCancellationParticipant as Participant,
    SpeechCancellationAggregateEvidence as AggregateEvidence,
    SpeechCancellationAggregateKind as AggregateKind, SpeechCancellationCapability as Capability,
    SpeechCancellationCoordinator as Coordinator,
    SpeechCancellationCoordinatorError as CoordinatorError, SpeechCancellationParticipant,
    SpeechCancellationSurface as Surface, SpeechSurfaceCancellationEvidence as SurfaceEvidence,
};

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
    let capability = Capability::cancellable(request.speech_id, Surface::Synthesis);
    let evidence = SurfaceEvidence {
        contract_version: nexa_speech::SPEECH_CANCELLATION_V1,
        speech_id: request.speech_id,
        surface: Surface::Synthesis,
        acknowledgement,
    };
    let aggregate = AggregateEvidence {
        contract_version: nexa_speech::SPEECH_CANCELLATION_V1,
        speech_id: request.speech_id,
        kind: AggregateKind::Stopped,
        surfaces: vec![evidence; 4],
    };
    let service = Service::new([ScriptedOutcome::Pending]);
    let participant = Participant::new(capability, [ScriptedOutcome::Pending]);
    let mut diagnostics = vec![
        format!("{request:?}"),
        format!("{acknowledgement:?}"),
        format!("{:?}", Surface::Synthesis),
        format!("{capability:?}"),
        format!("{evidence:?}"),
        format!("{:?}", AggregateKind::Stopped),
        format!("{aggregate:?}"),
        format!("{service:?}"),
        format!("{participant:?}"),
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
    diagnostics.extend(
        [
            CoordinatorError::UnsupportedVersion,
            CoordinatorError::InvalidCapabilitySet,
            CoordinatorError::MissingSurface,
            CoordinatorError::DuplicateSurface,
            CoordinatorError::NonCancellableSurface,
            CoordinatorError::AssociationMismatch,
            CoordinatorError::DependencyFailure,
            CoordinatorError::AcknowledgementMismatch,
            CoordinatorError::AggregateFailure,
        ]
        .map(|error| format!("{error:?} {error}")),
    );
    for surface in diagnostics {
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

fn participant(id: SpeechId, surface: Surface, outcome: ScriptedOutcome) -> Participant {
    Participant::new(Capability::cancellable(id, surface), [outcome])
}

fn assert_untouched(participants: &[Participant], remaining: usize) {
    assert!(participants.iter().all(|participant| {
        participant.received_requests().is_empty()
            && participant.consumed_outcome_count() == 0
            && participant.remaining_outcome_count() == remaining
            && participant.active_future_count() == 0
    }));
}

#[test]
fn composite_capability_wire_is_strict() {
    let value = Capability::cancellable(speech_id(1), Surface::QueuedAudio);
    let exact = r#"{"contract_version":"1.0","speech_id":"00000000-0000-0000-0000-000000000001","surface":"queued_audio","cancellable":true}"#;
    assert_eq!(serde_json::to_string(&value).unwrap(), exact);
    assert_eq!(serde_json::from_str::<Capability>(exact).unwrap(), value);
    let declared_unavailable = json!({"contract_version":"1.0","speech_id":speech_id(1),"surface":"queued_audio","cancellable":false});
    assert!(
        !serde_json::from_value::<Capability>(declared_unavailable)
            .unwrap()
            .cancellable
    );
    for invalid in [
        json!({"contract_version":"2.0","speech_id":speech_id(1),"surface":"queued_audio","cancellable":true}),
        json!({"contract_version":"1.0","speech_id":Uuid::nil(),"surface":"queued_audio","cancellable":true}),
        json!({"contract_version":"1.0","speech_id":speech_id(1),"surface":"unknown","cancellable":true}),
        json!({"contract_version":"1.0","speech_id":speech_id(1),"surface":"queued_audio"}),
        json!({"contract_version":"1.0","speech_id":speech_id(1),"surface":"queued_audio","cancellable":true,"secret":"x"}),
    ] {
        assert!(serde_json::from_value::<Capability>(invalid).is_err());
    }
}

#[test]
fn new_closed_enums_have_exact_strict_v1_wire() {
    for (surface, exact) in [
        (Surface::Synthesis, r#""synthesis""#),
        (Surface::QueuedAudio, r#""queued_audio""#),
        (Surface::Playback, r#""playback""#),
        (Surface::VisemeTimeline, r#""viseme_timeline""#),
    ] {
        assert_eq!(serde_json::to_string(&surface).unwrap(), exact);
        assert_eq!(serde_json::from_str::<Surface>(exact).unwrap(), surface);
    }
    assert_eq!(
        serde_json::to_string(&AggregateKind::Stopped).unwrap(),
        r#""stopped""#
    );
    assert_eq!(
        serde_json::from_str::<AggregateKind>(r#""stopped""#).unwrap(),
        AggregateKind::Stopped
    );
    assert!(serde_json::from_str::<Surface>(r#""unknown""#).is_err());
    assert!(serde_json::from_str::<AggregateKind>(r#""unknown""#).is_err());
}

#[test]
fn surface_and_aggregate_evidence_wire_is_exact_strict_and_canonical() {
    let id = speech_id(1);
    let ack = Ack::for_request(&Request::new(id));
    let surfaces = Surface::ALL
        .map(|surface| SurfaceEvidence {
            contract_version: nexa_speech::SPEECH_CANCELLATION_V1,
            speech_id: id,
            surface,
            acknowledgement: ack,
        })
        .to_vec();
    let surface_exact = r#"{"contract_version":"1.0","speech_id":"00000000-0000-0000-0000-000000000001","surface":"synthesis","acknowledgement":{"contract_version":"1.0","speech_id":"00000000-0000-0000-0000-000000000001"}}"#;
    assert_eq!(serde_json::to_string(&surfaces[0]).unwrap(), surface_exact);
    assert_eq!(
        serde_json::from_str::<SurfaceEvidence>(surface_exact).unwrap(),
        surfaces[0]
    );
    let aggregate = AggregateEvidence {
        contract_version: nexa_speech::SPEECH_CANCELLATION_V1,
        speech_id: id,
        kind: AggregateKind::Stopped,
        surfaces: surfaces.clone(),
    };
    let exact = serde_json::to_string(&aggregate).unwrap();
    assert_eq!(
        serde_json::from_str::<AggregateEvidence>(&exact).unwrap(),
        aggregate
    );

    let valid_surface = serde_json::to_value(surfaces[0]).unwrap();
    for invalid in [
        json!({"speech_id":id,"surface":"synthesis","acknowledgement":ack}),
        json!({"contract_version":"2.0","speech_id":id,"surface":"synthesis","acknowledgement":ack}),
        json!({"contract_version":"1.0","speech_id":Uuid::nil(),"surface":"synthesis","acknowledgement":ack}),
        json!({"contract_version":"1.0","speech_id":id,"surface":"unknown","acknowledgement":ack}),
        json!({"contract_version":"1.0","speech_id":id,"surface":"synthesis","acknowledgement":{"contract_version":"2.0","speech_id":id}}),
        json!({"contract_version":"1.0","speech_id":id,"surface":"synthesis","acknowledgement":{"contract_version":"1.0","speech_id":speech_id(2)}}),
        json!({"contract_version":"1.0","speech_id":id,"surface":"synthesis","acknowledgement":ack,"extra":true}),
    ] {
        assert!(serde_json::from_value::<SurfaceEvidence>(invalid).is_err());
    }

    let values = surfaces
        .iter()
        .map(|value| serde_json::to_value(value).unwrap())
        .collect::<Vec<_>>();
    for invalid in [
        json!({"speech_id":id,"kind":"stopped","surfaces":values}),
        json!({"contract_version":"2.0","speech_id":id,"kind":"stopped","surfaces":values}),
        json!({"contract_version":"1.0","speech_id":Uuid::nil(),"kind":"stopped","surfaces":values}),
        json!({"contract_version":"1.0","speech_id":id,"kind":"unknown","surfaces":values}),
        json!({"contract_version":"1.0","speech_id":id,"kind":"stopped","surfaces":&values[..3]}),
        json!({"contract_version":"1.0","speech_id":id,"kind":"stopped","surfaces":[valid_surface.clone(),valid_surface.clone(),values[2].clone(),values[3].clone()]}),
        json!({"contract_version":"1.0","speech_id":id,"kind":"stopped","surfaces":[values[1].clone(),values[0].clone(),values[2].clone(),values[3].clone()]}),
        json!({"contract_version":"1.0","speech_id":speech_id(2),"kind":"stopped","surfaces":values}),
        json!({"contract_version":"1.0","speech_id":id,"kind":"stopped","surfaces":values,"extra":true}),
    ] {
        assert!(serde_json::from_value::<AggregateEvidence>(invalid).is_err());
    }
}

#[test]
fn composite_is_canonical_and_invokes_every_surface_exactly_once() {
    let id = speech_id(10);
    let request = Request::new(id);
    let ack = Ack::for_request(&request);
    let participants = [
        participant(id, Surface::Playback, ScriptedOutcome::Acknowledged(ack)),
        participant(id, Surface::Synthesis, ScriptedOutcome::Acknowledged(ack)),
        participant(
            id,
            Surface::VisemeTimeline,
            ScriptedOutcome::Acknowledged(ack),
        ),
        participant(id, Surface::QueuedAudio, ScriptedOutcome::Acknowledged(ack)),
    ];
    let refs: Vec<&dyn SpeechCancellationParticipant> =
        participants.iter().map(|p| p as _).collect();
    let coordinator = Coordinator::new(id, refs).unwrap();
    let evidence = complete(Box::pin(coordinator.cancel(request)).as_mut()).unwrap();
    assert_eq!(
        evidence
            .surfaces
            .iter()
            .map(|e| e.surface)
            .collect::<Vec<_>>(),
        Surface::ALL
    );
    assert!(participants
        .iter()
        .all(|p| p.received_requests() == [request]));
    assert!(participants
        .iter()
        .all(|p| p.consumed_outcome_count() == 1 && p.active_future_count() == 0));
    let second = complete(Box::pin(coordinator.cancel(request)).as_mut());
    assert_eq!(second, Err(CoordinatorError::DependencyFailure));
}

#[test]
fn invalid_composite_sets_fail_before_activation() {
    let id = speech_id(11);
    let ack = Ack::for_request(&Request::new(id));
    let duplicate = [
        participant(id, Surface::Synthesis, ScriptedOutcome::Acknowledged(ack)),
        participant(id, Surface::Synthesis, ScriptedOutcome::Acknowledged(ack)),
        participant(id, Surface::Playback, ScriptedOutcome::Acknowledged(ack)),
        participant(
            id,
            Surface::VisemeTimeline,
            ScriptedOutcome::Acknowledged(ack),
        ),
    ];
    let refs: Vec<&dyn SpeechCancellationParticipant> = duplicate.iter().map(|p| p as _).collect();
    assert!(matches!(
        Coordinator::new(id, refs),
        Err(CoordinatorError::DuplicateSurface)
    ));
    assert!(duplicate
        .iter()
        .all(|p| p.received_requests().is_empty() && p.consumed_outcome_count() == 0));

    let non_cancellable = Participant::new(
        Capability {
            cancellable: false,
            ..Capability::cancellable(id, Surface::QueuedAudio)
        },
        [ScriptedOutcome::Acknowledged(ack)],
    );
    let refs: Vec<&dyn SpeechCancellationParticipant> = vec![
        &duplicate[0],
        &non_cancellable,
        &duplicate[2],
        &duplicate[3],
    ];
    assert!(matches!(
        Coordinator::new(id, refs),
        Err(CoordinatorError::NonCancellableSurface)
    ));
    assert_untouched(&duplicate, 1);
    assert!(non_cancellable.received_requests().is_empty());
    assert_eq!(non_cancellable.consumed_outcome_count(), 0);
    assert_eq!(non_cancellable.remaining_outcome_count(), 1);
    assert_eq!(non_cancellable.active_future_count(), 0);
}

#[test]
fn every_coordinator_capability_preflight_failure_has_zero_mutation() {
    let id = speech_id(20);
    let ack = Ack::for_request(&Request::new(id));
    let make = |capabilities: Vec<Capability>| {
        capabilities
            .into_iter()
            .map(|capability| Participant::new(capability, [ScriptedOutcome::Acknowledged(ack)]))
            .collect::<Vec<_>>()
    };
    let valid = |surface| Capability::cancellable(id, surface);
    let cases = [
        (
            vec![
                valid(Surface::Synthesis),
                valid(Surface::QueuedAudio),
                valid(Surface::Playback),
            ],
            CoordinatorError::MissingSurface,
        ),
        (
            vec![
                Capability {
                    contract_version: ProtocolVersion::new(2, 0),
                    ..valid(Surface::Synthesis)
                },
                valid(Surface::QueuedAudio),
                valid(Surface::Playback),
                valid(Surface::VisemeTimeline),
            ],
            CoordinatorError::UnsupportedVersion,
        ),
        (
            vec![
                Capability::cancellable(speech_id(21), Surface::Synthesis),
                valid(Surface::QueuedAudio),
                valid(Surface::Playback),
                valid(Surface::VisemeTimeline),
            ],
            CoordinatorError::AssociationMismatch,
        ),
        (
            vec![
                valid(Surface::Synthesis),
                valid(Surface::Synthesis),
                valid(Surface::Playback),
                valid(Surface::VisemeTimeline),
            ],
            CoordinatorError::DuplicateSurface,
        ),
        (
            vec![
                valid(Surface::Synthesis),
                Capability {
                    cancellable: false,
                    ..valid(Surface::QueuedAudio)
                },
                valid(Surface::Playback),
                valid(Surface::VisemeTimeline),
            ],
            CoordinatorError::NonCancellableSurface,
        ),
    ];
    for (capabilities, expected) in cases {
        let participants = make(capabilities);
        let refs = participants
            .iter()
            .map(|participant| participant as _)
            .collect::<Vec<&dyn SpeechCancellationParticipant>>();
        assert!(matches!(Coordinator::new(id, refs), Err(error) if error == expected));
        assert_untouched(&participants, 1);
    }
}

#[test]
fn coordinator_request_preflight_never_invokes_participants() {
    let id = speech_id(22);
    let request = Request::new(id);
    let ack = Ack::for_request(&request);
    let participants =
        Surface::ALL.map(|surface| participant(id, surface, ScriptedOutcome::Acknowledged(ack)));
    let refs = participants
        .iter()
        .map(|participant| participant as _)
        .collect::<Vec<&dyn SpeechCancellationParticipant>>();
    let coordinator = Coordinator::new(id, refs).unwrap();
    let mut unsupported = request;
    unsupported.contract_version = ProtocolVersion::new(2, 0);
    assert_eq!(
        complete(Box::pin(coordinator.cancel(unsupported)).as_mut()),
        Err(CoordinatorError::UnsupportedVersion)
    );
    assert_eq!(
        complete(Box::pin(coordinator.cancel(Request::new(speech_id(23)))).as_mut()),
        Err(CoordinatorError::AssociationMismatch)
    );
    assert_untouched(&participants, 1);
}

#[test]
fn coordinator_rejects_each_acknowledgement_mismatch_after_exact_activation() {
    let id = speech_id(24);
    for mismatch_version in [false, true] {
        let request = Request::new(id);
        let mut bad = Ack::for_request(&request);
        if mismatch_version {
            bad.contract_version = ProtocolVersion::new(2, 0);
        } else {
            bad.speech_id = speech_id(25);
        }
        let participants = Surface::ALL.map(|surface| {
            participant(
                id,
                surface,
                ScriptedOutcome::Acknowledged(if surface == Surface::Playback {
                    bad
                } else {
                    Ack::for_request(&request)
                }),
            )
        });
        let refs = participants
            .iter()
            .map(|participant| participant as _)
            .collect::<Vec<&dyn SpeechCancellationParticipant>>();
        let coordinator = Coordinator::new(id, refs).unwrap();
        assert_eq!(
            complete(Box::pin(coordinator.cancel(request)).as_mut()),
            Err(CoordinatorError::AcknowledgementMismatch)
        );
        assert!(participants
            .iter()
            .all(|participant| participant.received_requests() == [request]
                && participant.consumed_outcome_count() == 1
                && participant.remaining_outcome_count() == 0
                && participant.active_future_count() == 0));
    }
}

#[test]
fn coordinator_exhaustion_and_independent_runs_are_exact_and_deterministic() {
    let id = speech_id(26);
    let request = Request::new(id);
    let ack = Ack::for_request(&request);
    let run = || {
        let participants = [
            Surface::Playback,
            Surface::VisemeTimeline,
            Surface::Synthesis,
            Surface::QueuedAudio,
        ]
        .map(|surface| participant(id, surface, ScriptedOutcome::Acknowledged(ack)));
        let refs = participants
            .iter()
            .map(|participant| participant as _)
            .collect::<Vec<&dyn SpeechCancellationParticipant>>();
        let coordinator = Coordinator::new(id, refs).unwrap();
        let evidence = complete(Box::pin(coordinator.cancel(request)).as_mut()).unwrap();
        (evidence, participants)
    };
    let (first, first_participants) = run();
    let (second, second_participants) = run();
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
    for participants in [&first_participants, &second_participants] {
        assert!(participants
            .iter()
            .all(|participant| participant.received_requests() == [request]
                && participant.consumed_outcome_count() == 1
                && participant.remaining_outcome_count() == 0
                && participant.active_future_count() == 0));
    }
    let refs = first_participants
        .iter()
        .map(|participant| participant as _)
        .collect::<Vec<&dyn SpeechCancellationParticipant>>();
    let exhausted = Coordinator::new(id, refs).unwrap();
    assert_eq!(
        complete(Box::pin(exhausted.cancel(request)).as_mut()),
        Err(CoordinatorError::DependencyFailure)
    );
    assert!(first_participants
        .iter()
        .all(
            |participant| participant.received_requests() == [request, request]
                && participant.consumed_outcome_count() == 1
                && participant.remaining_outcome_count() == 0
                && participant.active_future_count() == 0
        ));
}

#[test]
fn composite_failure_and_drop_leave_no_active_work() {
    let id = speech_id(12);
    let participants = [
        participant(id, Surface::Synthesis, ScriptedOutcome::Pending),
        participant(id, Surface::QueuedAudio, ScriptedOutcome::Pending),
        participant(id, Surface::Playback, ScriptedOutcome::DependencyFailure),
        participant(id, Surface::VisemeTimeline, ScriptedOutcome::Pending),
    ];
    let refs: Vec<&dyn SpeechCancellationParticipant> =
        participants.iter().map(|p| p as _).collect();
    let coordinator = Coordinator::new(id, refs).unwrap();
    assert_eq!(
        complete(Box::pin(coordinator.cancel(Request::new(id))).as_mut()),
        Err(CoordinatorError::DependencyFailure)
    );
    assert!(participants.iter().all(|p| p.active_future_count() == 0));
    assert!(participants
        .iter()
        .all(|p| p.received_requests() == [Request::new(id)]
            && p.consumed_outcome_count() == 1
            && p.remaining_outcome_count() == 0));

    let pending = [
        participant(id, Surface::Synthesis, ScriptedOutcome::Pending),
        participant(id, Surface::QueuedAudio, ScriptedOutcome::Pending),
        participant(id, Surface::Playback, ScriptedOutcome::Pending),
        participant(id, Surface::VisemeTimeline, ScriptedOutcome::Pending),
    ];
    let refs: Vec<&dyn SpeechCancellationParticipant> = pending.iter().map(|p| p as _).collect();
    let coordinator = Coordinator::new(id, refs).unwrap();
    let mut future = Box::pin(coordinator.cancel(Request::new(id)));
    assert!(poll_once(future.as_mut()).is_pending());
    assert!(pending.iter().all(|p| p.active_future_count() == 1));
    drop(future);
    assert!(pending
        .iter()
        .all(|p| p.received_requests() == [Request::new(id)]
            && p.consumed_outcome_count() == 1
            && p.remaining_outcome_count() == 0
            && p.active_future_count() == 0));
}
