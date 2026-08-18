use crate::asset::{inspect, validate_first_model, AssetError, AssetReport};
use std::path::Path;

/// The CI entry point: no GPU, window, audio device, or event loop is created.
pub fn validate_glb(path: impl AsRef<Path>) -> Result<AssetReport, AssetError> {
    let report = inspect(path)?;
    validate_first_model(&report)?;
    Ok(report)
}
