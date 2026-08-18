//! Renderer-neutral eye and head gaze calculation.

use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GazeLimits {
    pub head_yaw_limit_radians: f32,
    pub head_pitch_limit_radians: f32,
    pub eye_yaw_limit_radians: f32,
    pub eye_pitch_limit_radians: f32,
    /// Portion of the requested aim allocated to head motion before eye motion.
    pub head_share: f32,
}

impl Default for GazeLimits {
    fn default() -> Self {
        Self {
            head_yaw_limit_radians: 35_f32.to_radians(),
            head_pitch_limit_radians: 20_f32.to_radians(),
            eye_yaw_limit_radians: 28_f32.to_radians(),
            eye_pitch_limit_radians: 18_f32.to_radians(),
            head_share: 0.55,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GazePose {
    pub head_yaw_radians: f32,
    pub head_pitch_radians: f32,
    pub eye_yaw_radians: f32,
    pub eye_pitch_radians: f32,
}

/// Solves world-space target direction using glTF's conventional forward -Z
/// orientation. A target at the origin yields a neutral pose.
pub fn solve(origin: Vec3, target: Vec3, limits: GazeLimits) -> GazePose {
    let offset = target - origin;
    let horizontal = offset.x.hypot(offset.z);
    if offset.length_squared() <= f32::EPSILON {
        return GazePose {
            head_yaw_radians: 0.0,
            head_pitch_radians: 0.0,
            eye_yaw_radians: 0.0,
            eye_pitch_radians: 0.0,
        };
    }
    let yaw = offset.x.atan2(-offset.z);
    let pitch = offset.y.atan2(horizontal);
    let head_share = limits.head_share.clamp(0.0, 1.0);
    let head_yaw = (yaw * head_share).clamp(
        -limits.head_yaw_limit_radians,
        limits.head_yaw_limit_radians,
    );
    let head_pitch = (pitch * head_share).clamp(
        -limits.head_pitch_limit_radians,
        limits.head_pitch_limit_radians,
    );
    GazePose {
        head_yaw_radians: head_yaw,
        head_pitch_radians: head_pitch,
        eye_yaw_radians: (yaw - head_yaw)
            .clamp(-limits.eye_yaw_limit_radians, limits.eye_yaw_limit_radians),
        eye_pitch_radians: (pitch - head_pitch).clamp(
            -limits.eye_pitch_limit_radians,
            limits.eye_pitch_limit_radians,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_target_is_neutral() {
        let pose = solve(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), GazeLimits::default());
        assert_eq!(pose.head_yaw_radians, 0.0);
        assert_eq!(pose.eye_pitch_radians, 0.0);
    }

    #[test]
    fn extreme_target_respects_head_and_eye_limits() {
        let limits = GazeLimits::default();
        let pose = solve(Vec3::ZERO, Vec3::new(100.0, 100.0, 0.0), limits);
        assert!(pose.head_yaw_radians.abs() <= limits.head_yaw_limit_radians);
        assert!(pose.head_pitch_radians.abs() <= limits.head_pitch_limit_radians);
        assert!(pose.eye_yaw_radians.abs() <= limits.eye_yaw_limit_radians);
        assert!(pose.eye_pitch_radians.abs() <= limits.eye_pitch_limit_radians);
    }

    #[test]
    fn head_receives_the_requested_share_before_eyes() {
        let limits = GazeLimits {
            head_share: 0.5,
            ..Default::default()
        };
        let pose = solve(Vec3::ZERO, Vec3::new(1.0, 0.0, -1.0), limits);
        assert!((pose.head_yaw_radians - std::f32::consts::FRAC_PI_8).abs() < 0.0001);
        assert!((pose.eye_yaw_radians - std::f32::consts::FRAC_PI_8).abs() < 0.0001);
    }
}
