//! Renderer-independent behavior events consumed by the Nexa avatar adapter.
use glam::Vec3;

use crate::{
    avatar::{AvatarRenderer, NexaAvatarAdapter},
    avatar::{ExpressionCommand, GestureCommand, VisemeCommand},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AvatarBehaviorEvent {
    Expression(ExpressionCommand),
    Gesture(GestureCommand),
    Gaze { target: Vec3 },
    Viseme(VisemeCommand),
}

pub fn dispatch<R: AvatarRenderer>(adapter: &mut NexaAvatarAdapter<R>, event: AvatarBehaviorEvent) {
    match event {
        AvatarBehaviorEvent::Expression(command) => {
            adapter.apply_expression(&command.canonical_name, command.weight)
        }
        AvatarBehaviorEvent::Gesture(command) => {
            adapter.apply_gesture(&command.canonical_name, command.intensity)
        }
        AvatarBehaviorEvent::Gaze { target } => adapter.apply_gaze(target),
        AvatarBehaviorEvent::Viseme(command) => adapter.apply_viseme(
            &command.canonical_name,
            command.weight,
            command.duration_seconds,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avatar::GazeCommand;

    #[derive(Default)]
    struct Recorder {
        events: Vec<&'static str>,
    }
    impl AvatarRenderer for Recorder {
        fn set_behavior_state(&mut self, _: nexa_nbp::BehaviorState) {}
        fn cancel_behavior(&mut self, _: &nexa_nbp::BehaviorCancel) {}
        fn set_expression(&mut self, _: ExpressionCommand) {
            self.events.push("expression");
        }
        fn set_viseme(&mut self, _: VisemeCommand) {
            self.events.push("viseme");
        }
        fn set_gaze(&mut self, _: GazeCommand) {
            self.events.push("gaze");
        }
        fn play_gesture(&mut self, _: GestureCommand) {
            self.events.push("gesture");
        }
        fn set_animation_time(&mut self, _: f32) {}
    }

    #[test]
    fn semantic_events_dispatch_without_a_render_backend() {
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        dispatch(
            &mut adapter,
            AvatarBehaviorEvent::Expression(ExpressionCommand {
                canonical_name: "Focused".into(),
                weight: 0.8,
            }),
        );
        dispatch(
            &mut adapter,
            AvatarBehaviorEvent::Gesture(GestureCommand {
                canonical_name: "Point_Right".into(),
                intensity: 1.0,
            }),
        );
        dispatch(
            &mut adapter,
            AvatarBehaviorEvent::Gaze { target: Vec3::ZERO },
        );
        assert_eq!(
            adapter.into_inner().events,
            ["expression", "gesture", "gaze"]
        );
    }
}
