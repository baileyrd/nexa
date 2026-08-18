use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    animation::{Channel, ChannelValues},
    skin::VertexSkin,
};

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("could not import glTF/GLB `{path}`: {source}")]
    Import { path: PathBuf, source: gltf::Error },
    #[error("Nexa asset has no skinned skeleton")]
    MissingSkeleton,
    #[error("Nexa asset has no morph targets; facial/viseme validation cannot proceed")]
    MissingMorphTargets,
    #[error("Nexa asset has no renderable scene geometry")]
    MissingGeometry,
    #[error("mesh primitive {mesh}:{primitive} has no POSITION attribute")]
    MissingPosition { mesh: usize, primitive: usize },
    #[error("invalid glTF animation data: {detail}")]
    InvalidAnimationData { detail: String },
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub emission: [f32; 3],
}

#[derive(Debug, Default)]
pub struct StaticGeometry {
    pub vertices: Vec<StaticVertex>,
    pub indices: Vec<u32>,
    pub morph_position_deltas: Vec<Vec<[f32; 3]>>,
    /// One entry per vertex, always parallel to `vertices`. Vertices from
    /// unskinned primitives carry zero weights.
    pub vertex_skins: Vec<VertexSkin>,
    /// The glTF skin driving `vertex_skins`. Assets with several skinned meshes
    /// need a palette per draw call, which this single-buffer debug viewer does
    /// not do, so only the first skin encountered is recorded.
    pub skin_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub minimum: glam::Vec3,
    pub maximum: glam::Vec3,
}

impl Bounds {
    pub fn center(self) -> glam::Vec3 {
        (self.minimum + self.maximum) * 0.5
    }
    pub fn extent(self) -> glam::Vec3 {
        self.maximum - self.minimum
    }
}

