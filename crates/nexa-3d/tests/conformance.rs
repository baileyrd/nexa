use nexa_3d_runtime::{
    avatar::{
        AvatarRenderer, ExpressionCommand, GazeCommand, GestureCommand, NexaAvatarAdapter,
        VisemeCommand,
    },
    integration::{execute, FlowIdentity, LifecycleEvent},
};
use nexa_avatar::{AvatarCapabilities, AvatarCapability, FakeAvatarAdapter};
use nexa_domain::*;
use nexa_nbp::*;
use std::str::FromStr;

fn mid(n: u8) -> MessageId {
    format!("018f1f64-4f09-7cc0-98c2-7b3e8f2490{n:02}")
        .parse()
        .unwrap()
}
fn eid(n: u8) -> EventId {
    format!("018f1f64-4f09-7cc0-98c2-7b3e8f2480{n:02}")
        .parse()
        .unwrap()
}
fn behavior_id() -> BehaviorId {
    "018f1f64-4f09-7cc0-98c2-7b3e8f249001".parse().unwrap()
}
fn command(emotion: bool) -> BehaviorCommand {
    BehaviorCommand {
        behavior_id: behavior_id(),
        state: BehaviorState::Explaining,
        priority: Priority::default(),
        interruptibility: Interruptibility::Immediate,
        emotion: emotion.then_some(Emotion {
            preset: EmotionPreset::Focused,
            confidence: None,
            intensity: None,
        }),
        gaze: None,
        gesture: None,
        speech: None,
    }
}
fn input(payload: Payload, n: u8) -> NbpMessage {
    NbpMessage::new(
        ProtocolVersion::new(1, 0),
        mid(n),
        Timestamp::from_str("2026-08-18T12:00:00Z").unwrap(),
        SessionId::from_str("018f1f64-4f09-7cc0-98c2-7b3e8f249003").unwrap(),
        Sequence::new(41),
        EndpointId::new("nexa.orchestrator").unwrap(),
        Some(EndpointId::new("nexa.avatar").unwrap()),
        Some("018f1f64-4f09-7cc0-98c2-7b3e8f249004".parse().unwrap()),
        payload,
        Default::default(),
    )
    .unwrap()
}
fn identity() -> FlowIdentity {
    FlowIdentity {
        source: EndpointId::new("nexa.avatar").unwrap(),
        first_output_sequence: Sequence::new(90),
        output_message_ids: (10..20).map(mid).collect(),
        first_event_sequence: Sequence::new(200),
        event_ids: (10..20).map(eid).collect(),
    }
}

#[test]
fn supported_command_completes_with_correlated_outputs_events_and_stable_fixture() {
    let input = input(Payload::BehaviorCommand(command(false)), 2);
    let mut adapter = FakeAvatarAdapter::new(
        SemanticKey::new("headless-avatar").unwrap(),
        AvatarCapabilities::new([
            AvatarCapability::BehaviorState,
            AvatarCapability::Cancellation,
        ]),
    );
    let result = execute(&mut adapter, &input, identity()).unwrap();
    assert_eq!(
        result.report.lifecycle,
        [
            RuntimeStatus::Accepted,
            RuntimeStatus::Started,
            RuntimeStatus::Completed
        ]
    );
    assert!(matches!(
        result.events.as_slice(),
        [
            LifecycleEvent::Accepted(_),
            LifecycleEvent::Started(_),
            LifecycleEvent::Completed(_)
        ]
    ));
    for output in &result.outputs {
        assert_eq!(output.session_id, input.session_id);
        assert_eq!(output.correlation_id, input.correlation_id);
        match &output.payload {
            Payload::RuntimeAck(value) => {
                assert_eq!(value.message_id, input.message_id);
                assert_eq!(value.behavior_id, Some(behavior_id()));
            }
            Payload::RuntimeState(value) => {
                assert_eq!(value.message_id, input.message_id);
                assert_eq!(value.behavior_id, Some(behavior_id()));
            }
            _ => panic!("unexpected success output"),
        }
    }
    let json = serde_json::to_string_pretty(&result.outputs).unwrap();
    if std::env::var_os("NEXA_UPDATE_FIXTURES").is_some() {
        std::fs::write("tests/fixtures/success-outputs.json", &json).unwrap();
    }
    assert_eq!(json, include_str!("fixtures/success-outputs.json").trim());
}

#[test]
fn optional_capability_and_unresolved_semantic_target_degrade_truthfully() {
    let mut fake = FakeAvatarAdapter::new(
        SemanticKey::new("headless-avatar").unwrap(),
        AvatarCapabilities::new([AvatarCapability::BehaviorState]),
    );
    let result = execute(
        &mut fake,
        &input(Payload::BehaviorCommand(command(true)), 2),
        identity(),
    )
    .unwrap();
    assert_eq!(result.report.terminal_status(), RuntimeStatus::Degraded);
    assert!(matches!(
        result.events.as_slice(),
        [LifecycleEvent::Degraded(_)]
    ));

    let mut gaze_command = command(false);
    gaze_command.gaze = Some(Gaze {
        target_type: GazeTarget::CanvasObject,
        target_id: Some(SemanticKey::new("lesson.diagram.axis").unwrap()),
        intensity: Confidence::new(1.0).unwrap(),
        duration_ms: None,
        lead_time_ms: None,
    });
    let mut adapter = NexaAvatarAdapter::new(Recorder);
    let result = execute(
        &mut adapter,
        &input(Payload::BehaviorCommand(gaze_command), 2),
        identity(),
    )
    .unwrap();
    assert_eq!(result.report.terminal_status(), RuntimeStatus::Degraded);
    assert_eq!(
        result.report.error.unwrap().code.as_str(),
        "avatar.gaze.target_unresolved"
    );
}

#[test]
fn cancellation_and_output_only_rejection_are_explicit() {
    let cancel = BehaviorCancel {
        behavior_id: behavior_id(),
        reason: "new turn".into(),
        transition: CancellationMode::Immediate,
    };
    let mut fake = FakeAvatarAdapter::new(
        SemanticKey::new("headless-avatar").unwrap(),
        AvatarCapabilities::new([AvatarCapability::Cancellation]),
    );
    let result = execute(
        &mut fake,
        &input(Payload::BehaviorCancel(cancel), 5),
        identity(),
    )
    .unwrap();
    assert_eq!(result.report.terminal_status(), RuntimeStatus::Cancelled);
    assert!(matches!(
        result.events.as_slice(),
        [LifecycleEvent::Cancelled(_)]
    ));
    let output_only = input(
        Payload::RuntimeAck(RuntimeAck {
            message_id: mid(2),
            behavior_id: Some(behavior_id()),
            status: RuntimeStatus::Accepted,
        }),
        6,
    );
    assert!(execute(&mut fake, &output_only, identity()).is_err());
    let malformed = serde_json::to_string(&output_only)
        .unwrap()
        .replace("runtime.ack", "behavior.command");
    assert!(serde_json::from_str::<NbpMessage>(&malformed).is_err());
}

struct Recorder;
impl AvatarRenderer for Recorder {
    fn set_behavior_state(&mut self, _: BehaviorState) {}
    fn cancel_behavior(&mut self, _: &BehaviorCancel) {}
    fn set_expression(&mut self, _: ExpressionCommand) {}
    fn set_viseme(&mut self, _: VisemeCommand) {}
    fn set_gaze(&mut self, _: GazeCommand) {}
    fn play_gesture(&mut self, _: GestureCommand) {}
    fn set_animation_time(&mut self, _: f32) {}
}
