import json
import tempfile
import types
import unittest
import sys
from pathlib import Path
from unittest.mock import patch

sys.modules.setdefault("psutil", types.SimpleNamespace(Process=lambda: None))
sys.modules.setdefault("soundfile", types.SimpleNamespace())
import g2_spike.run as run_module


class RunReportTests(unittest.TestCase):
    def test_report_captures_exact_head_packages_config_and_locality_deterministically(self):
        def synthesize(request, _cancel):
            request.output_wav.parent.mkdir(parents=True, exist_ok=True)
            request.output_wav.write_bytes(b"wav")
            return types.SimpleNamespace(output_wav=request.output_wav, sample_rate=16000, sample_count=3)

        provider = types.SimpleNamespace(
            recognize=lambda _request, _cancel: types.SimpleNamespace(transcript="text"), synthesize=synthesize)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); config = root / "config.json"; fixtures = root / "fixtures.json"
            config.write_text(json.dumps({"asr":{"model":"a"}, "tts":{"model":"b"}}))
            fixtures.write_text(json.dumps({"recognition":[{"id":"r", "wav":"in.wav"}],
                                             "synthesis":[{"id":"s", "text":"hello"}]}))
            outputs = []
            for name in ("one.json", "two.json"):
                output = root / name
                with patch.object(run_module, "SherpaProvider", return_value=provider), \
                     patch.object(run_module, "timed", side_effect=lambda call: (call(), {"elapsed_ms":1.0,"rss_before":2,"rss_after":3})), \
                     patch.object(run_module.time, "time", return_value=123.0), \
                     patch.object(run_module.platform, "platform", return_value="Windows-test"), \
                     patch.object(run_module.os, "cpu_count", return_value=4), \
                     patch.object(run_module.sys, "version", "3.11-test"), \
                     patch.object(run_module.subprocess, "check_output", side_effect=["pkg==1\n", "abc123\n"]):
                    self.assertEqual(run_module.main(["--config", str(config), "--fixtures", str(fixtures),
                                                      "--output", str(output)]), 0)
                outputs.append(json.loads(output.read_text()))
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[0]["git_head"], "abc123")
            self.assertEqual(outputs[0]["packages"], ["pkg==1"])
            self.assertEqual(outputs[0]["config"], {"asr":{"model":"a"}, "tts":{"model":"b"}})
            self.assertIn("offline", outputs[0]["locality"])


class TrialCliTests(unittest.TestCase):
    def test_each_stage_routes_stage_and_device_cleanup(self):
        fake_sd = types.SimpleNamespace(rec=lambda *a, **k: [0], wait=lambda: None,
                                        stop=lambda: None, play=lambda *a: None)
        fake_sf = types.SimpleNamespace(read=lambda *a, **k: ([0], 16000), write=lambda *a, **k: None)
        with patch.dict("sys.modules", {"sounddevice": fake_sd, "soundfile": fake_sf}):
            import importlib
            trial_module = importlib.reload(importlib.import_module("g2_spike.trial"))
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory); config = root / "config.json"; config.write_text('{"asr":{},"tts":{}}')
                source = root / "in.wav"; source.write_bytes(b"wav")
                for stage in ("capture", "recognition", "synthesis", "playback"):
                    report = root / f"{stage}.json"; output = root / f"{stage}.wav"
                    args = ["--config", str(config), "--stage", stage, "--report", str(report)]
                    if stage in {"capture", "synthesis"}: args += ["--output", str(output)]
                    if stage in {"recognition", "playback"}: args += ["--input", str(source)]
                    recorded = {"schema":1, "stage":stage, "outcome":"test"}
                    with patch.object(trial_module, "SherpaProvider", return_value=types.SimpleNamespace(
                            recognize=lambda *a: None, synthesize=lambda *a: None)), \
                         patch.object(trial_module, "trial", return_value=recorded) as invoke:
                        self.assertEqual(trial_module.main(args), 0)
                    self.assertEqual(invoke.call_args.kwargs["stage"], stage)
                    self.assertEqual(invoke.call_args.kwargs["stop"] is not None,
                                     stage in {"capture", "playback"})
                    self.assertEqual(json.loads(report.read_text()), recorded)


class PowerShellHarnessTests(unittest.TestCase):
    def test_every_native_command_is_fail_closed(self):
        script = (Path(__file__).parents[1] / "scripts/validate-windows.ps1").read_text()
        self.assertIn("if ($LASTEXITCODE -ne 0)", script)
        native_lines = [line.strip() for line in script.splitlines()
                        if "git status" in line or "py -3.11" in line or "\\Scripts\\python" in line]
        self.assertTrue(native_lines)
        self.assertTrue(all("Invoke-Native" in line for line in native_lines), native_lines)
        self.assertLess(script.index("repository postflight"), script.index('Write-Host "Automation finished'))


if __name__ == "__main__":
    unittest.main()