impl StaticGeometry {
    pub fn bounds(&self) -> Option<Bounds> {
        let first = self.vertices.first()?;
        let (minimum, maximum) = self.vertices.iter().skip(1).fold(
            (
                glam::Vec3::from(first.position),
                glam::Vec3::from(first.position),
            ),
            |(minimum, maximum), vertex| {
                let position = glam::Vec3::from(vertex.position);
                (minimum.min(position), maximum.max(position))
            },
        );
        Some(Bounds { minimum, maximum })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkinReport {
    pub name: String,
    pub joint_count: usize,
    pub joint_names: Vec<String>,
    pub root_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MorphTargetReport {
    /// Stable diagnostic identifier; map it to semantic face names in the runtime manifest.
    pub id: String,
    pub mesh_name: String,
    pub primitive_index: usize,
    pub target_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationReport {
    pub name: String,
    pub channel_count: usize,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeReport {
    pub index: usize,
    pub name: String,
    pub has_mesh: bool,
    pub is_joint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetReport {
    pub source: PathBuf,
    pub node_count: usize,
    pub mesh_count: usize,
    pub primitive_count: usize,
    pub morph_target_count: usize,
    pub morph_targets: Vec<MorphTargetReport>,
    pub nodes: Vec<NodeReport>,
    pub skins: Vec<SkinReport>,
    pub animations: Vec<AnimationReport>,
}

impl AssetReport {
    pub fn has_skeleton(&self) -> bool {
        !self.skins.is_empty()
    }
    pub fn has_morph_targets(&self) -> bool {
        self.morph_target_count > 0
    }
    pub fn animation_names(&self) -> impl Iterator<Item = &str> {
        self.animations.iter().map(|a| a.name.as_str())
    }
}

/// Import only metadata. This is intentionally suitable for CI/headless usage.
pub fn inspect(path: impl AsRef<Path>) -> Result<AssetReport, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import {
            path: path.clone(),
            source,
        })?;

    let mut primitive_count = 0;
    let mut morph_targets = Vec::new();
    for (mesh_index, mesh) in document.meshes().enumerate() {
        for primitive in mesh.primitives() {
            primitive_count += 1;
            for (target_index, _) in primitive.morph_targets().enumerate() {
                let mesh_name = mesh
                    .name()
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("mesh_{mesh_index}"));
                morph_targets.push(MorphTargetReport {
                    id: format!(
                        "{mesh_name}/primitive_{}/target_{target_index}",
                        primitive.index()
                    ),
                    mesh_name,
                    primitive_index: primitive.index(),
                    target_index,
                });
            }
        }
    }
    let joint_indices: HashSet<usize> = document
        .skins()
        .flat_map(|skin| skin.joints().map(|joint| joint.index()))
        .collect();
    let nodes = document
        .nodes()
        .map(|node| NodeReport {
            index: node.index(),
            name: node
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("node_{}", node.index())),
            has_mesh: node.mesh().is_some(),
            is_joint: joint_indices.contains(&node.index()),
        })
        .collect();
    let skins = document
        .skins()
        .map(|skin| SkinReport {
            name: skin.name().unwrap_or("<unnamed-skin>").to_owned(),
            joint_count: skin.joints().count(),
            joint_names: skin
                .joints()
                .map(|joint| joint.name().unwrap_or("<unnamed-joint>").to_owned())
                .collect(),
            root_names: skin
                .skeleton()
                .map(|root| vec![root.name().unwrap_or("<unnamed-root>").to_owned()])
                .unwrap_or_default(),
        })
        .collect();
    let animations = document
        .animations()
        .enumerate()
        .map(|(index, animation)| AnimationReport {
            name: animation
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("animation_{index}")),
            channel_count: animation.channels().count(),
            duration_seconds: animation
                .channels()
                .filter_map(|channel| {
                    channel
                        .reader(|buffer| Some(&buffers[buffer.index()]))
                        .read_inputs()
                        .and_then(Iterator::last)
                })
                .fold(0.0, f32::max),
        })
        .collect();
    Ok(AssetReport {
        source: path,
        node_count: document.nodes().count(),
        mesh_count: document.meshes().count(),
        primitive_count,
        morph_target_count: morph_targets.len(),
        morph_targets,
        nodes,
        skins,
        animations,
    })
}

pub fn validate_first_model(report: &AssetReport) -> Result<(), AssetError> {
    if !report.has_skeleton() {
        return Err(AssetError::MissingSkeleton);
    }
    if !report.has_morph_targets() {
        return Err(AssetError::MissingMorphTargets);
    }
    Ok(())
}

pub fn validate_render_geometry(geometry: &StaticGeometry) -> Result<(), AssetError> {
    if geometry.vertices.is_empty() || geometry.indices.is_empty() {
        return Err(AssetError::MissingGeometry);
    }
    Ok(())
}

/// Load a rest-pose render mesh. Animation, skinning, and morph mixing remain
/// renderer work and intentionally do not leak into this asset-inspection API.
pub fn load_static_geometry(path: impl AsRef<Path>) -> Result<StaticGeometry, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import {
            path: path.clone(),
            source,
        })?;
    let mut geometry = StaticGeometry::default();
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next());
    if let Some(scene) = scene {
        for node in scene.nodes() {
            append_node(&mut geometry, &buffers, node, glam::Mat4::IDENTITY)?;
        }
    }
    Ok(geometry)
}

/// Load rest-pose joint connections as line-list vertices for debug rendering.
pub fn load_skeleton_debug_geometry(
    path: impl AsRef<Path>,
) -> Result<Vec<StaticVertex>, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, _buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import {
            path: path.clone(),
            source,
        })?;
    let joints: HashSet<usize> = document
        .skins()
        .flat_map(|skin| skin.joints().map(|joint| joint.index()))
        .collect();
    let mut vertices = Vec::new();
    if let Some(scene) = document
        .default_scene()
        .or_else(|| document.scenes().next())
    {
        for node in scene.nodes() {
            append_skeleton_lines(
                &mut vertices,
                &joints,
                node,
                glam::Mat4::IDENTITY,
                None,
                None,
            );
        }
    }
    Ok(vertices)
}

