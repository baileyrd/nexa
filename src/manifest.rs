//! Versioned semantic mapping between NEXA behavior names and exported GLB names.
use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::asset::AssetReport;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("could not read runtime manifest: {0}")]
    Read(#[from] std::io::Error),
    #[error("runtime manifest is not valid JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("runtime manifest is invalid: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NexaRuntimeManifest {
    pub schema_version: u32,
    pub asset_version: String,
    pub canonical_forward: String,
    pub rig: RigMap,
    pub expressions: BTreeMap<String, String>,
    pub visemes: BTreeMap<String, String>,
    pub gestures: BTreeMap<String, GestureBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RigMap {
    pub armature: String,
    pub head: String,
    pub left_eye: String,
    pub right_eye: String,
    pub jaw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GestureBinding {
    pub animation: String,
    pub looping: bool,
}

impl NexaRuntimeManifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }
    pub fn validate(&self, report: &AssetReport) -> Result<(), ManifestError> {
        if self.schema_version != 1 {
            return Err(ManifestError::Invalid("schema_version must be 1".into()));
        }
        if self.asset_version.trim().is_empty() {
            return Err(ManifestError::Invalid("asset_version is required".into()));
        }
        for required in [
            "Neutral",
            "Focused",
            "Encouraging",
            "Skeptical",
            "Corrective",
        ] {
            require_mapping(&self.expressions, required, "expression")?;
        }
        for required in [
            "REST", "A", "E", "I", "O", "U", "MBP", "FV", "L", "WQ", "TH", "CHSH", "R",
        ] {
            require_mapping(&self.visemes, required, "viseme")?;
        }
        for required in [
            "Idle_Seated",
            "Point_Left",
            "Point_Right",
            "Open_Hand_Explain",
            "Adjust_Glasses",
            "Thumbs_Up",
            "Typing",
            "Listening",
        ] {
            let binding = self.gestures.get(required).ok_or_else(|| {
                ManifestError::Invalid(format!("missing required gesture `{required}`"))
            })?;
            if !report
                .animation_names()
                .any(|name| name == binding.animation)
            {
                return Err(ManifestError::Invalid(format!(
                    "gesture `{required}` refers to missing GLB animation `{}`",
                    binding.animation
                )));
            }
        }
        Ok(())
    }
}

fn require_mapping(
    map: &BTreeMap<String, String>,
    name: &str,
    category: &str,
) -> Result<(), ManifestError> {
    match map.get(name).filter(|value| !value.trim().is_empty()) {
        Some(_) => Ok(()),
        None => Err(ManifestError::Invalid(format!(
            "missing required {category} mapping `{name}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_canonical_viseme_is_rejected_without_gpu() {
        let manifest = NexaRuntimeManifest {
            schema_version: 1,
            asset_version: "v001".into(),
            canonical_forward: "-Z".into(),
            rig: RigMap {
                armature: "Nexa_Rig".into(),
                head: "Head".into(),
                left_eye: "Eye.L".into(),
                right_eye: "Eye.R".into(),
                jaw: "Jaw".into(),
            },
            expressions: [
                "Neutral",
                "Focused",
                "Encouraging",
                "Skeptical",
                "Corrective",
            ]
            .into_iter()
            .map(|name| (name.into(), name.into()))
            .collect(),
            visemes: BTreeMap::new(),
            gestures: BTreeMap::new(),
        };
        assert!(manifest
            .validate(&AssetReport {
                source: Default::default(),
                node_count: 0,
                mesh_count: 0,
                primitive_count: 0,
                morph_target_count: 0,
                morph_targets: vec![],
                skins: vec![],
                animations: vec![]
            })
            .is_err());
    }
}
