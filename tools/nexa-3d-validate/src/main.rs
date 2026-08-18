//! CI-safe validation command. Never creates a window or GPU device.
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let glb_path = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: nexa-3d-validate <Nexa.glb> <nexa.runtime.json>"))?;
    let manifest_path = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: nexa-3d-validate <Nexa.glb> <nexa.runtime.json>"))?;
    if arguments.next().is_some() {
        return Err(anyhow::anyhow!(
            "usage: nexa-3d-validate <Nexa.glb> <nexa.runtime.json>"
        ));
    }

    let report = nexa_3d_runtime::headless::validate_glb(&glb_path)?;
    let manifest = nexa_3d_runtime::manifest::NexaRuntimeManifest::load(&manifest_path)?;
    manifest.validate(&report)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "result": "accepted",
            "asset_version": manifest.asset_version,
            "report": report,
        }))?
    );
    Ok(())
}
