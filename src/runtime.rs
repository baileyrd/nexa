//! Bridges debug controls to the NEXA-3D-001 semantic renderer port.
use crate::{
    avatar::{AvatarRenderer, NexaAvatarAdapter},
    control::RuntimeControls,
};

/// Applies one frame of control intent. A renderer owns the final eye/head IK,
/// viseme blend weights, and animation mixing implementation.
pub fn apply_debug_controls<R: AvatarRenderer>(
    adapter: &mut NexaAvatarAdapter<R>,
    controls: &mut RuntimeControls,
) {
    if controls.gaze_enabled {
        adapter.apply_gaze(controls.gaze_target);
    }
    if let Some(viseme) = controls.pending_viseme.take() {
        adapter.apply_viseme(&viseme, 1.0, 0.12);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avatar::{ExpressionCommand, GazeCommand, GestureCommand, VisemeCommand};

    #[derive(Default)]
    struct Recorder {
        gaze: Option<GazeCommand>,
        viseme: Option<VisemeCommand>,
    }
    impl AvatarRenderer for Recorder {
        fn set_expression(&mut self, _: ExpressionCommand) {}
        fn set_viseme(&mut self, command: VisemeCommand) {
            self.viseme = Some(command);
        }
        fn set_gaze(&mut self, command: GazeCommand) {
            self.gaze = Some(command);
        }
        fn play_gesture(&mut self, _: GestureCommand) {}
        fn set_animation_time(&mut self, _: f32) {}
    }

    #[test]
    fn gaze_and_one_shot_viseme_are_forwarded_without_gpu() {
        let mut controls = RuntimeControls::default();
        controls.trigger_viseme("MBP");
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        apply_debug_controls(&mut adapter, &mut controls);
        let recorder = adapter.into_inner();
        assert_eq!(recorder.viseme.unwrap().canonical_name, "MBP");
        assert!(recorder.gaze.is_some());
        assert!(controls.pending_viseme.is_none());
    }
}
