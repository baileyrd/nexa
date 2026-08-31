"""Strict, deterministic provenance and footprint evidence helpers."""
from __future__ import annotations

import hashlib
import json
import re
import argparse
from pathlib import Path

SHA256 = re.compile(r"^[0-9a-fA-F]{64}$")
PLACEHOLDERS = {"required", "replace", "replace_me", "todo", "tbd"}


def _required(value: object, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{field} must be a non-blank string")
    text = value.strip()
    if text.lower() in PLACEHOLDERS or "required" in text.lower() or "replace" in text.lower():
        raise ValueError(f"{field} contains a placeholder")
    return text


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def validate_manifest(document: object, root: Path) -> list[dict[str, object]]:
    """Validate manifest shape and verify every declared artifact on disk."""
    if not isinstance(document, dict) or document.get("schema") != 1:
        raise ValueError("manifest schema must be 1")
    runtime = document.get("runtime")
    if not isinstance(runtime, dict):
        raise ValueError("runtime must be an object")
    for field in ("name", "version", "license", "source"):
        _required(runtime.get(field), f"runtime.{field}")
    models = document.get("models")
    if not isinstance(models, list) or not models:
        raise ValueError("models must be a non-empty array")
    verified = []
    seen: set[str] = set()
    for index, model in enumerate(models):
        if not isinstance(model, dict):
            raise ValueError(f"models[{index}] must be an object")
        for field in ("name", "source", "license"):
            _required(model.get(field), f"models[{index}].{field}")
        artifacts = model.get("artifacts")
        if not isinstance(artifacts, list) or not artifacts:
            raise ValueError(f"models[{index}].artifacts must be a non-empty array")
        for artifact_index, artifact in enumerate(artifacts):
            prefix = f"models[{index}].artifacts[{artifact_index}]"
            if not isinstance(artifact, dict):
                raise ValueError(f"{prefix} must be an object")
            relative = _required(artifact.get("path"), f"{prefix}.path")
            expected = _required(artifact.get("sha256"), f"{prefix}.sha256").lower()
            if not SHA256.fullmatch(expected):
                raise ValueError(f"{prefix}.sha256 must be 64 hexadecimal characters")
            candidate = (root / relative).resolve()
            try:
                candidate.relative_to(root.resolve())
            except ValueError as error:
                raise ValueError(f"{prefix}.path escapes the spike root") from error
            if relative in seen:
                raise ValueError(f"duplicate artifact path: {relative}")
            seen.add(relative)
            if not candidate.is_file():
                raise ValueError(f"artifact is not a file: {relative}")
            actual = sha256(candidate)
            if actual != expected:
                raise ValueError(f"artifact hash mismatch: {relative}")
            verified.append({"model": model["name"], "path": relative,
                             "sha256": actual, "bytes": candidate.stat().st_size})
    return sorted(verified, key=lambda item: str(item["path"]))


def path_size(path: Path) -> int:
    if path.is_file():
        return path.stat().st_size
    if path.is_dir():
        return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())
    raise ValueError(f"footprint path does not exist: {path}")


def footprint(root: Path, manifest: dict[str, object], verified: list[dict[str, object]], venv: Path) -> dict[str, object]:
    """Return stable machine-readable archive, extracted, environment and totals."""
    archives = manifest.get("archives", [])
    if not isinstance(archives, list):
        raise ValueError("archives must be an array")
    archive_rows = []
    for index, item in enumerate(archives):
        if not isinstance(item, dict):
            raise ValueError(f"archives[{index}] must be an object")
        relative = _required(item.get("path"), f"archives[{index}].path")
        path = (root / relative).resolve()
        if not path.is_file():
            raise ValueError(f"archive is not a file: {relative}")
        expected = _required(item.get("sha256"), f"archives[{index}].sha256").lower()
        if not SHA256.fullmatch(expected) or sha256(path) != expected:
            raise ValueError(f"archive hash invalid or mismatched: {relative}")
        archive_rows.append({"path": relative, "sha256": expected, "bytes": path.stat().st_size})
    extracted = sum(int(item["bytes"]) for item in verified)
    venv_bytes = path_size(venv)
    archive_bytes = sum(int(item["bytes"]) for item in archive_rows)
    return {"schema": 1, "artifacts": verified, "archives": sorted(archive_rows, key=lambda x: x["path"]),
            "extracted_model_bytes": extracted, "archive_bytes": archive_bytes,
            "virtual_environment_bytes": venv_bytes,
            "combined_bytes": extracted + archive_bytes + venv_bytes}


def load_verify_write(manifest_path: Path, root: Path, venv: Path, output: Path) -> None:
    document = json.loads(manifest_path.read_text(encoding="utf-8"))
    verified = validate_manifest(document, root)
    result = footprint(root, document, verified, venv)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--root", default=Path("."), type=Path)
    parser.add_argument("--venv", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    load_verify_write(args.manifest, args.root, args.venv, args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
