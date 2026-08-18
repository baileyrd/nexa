use nexa_avatar::{
    missing_required_capabilities, required_capabilities, AvatarCapabilities, AvatarCapability,
    AvatarPort, AvatarRequest, FakeAvatarAdapter,
};
use nexa_domain::{
    BehaviorId, EndpointId, MessageId, ProtocolVersion, SemanticKey, Sequence, SessionId, Timestamp,
};
use nexa_nbp::{
    BehaviorCancel, BehaviorCommand, BehaviorState, CancellationMode, Emotion, EmotionPreset,
    Interruptibility, NbpMessage, Payload, Priority, RuntimeStatus,
};
use std::collections::BTreeMap;
use std::str::FromStr;

fn behavior_id() -> BehaviorId {
    BehaviorId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249001").unwrap()
}

fn message_id() -> MessageId {
    MessageId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249002").unwrap()
}

fn command() -> BehaviorCommand {
    BehaviorCommand {
        behavior_id: behavior_id(),
        state: BehaviorState::Attentive,
        priority: Priority::default(),
        interruptibility: Interruptibility::Immediate,
        emotion: Some(Emotion {
            preset: EmotionPreset::Focused,
            confidence: None,
            intensity: None,
        }),
        gaze: None,
        gesture: None,
        speech: None,
    }
}

fn fake(capabilities: impl IntoIterator<Item = AvatarCapability>) -> FakeAvatarAdapter {
    FakeAvatarAdapter::new(
        SemanticKey::new("headless-avatar").unwrap(),
        AvatarCapabilities::new(capabilities),
    )
}

#[test]
fn behavior_command_converts_to_avatar_request() {
    let message = NbpMessage::new(
        ProtocolVersion::new(1, 0),
        message_id(),
        Timestamp::from_str("2026-08-18T12:00:00Z").unwrap(),
        SessionId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249003").unwrap(),
        Sequence::new(1),
        EndpointId::new("tutor").unwrap(),
        Some(EndpointId::new("avatar").unwrap()),
        None,
        Payload::BehaviorCommand(command()),
        BTreeMap::new(),
    )
    .unwrap();
    let request = AvatarRequest::try_from(&message).unwrap();
    assert_eq!(request.message_id(), message_id());
    assert!(matches!(request, AvatarRequest::Submit { .. }));
}

#[test]
fn cancellation_propagates_to_the_adapter_and_acknowledgement() {
    let mut adapter = fake([AvatarCapability::Cancellation]);
    let cancellation = BehaviorCancel {
        behavior_id: behavior_id(),
        reason: "new turn".into(),
        transition: CancellationMode::Graceful,
    };
    let report = adapter.handle(AvatarRequest::Cancel {
        message_id: message_id(),
        cancellation: cancellation.clone(),
    });
    assert_eq!(report.terminal_status(), RuntimeStatus::Cancelled);
    assert_eq!(
        adapter.requests(),
        &[AvatarRequest::Cancel {
            message_id: message_id(),
            cancellation
        }]
    );
}

#[test]
fn acknowledgement_and_state_map_without_renderer_values() {
    let mut adapter = fake([AvatarCapability::BehaviorState, AvatarCapability::Emotion]);
    let report = adapter.submit(message_id(), command());
    assert_eq!(report.terminal_status(), RuntimeStatus::Completed);
    assert_eq!(report.state().unwrap().state, BehaviorState::Attentive);
    assert!(report.error().is_none());
}

#[test]
fn unsupported_capability_is_explicitly_degraded_with_recoverable_error() {
    let mut adapter = fake([AvatarCapability::BehaviorState]);
    let report = adapter.submit(message_id(), command());
    assert_eq!(report.terminal_status(), RuntimeStatus::Degraded);
    let error = report.error().unwrap();
    assert!(error.recoverable);
    assert_eq!(error.code.as_str(), "avatar.capability.unsupported");
}

#[test]
fn mandatory_behavior_state_is_checked_with_optional_capabilities() {
    let mut adapter = fake([AvatarCapability::Emotion]);
    let report = adapter.submit(message_id(), command());
    assert_eq!(report.terminal_status(), RuntimeStatus::Degraded);
    assert!(report.error().unwrap().message.contains("BehaviorState"));
}

