//! Bridges debug controls to the NEXA-3D-001 semantic renderer port.
use crate::{
    avatar::{AvatarRenderer, NexaAvatarAdapter},
    control::RuntimeControls,
    viseme::{VisemeCue, VisemePlayer},
};

/// Time-aware behavior state kept by a host or game loop. It has no window or
/// GPU dependency, making viseme playback deterministic in headless tests.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DebugBehaviorState {
    pub visemes: VisemePlayer,
}

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

/// Applies controls at a deterministic timeline time, scheduling any newly
/// requested viseme then emitting the active blendshape weights for this frame.
pub fn apply_debug_controls_at<R: AvatarRenderer>(
    adapter: &mut NexaAvatarAdapter<R>,
    controls: &mut RuntimeControls,
    state: &mut DebugBehaviorState,
    time_seconds: f32,
) {
    if controls.gaze_enabled {
        adapter.apply_gaze(controls.gaze_target);
    }
    if let Some(viseme) = controls.pending_viseme.take() {
        state.visemes.schedule(VisemeCue {
            canonical_name: viseme,
            start_seconds: time_seconds,
            duration_seconds: 0.12,
            peak_weight: 1.0,
        });
    }
    for weight in state.visemes.sample(time_seconds) {
        adapter.apply_viseme(&weight.canonical_name, weight.weight, 0.0);
    }
    state.visemes.discard_finished(time_seconds);
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
        fn set_behavior_state(&mut self, _: nexa_nbp::BehaviorState) {}
        fn cancel_behavior(&mut self, _: &nexa_nbp::BehaviorCancel) {}
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

    #[test]
    fn scheduled_viseme_emits_envelope_weights_without_gpu() {
        let mut controls = RuntimeControls::default();
        controls.trigger_viseme("MBP");
        let mut adapter = NexaAvatarAdapter::new(Recorder::default());
        let mut state = DebugBehaviorState::default();
        apply_debug_controls_at(&mut adapter, &mut controls, &mut state, 0.0);
        apply_debug_controls_at(&mut adapter, &mut controls, &mut state, 0.03);
        let recorder = adapter.into_inner();
        let viseme = recorder.viseme.unwrap();
        assert_eq!(viseme.canonical_name, "MBP");
        assert!(viseme.weight > 0.9);
        assert_eq!(viseme.duration_seconds, 0.0);
    }
}
