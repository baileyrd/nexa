#!/usr/bin/env bash
set -euo pipefail
metadata=$(cargo metadata --format-version 1 --no-deps)
python3 -c '
import json,sys
m=json.load(sys.stdin)
packages={p["name"]: {d["name"] for d in p["dependencies"]} for p in m["packages"]}
expected={
 "nexa-domain": {"chrono", "serde", "thiserror", "uuid"},
 "nexa-events": {"nexa-domain", "serde", "serde_json", "thiserror"},
 "nexa-nbp": {"nexa-domain", "serde", "serde_json", "thiserror"},
 "nexa-avatar": {"nexa-domain", "nexa-nbp", "serde", "thiserror"},
}
# Ignore dev-only dependencies while enforcing all normal dependency edges.
for package, allowed in expected.items():
    normal={d["name"] for p in m["packages"] if p["name"]==package for d in p["dependencies"] if d.get("kind") is None}
    unexpected=normal-allowed
    if unexpected:
        raise SystemExit(f"{package} has forbidden normal dependencies: {sorted(unexpected)}")
print("contract dependency DAG passed")
' <<<"$metadata"

# Renderer, platform, provider, and executor crates must never enter the avatar contract.
if rg -n --glob 'Cargo.toml' --glob '*.rs' \
  '\b(wgpu|winit|gltf|tokio|async-std|rodio|cpal|nexa-3d-runtime)\b' crates/nexa-avatar; then
  echo "nexa-avatar references a forbidden renderer/platform/provider/runtime dependency" >&2
  exit 1
fi