/// Load joint lines after applying a sampled animation pose. This remains a
/// diagnostic-only path: mesh skinning stays renderer-specific.
pub fn load_animated_skeleton_debug_geometry(
    path: impl AsRef<Path>,
    channels: &[Channel],
    time_seconds: f32,
) -> Result<Vec<StaticVertex>, AssetError> {
    let path = path.as_ref().to_path_buf();
    let (document, _buffers, _images) =
        gltf::import(&path).map_err(|source| AssetError::Import { path, source })?;
    let joints: HashSet<usize> = document
        .skins()
        .flat_map(|skin| skin.joints().map(|joint| joint.index()))
        .collect();
    let mut vertices = Vec::new();
    if let Some(scene) = document
        .default_scene()
        .or_else(|| document.scenes().next())
    {
        for node in scene.nodes() {
            append_skeleton_lines(
                &mut vertices,
                &joints,
                node,
                glam::Mat4::IDENTITY,
                None,
                Some((channels, time_seconds)),
            );
        }
    }
    Ok(vertices)
}

fn append_skeleton_lines(
    vertices: &mut Vec<StaticVertex>,
    joints: &HashSet<usize>,
    node: gltf::Node<'_>,
    parent_transform: glam::Mat4,
    closest_parent_joint: Option<glam::Vec3>,
    animation: Option<(&[Channel], f32)>,
) {
    let world_transform = parent_transform * animated_local_transform(&node, animation);
    let position = world_transform.transform_point3(glam::Vec3::ZERO);
    let is_joint = joints.contains(&node.index());
    if is_joint {
        if let Some(parent) = closest_parent_joint {
            let color = [0.08, 0.85, 1.0, 1.0];
            vertices.push(StaticVertex {
                position: parent.to_array(),
                normal: [0.0, 1.0, 0.0],
                color,
                emission: [0.0; 3],
            });
            vertices.push(StaticVertex {
                position: position.to_array(),
                normal: [0.0, 1.0, 0.0],
                color,
                emission: [0.0; 3],
            });
        }
    }
    let parent_joint = if is_joint {
        Some(position)
    } else {
        closest_parent_joint
    };
    for child in node.children() {
        append_skeleton_lines(
            vertices,
            joints,
            child,
            world_transform,
            parent_joint,
            animation,
        );
    }
}

