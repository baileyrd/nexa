//! Renderer-neutral animation pose sampling.
//!
//! The GLB importer adapts exported channels into these values; render backends
//! consume the sampled pose without taking a dependency on `wgpu` or `winit`.

use std::collections::BTreeMap;

use glam::{Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Step,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelValues {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
    MorphWeight(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub node_index: usize,
    pub interpolation: Interpolation,
    pub times_seconds: Vec<f32>,
    pub values: Vec<ChannelValues>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pose {
    pub nodes: BTreeMap<usize, NodeTransform>,
    pub morph_weights: BTreeMap<usize, f32>,
}

impl Channel {
    /// Samples a single glTF-compatible channel. Malformed empty or mismatched
    /// channels are ignored so the debug viewer remains resilient to bad assets.
    pub fn sample(&self, time_seconds: f32) -> Option<ChannelValues> {
        let count = self.times_seconds.len().min(self.values.len());
        if count == 0 {
            return None;
        }
        let time = time_seconds.max(0.0);
        let right = self.times_seconds[..count].partition_point(|key| *key <= time);
        if right == 0 {
            return Some(self.values[0]);
        }
        if right == count || self.interpolation == Interpolation::Step {
            return Some(self.values[right - 1]);
        }
        let left = right - 1;
        let start = self.times_seconds[left];
        let end = self.times_seconds[right];
        let amount = if end > start {
            ((time - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        interpolate(self.values[left], self.values[right], amount)
    }
}

pub fn sample_pose(channels: &[Channel], time_seconds: f32) -> Pose {
    let mut pose = Pose::default();
    for channel in channels {
        let Some(value) = channel.sample(time_seconds) else {
            continue;
        };
        match value {
            ChannelValues::Translation(translation) => {
                pose.nodes
                    .entry(channel.node_index)
                    .or_default()
                    .translation = translation;
            }
            ChannelValues::Rotation(rotation) => {
                pose.nodes.entry(channel.node_index).or_default().rotation = rotation;
            }
            ChannelValues::Scale(scale) => {
                pose.nodes.entry(channel.node_index).or_default().scale = scale;
            }
            ChannelValues::MorphWeight(weight) => {
                pose.morph_weights.insert(channel.node_index, weight);
            }
        }
    }
    pose
}

fn interpolate(left: ChannelValues, right: ChannelValues, amount: f32) -> Option<ChannelValues> {
    match (left, right) {
        (ChannelValues::Translation(a), ChannelValues::Translation(b)) => {
            Some(ChannelValues::Translation(a.lerp(b, amount)))
        }
        (ChannelValues::Rotation(a), ChannelValues::Rotation(b)) => {
            Some(ChannelValues::Rotation(a.slerp(b, amount)))
        }
        (ChannelValues::Scale(a), ChannelValues::Scale(b)) => {
            Some(ChannelValues::Scale(a.lerp(b, amount)))
        }
        (ChannelValues::MorphWeight(a), ChannelValues::MorphWeight(b)) => {
            Some(ChannelValues::MorphWeight(a + (b - a) * amount))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_translation_interpolates_between_keys() {
        let channel = Channel {
            node_index: 7,
            interpolation: Interpolation::Linear,
            times_seconds: vec![0.0, 2.0],
            values: vec![
                ChannelValues::Translation(Vec3::ZERO),
                ChannelValues::Translation(Vec3::new(4.0, 2.0, 0.0)),
            ],
        };
        let pose = sample_pose(&[channel], 0.5);
        assert_eq!(pose.nodes[&7].translation, Vec3::new(1.0, 0.5, 0.0));
    }

    #[test]
    fn step_channels_hold_the_previous_value() {
        let channel = Channel {
            node_index: 2,
            interpolation: Interpolation::Step,
            times_seconds: vec![0.0, 1.0],
            values: vec![
                ChannelValues::MorphWeight(0.2),
                ChannelValues::MorphWeight(0.9),
            ],
        };
        let pose = sample_pose(&[channel], 0.9);
        assert_eq!(pose.morph_weights[&2], 0.2);
    }

    #[test]
    fn rotation_sampling_uses_a_normalized_slerp() {
        let channel = Channel {
            node_index: 4,
            interpolation: Interpolation::Linear,
            times_seconds: vec![0.0, 1.0],
            values: vec![
                ChannelValues::Rotation(Quat::IDENTITY),
                ChannelValues::Rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ],
        };
        let pose = sample_pose(&[channel], 0.5);
        assert!(pose.nodes[&4].rotation.is_normalized());
        let facing = pose.nodes[&4].rotation.mul_vec3(Vec3::Z);
        assert!(facing.x.abs() > 0.9999, "facing was {facing:?}");
        assert!(facing.y.abs() < 0.0001 && facing.z.abs() < 0.0001);
    }
}
