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
 "nexa-student": {"nexa-domain", "nexa-events", "serde", "thiserror"},
 "nexa-pedagogy": {"nexa-domain", "nexa-student", "serde", "thiserror"},
 "nexa-lessons": {"nexa-domain", "nexa-pedagogy", "serde", "thiserror"},
 "nexa-assessment": {"nexa-domain", "nexa-student", "serde", "thiserror"},
 "nexa-learning-core": {"nexa-domain", "nexa-events", "nexa-student", "nexa-pedagogy", "nexa-lessons", "nexa-assessment", "serde", "thiserror"},
 "nexa-knowledge": {"nexa-domain", "serde", "sha2", "thiserror"},
 "nexa-labs": {"nexa-domain", "serde", "thiserror"},
 "nexa-knowledge-runtime": {"nexa-domain", "nexa-knowledge", "tokio", "tokio-util"},
 "nexa-tutor": {"nexa-domain", "nexa-knowledge", "serde", "serde_json", "sha2", "thiserror"},
 "nexa-speech": {"nexa-domain", "serde"},
 "nexa-orchestrator": {"nexa-domain", "serde", "thiserror"},
 "nexa-orchestrator-runtime": {"nexa-orchestrator", "tokio", "tokio-util"},
 "nexa-headless": {"nexa-avatar", "nexa-domain", "nexa-knowledge", "nexa-knowledge-runtime", "nexa-labs", "nexa-nbp", "nexa-orchestrator", "nexa-orchestrator-runtime", "nexa-speech", "nexa-tutor", "tokio"},
}
# Ignore dev-only dependencies while enforcing all normal dependency edges.
for package, allowed in expected.items():
    normal={d["name"] for p in m["packages"] if p["name"]==package for d in p["dependencies"] if d.get("kind") is None}
    unexpected=normal-allowed
    if unexpected:
        raise SystemExit(f"{package} has forbidden normal dependencies: {sorted(unexpected)}")
print("contract dependency DAG passed")
' <<<"$metadata"

# Renderer, platform, provider, executor, networking, and persistence crates must never enter contract crates.
if rg -n --glob 'Cargo.toml' --glob '*.rs' '\b(wgpu|winit|gltf|tokio|async-std|rodio|cpal|reqwest|hyper|sqlx|rusqlite)\b' crates/nexa-{domain,events,nbp,avatar,student,pedagogy,lessons,assessment,learning-core,knowledge,labs,tutor,speech,orchestrator}; then
  echo "contract crate references a forbidden implementation dependency" >&2
  exit 1
fi

# Tokio is isolated in the runtime adapter and must not enter protected synchronous crates.
if rg -n --glob 'Cargo.toml' --glob '*.rs' '\b(tokio|tokio-util|tokio_util)\b' \
  crates/nexa-{domain,events,nbp,avatar,student,pedagogy,lessons,assessment,learning-core,knowledge,labs,tutor,speech,orchestrator}; then
  echo "protected contract/domain crate references Tokio" >&2
  exit 1
fi

# Renderer, platform, provider, and executor crates must never enter the avatar contract.
if rg -n --glob 'Cargo.toml' --glob '*.rs' \
  '\b(wgpu|winit|gltf|tokio|async-std|rodio|cpal|nexa-3d-runtime)\b' crates/nexa-avatar; then
  echo "nexa-avatar references a forbidden renderer/platform/provider/runtime dependency" >&2
  exit 1
fi

# Renderer and OS-window dependencies belong exclusively to the viewer composition root.
if cargo tree -p nexa-3d --no-default-features --edges normal | rg -q '\b(wgpu|winit|pollster)\b'; then
  echo "nexa-3d headless dependency graph contains viewer dependencies" >&2
  exit 1
fi
if cargo tree -p nexa-3d-validate --edges normal | rg -q '\b(wgpu|winit|pollster)\b'; then
  echo "nexa-3d-validate dependency graph contains viewer dependencies" >&2
  exit 1
fi

for crate in nexa-domain nexa-events nexa-nbp nexa-avatar nexa-student nexa-pedagogy nexa-lessons nexa-assessment nexa-learning-core nexa-knowledge nexa-labs nexa-tutor nexa-speech nexa-orchestrator; do
  if cargo tree -p "$crate" --edges normal | rg -q '\b(wgpu|winit|gltf|pollster)\b'; then
    echo "$crate dependency graph contains a renderer dependency" >&2
    exit 1
  fi
done

echo "3D renderer boundary passed"