#[test]
fn fake_adapter_is_deterministic_and_records_semantic_inputs() {
    let mut first = fake([AvatarCapability::BehaviorState, AvatarCapability::Emotion]);
    let mut second = first.clone();
    let first_report = first.submit(message_id(), command());
    let second_report = second.submit(message_id(), command());
    assert_eq!(first_report, second_report);
    assert_eq!(first.requests(), second.requests());
}

#[test]
fn avatar_requests_and_reports_round_trip_as_json() {
    let request = AvatarRequest::Submit {
        message_id: message_id(),
        command: command(),
    };
    let encoded = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<AvatarRequest>(&encoded).unwrap(),
        request
    );

    let mut adapter = fake([AvatarCapability::BehaviorState, AvatarCapability::Emotion]);
    let report = adapter.handle(request);
    let encoded = serde_json::to_string(&report).unwrap();
    assert_eq!(
        serde_json::from_str::<nexa_avatar::AvatarReport>(&encoded).unwrap(),
        report
    );
}

#[test]
fn required_capabilities_are_derived_only_from_semantic_fields() {
    let capabilities = required_capabilities(&command());
    assert_eq!(
        capabilities.iter().collect::<Vec<_>>(),
        vec![AvatarCapability::BehaviorState, AvatarCapability::Emotion]
    );
}

#[test]
fn visemes_are_required_only_when_speech_requests_emission() {
    use nexa_nbp::{Speech, SpeechStyle};

    let mut without_visemes = command();
    without_visemes.emotion = None;
    without_visemes.speech = Some(Speech {
        text: "Hello".into(),
        style: SpeechStyle::Instructional,
        allow_interruption: true,
        emit_visemes: false,
    });
    assert_eq!(
        required_capabilities(&without_visemes)
            .iter()
            .collect::<Vec<_>>(),
        [AvatarCapability::BehaviorState, AvatarCapability::Speech]
    );

    let mut with_visemes = without_visemes.clone();
    with_visemes.speech.as_mut().unwrap().emit_visemes = true;
    assert_eq!(
        required_capabilities(&with_visemes)
            .iter()
            .collect::<Vec<_>>(),
        [
            AvatarCapability::BehaviorState,
            AvatarCapability::Speech,
            AvatarCapability::Visemes,
        ]
    );
    let missing = missing_required_capabilities(
        &with_visemes,
        &AvatarCapabilities::new([AvatarCapability::BehaviorState, AvatarCapability::Speech]),
    );
    assert_eq!(
        missing.iter().collect::<Vec<_>>(),
        [AvatarCapability::Visemes]
    );
}

#[test]
fn malformed_reports_are_rejected_during_deserialization() {
    let mut adapter = fake([AvatarCapability::BehaviorState, AvatarCapability::Emotion]);
    let valid = serde_json::to_value(adapter.submit(message_id(), command())).unwrap();

    for lifecycle in [serde_json::json!([]), serde_json::json!(["degraded"])] {
        let mut malformed = valid.clone();
        malformed["lifecycle"] = lifecycle;
        if malformed["lifecycle"] == serde_json::json!(["degraded"]) {
            malformed["state"] = serde_json::Value::Null;
            malformed.as_object_mut().unwrap().remove("error");
        }
        assert!(serde_json::from_value::<nexa_avatar::AvatarReport>(malformed).is_err());
    }
}

#[test]
fn output_sequence_overflow_is_structured() {
    let input = NbpMessage::new(
        ProtocolVersion::new(1, 0),
        message_id(),
        Timestamp::from_str("2026-08-18T12:00:00Z").unwrap(),
        SessionId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249003").unwrap(),
        Sequence::new(1),
        EndpointId::new("tutor").unwrap(),
        None,
        None,
        Payload::BehaviorCommand(command()),
        BTreeMap::new(),
    )
    .unwrap();
    let report = fake([AvatarCapability::BehaviorState, AvatarCapability::Emotion])
        .submit(message_id(), command());
    let error = report
        .to_nbp_messages(
            &input,
            EndpointId::new("avatar").unwrap(),
            Sequence::new(u64::MAX),
            [message_id(), message_id(), message_id(), message_id()],
        )
        .unwrap_err();
    assert!(matches!(
        error,
        nexa_avatar::OutputConversionError::SequenceOverflow
    ));
}