fn animated_local_transform(
    node: &gltf::Node<'_>,
    animation: Option<(&[Channel], f32)>,
) -> glam::Mat4 {
    let (translation, rotation, scale) = node.transform().decomposed();
    let mut translation = glam::Vec3::from(translation);
    let mut rotation = glam::Quat::from_array(rotation);
    let mut scale = glam::Vec3::from(scale);
    if let Some((channels, time_seconds)) = animation {
        for channel in channels
            .iter()
            .filter(|channel| channel.node_index == node.index())
        {
            match channel.sample(time_seconds) {
                Some(ChannelValues::Translation(value)) => translation = value,
                Some(ChannelValues::Rotation(value)) => rotation = value,
                Some(ChannelValues::Scale(value)) => scale = value,
                None => {}
            }
        }
    }
    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

fn append_node(
    geometry: &mut StaticGeometry,
    buffers: &[gltf::buffer::Data],
    node: gltf::Node<'_>,
    parent_transform: glam::Mat4,
) -> Result<(), AssetError> {
    let matrix = node.transform().matrix();
    let local_transform = glam::Mat4::from_cols(
        glam::Vec4::from_array(matrix[0]),
        glam::Vec4::from_array(matrix[1]),
        glam::Vec4::from_array(matrix[2]),
        glam::Vec4::from_array(matrix[3]),
    );
    let world_transform = parent_transform * local_transform;
    if let Some(mesh) = node.mesh() {
        // Skinned vertices stay in the mesh node's local space: the joint
        // palette maps them straight to scene space, so baking the node's world
        // transform in would apply it twice.
        let skin = node.skin();
        let vertex_transform = if skin.is_some() {
            glam::Mat4::IDENTITY
        } else {
            world_transform
        };
        let normal_transform = vertex_transform.inverse().transpose();
        if let Some(skin) = &skin {
            geometry.skin_index.get_or_insert(skin.index());
        }
        for primitive in mesh.primitives() {
            let color = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            let emission = primitive.material().emissive_factor();
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or(AssetError::MissingPosition {
                    mesh: mesh.index(),
                    primitive: primitive.index(),
                })?
                .collect();
            let normals = reader.read_normals().map(Iterator::collect::<Vec<_>>);
            let base = geometry.vertices.len() as u32;
            let local_deltas: Vec<Vec<[f32; 3]>> = reader
                .read_morph_targets()
                .map(|(target_positions, _, _)| {
                    target_positions
                        .map(Iterator::collect)
                        .unwrap_or_else(|| vec![[0.0; 3]; positions.len()])
                })
                .collect();
            while geometry.morph_position_deltas.len() < local_deltas.len() {
                geometry
                    .morph_position_deltas
                    .push(vec![[0.0; 3]; base as usize]);
            }
            for (target_index, target) in geometry.morph_position_deltas.iter_mut().enumerate() {
                let local = local_deltas.get(target_index);
                target.extend((0..positions.len()).map(|index| {
                    local
                        .and_then(|delta| delta.get(index))
                        .map(|delta| {
                            vertex_transform
                                .transform_vector3(glam::Vec3::from(*delta))
                                .to_array()
                        })
                        .unwrap_or([0.0; 3])
                }));
            }
            match normals {
                Some(normals) => geometry.vertices.extend(positions.iter().zip(normals).map(
                    |(position, normal)| {
                        StaticVertex {
                            position: vertex_transform
                                .transform_point3(glam::Vec3::from(*position))
                                .to_array(),
                            normal: normal_transform
                                .transform_vector3(glam::Vec3::from(normal))
                                .normalize_or_zero()
                                .to_array(),
                            color,
                            emission,
                        }
                    },
                )),
                None => geometry.vertices.extend(positions.iter().map(|position| {
                    StaticVertex {
                        position: vertex_transform
                            .transform_point3(glam::Vec3::from(*position))
                            .to_array(),
                        normal: [0.0, 1.0, 0.0],
                        color,
                        emission,
                    }
                })),
            }
            let joints = reader
                .read_joints(0)
                .map(|joints| joints.into_u16().collect::<Vec<_>>());
            let weights = reader
                .read_weights(0)
                .map(|weights| weights.into_f32().collect::<Vec<_>>());
            geometry
                .vertex_skins
                .extend((0..positions.len()).map(|index| {
                    match (&joints, &weights) {
                        (Some(joints), Some(weights)) => VertexSkin {
                            joints: joints
                                .get(index)
                                .map(|joints| joints.map(u32::from))
                                .unwrap_or_default(),
                            weights: weights.get(index).copied().unwrap_or_default(),
                        },
                        _ => VertexSkin::default(),
                    }
                }));
            if let Some(indices) = reader.read_indices() {
                geometry
                    .indices
                    .extend(indices.into_u32().map(|index| base + index));
            } else {
                geometry
                    .indices
                    .extend(base..geometry.vertices.len() as u32);
            }
        }
    }
    for child in node.children() {
        append_node(geometry, buffers, child, world_transform)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{animation::load_animation_clip, test_fixtures::skinned_rig_gltf};

    #[test]
    fn skinned_vertices_import_unbaked_with_their_joint_bindings() {
        let directory = tempfile::tempdir().unwrap();
        let geometry = load_static_geometry(skinned_rig_gltf(directory.path())).unwrap();
        assert_eq!(geometry.skin_index, Some(0));
        assert_eq!(geometry.vertex_skins.len(), geometry.vertices.len());
        // The mesh node sits at x=10, but the joint palette will place these
        // vertices, so the node transform must not be baked into them.
        assert_eq!(geometry.vertices[0].position, [0.0, 1.5, 0.0]);
        let skin = geometry.vertex_skins[0];
        assert!(skin.is_skinned());
        assert_eq!(skin.joints, [1, 0, 0, 0]);
        assert_eq!(skin.weights, [1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn unskinned_vertices_stay_baked_to_world_space_and_carry_no_weights() {
        let directory = tempfile::tempdir().unwrap();
        let gltf_path = directory.path().join("static.gltf");
        std::fs::write(directory.path().join("static.bin"), {
            let mut bytes = Vec::new();
            for value in [0.0_f32, 1.5, 0.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes
        })
        .unwrap();
        std::fs::write(
            &gltf_path,
            r#"{
              "asset":{"version":"2.0"},
              "buffers":[{"uri":"static.bin","byteLength":12}],
              "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":12}],
              "accessors":[{"bufferView":0,"componentType":5126,"count":1,"type":"VEC3","min":[0,1.5,0],"max":[0,1.5,0]}],
              "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],
              "nodes":[{"name":"Prop","mesh":0,"translation":[10,0,0]}],
              "scenes":[{"nodes":[0]}],"scene":0
            }"#,
        )
        .unwrap();

        let geometry = load_static_geometry(&gltf_path).unwrap();
        assert_eq!(geometry.skin_index, None);
        assert_eq!(geometry.vertices[0].position, [10.0, 1.5, 0.0]);
        assert!(!geometry.vertex_skins[0].is_skinned());
    }

    #[test]
    fn bounds_cover_every_static_vertex() {
        let geometry = StaticGeometry {
            vertices: vec![
                StaticVertex {
                    position: [-2.0, 0.0, 3.0],
                    normal: [0.0; 3],
                    color: [1.0; 4],
                    emission: [0.0; 3],
                },
                StaticVertex {
                    position: [4.0, 5.0, -1.0],
                    normal: [0.0; 3],
                    color: [1.0; 4],
                    emission: [0.0; 3],
                },
            ],
            ..Default::default()
        };
        let bounds = geometry.bounds().unwrap();
        assert_eq!(bounds.center(), glam::Vec3::new(1.0, 2.5, 1.0));
        assert_eq!(bounds.extent(), glam::Vec3::new(6.0, 5.0, 4.0));
    }

    #[test]
    fn empty_geometry_fails_first_model_render_gate() {
        assert!(matches!(
            validate_render_geometry(&StaticGeometry::default()),
            Err(AssetError::MissingGeometry)
        ));
    }

    #[test]
    fn animated_skeleton_lines_follow_the_sampled_joint_pose() {
        let directory = tempfile::tempdir().unwrap();
        let gltf_path = directory.path().join("rig.gltf");
        let bin_path = directory.path().join("rig.bin");
        let mut bytes = Vec::new();
        for value in [0.0_f32, 1.0] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [0.0_f32, 0.0, 0.0, 2.0, 0.0, 0.0] {
            bytes.extend(value.to_le_bytes());
        }
        std::fs::write(bin_path, bytes).unwrap();
        std::fs::write(
            &gltf_path,
            r#"{
              "asset":{"version":"2.0"},
              "buffers":[{"uri":"rig.bin","byteLength":32}],
              "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":8},
                {"buffer":0,"byteOffset":8,"byteLength":24}
              ],
              "accessors":[
                {"bufferView":0,"componentType":5126,"count":2,"type":"SCALAR","min":[0],"max":[1]},
                {"bufferView":1,"componentType":5126,"count":2,"type":"VEC3"}
              ],
              "nodes":[
                {"name":"Root","children":[1]},
                {"name":"Head","translation":[0,1,0]}
              ],
              "skins":[{"joints":[0,1],"skeleton":0}],
              "scenes":[{"nodes":[0]}],"scene":0,
              "animations":[{"samplers":[{"input":0,"output":1}],"channels":[{"sampler":0,"target":{"node":0,"path":"translation"}}]}]
            }"#,
        )
        .unwrap();

        let channels = load_animation_clip(&gltf_path, 0).unwrap().transforms;
        let lines = load_animated_skeleton_debug_geometry(&gltf_path, &channels, 0.5).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].position, [1.0, 0.0, 0.0]);
        assert_eq!(lines[1].position, [1.0, 1.0, 0.0]);
    }
}
