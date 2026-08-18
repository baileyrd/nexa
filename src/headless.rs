use crate::asset::{
    inspect, load_static_geometry, validate_first_model, validate_render_geometry, AssetError,
    AssetReport,
};
use std::path::Path;

/// The CI entry point: no GPU, window, audio device, or event loop is created.
pub fn validate_glb(path: impl AsRef<Path>) -> Result<AssetReport, AssetError> {
    let report = inspect(path)?;
    validate_first_model(&report)?;
    let geometry = load_static_geometry(&report.source)?;
    validate_render_geometry(&geometry)?;
    Ok(report)
}
