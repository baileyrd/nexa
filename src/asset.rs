use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("could not import glTF/GLB `{path}`: {source}")]
    Import { path: PathBuf, source: gltf::Error },
    #[error("Nexa asset has no skinned skeleton")]
    MissingSkeleton,
    #[error("Nexa asset has no morph targets; facial/viseme validation cannot proceed")]
    MissingMorphTargets,
    #[error("mesh primitive {mesh}:{primitive} has no POSITION attribute")]
    MissingPosition { mesh: usize, primitive: usize },
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Default)]
pub struct StaticGeometry {
    pub vertices: Vec<StaticVertex>,
    pub indices: Vec<u32>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnimationReport {
    pub name: String,
    pub channel_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetReport {
    pub source: PathBuf,
    pub node_count: usize,
    pub mesh_count: usize,
    pub primitive_count: usize,
    pub morph_target_count: usize,
    pub morph_targets: Vec<MorphTargetReport>,
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
    let (document, _buffers, _images) =
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
        })
        .collect();
    Ok(AssetReport {
        source: path,
        node_count: document.nodes().count(),
        mesh_count: document.meshes().count(),
        primitive_count,
        morph_target_count: morph_targets.len(),
        morph_targets,
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
        let normal_transform = world_transform.inverse().transpose();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions = reader.read_positions().ok_or(AssetError::MissingPosition {
                mesh: mesh.index(),
                primitive: primitive.index(),
            })?;
            let normals = reader.read_normals();
            let base = geometry.vertices.len() as u32;
            match normals {
                Some(normals) => {
                    geometry
                        .vertices
                        .extend(positions.zip(normals).map(|(position, normal)| {
                            StaticVertex {
                                position: world_transform
                                    .transform_point3(glam::Vec3::from(position))
                                    .to_array(),
                                normal: normal_transform
                                    .transform_vector3(glam::Vec3::from(normal))
                                    .normalize_or_zero()
                                    .to_array(),
                            }
                        }))
                }
                None => geometry.vertices.extend(positions.map(|position| {
                    StaticVertex {
                        position: world_transform
                            .transform_point3(glam::Vec3::from(position))
                            .to_array(),
                        normal: [0.0, 1.0, 0.0],
                    }
                })),
            }
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

    #[test]
    fn bounds_cover_every_static_vertex() {
        let geometry = StaticGeometry {
            vertices: vec![
                StaticVertex {
                    position: [-2.0, 0.0, 3.0],
                    normal: [0.0; 3],
                },
                StaticVertex {
                    position: [4.0, 5.0, -1.0],
                    normal: [0.0; 3],
                },
            ],
            indices: vec![],
        };
        let bounds = geometry.bounds().unwrap();
        assert_eq!(bounds.center(), glam::Vec3::new(1.0, 2.5, 1.0));
        assert_eq!(bounds.extent(), glam::Vec3::new(6.0, 5.0, 4.0));
    }
}
