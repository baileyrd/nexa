import hashlib
import json
import sys
import tempfile
import types
import unittest
from pathlib import Path
from threading import Event
from unittest.mock import patch

from g2_spike.boundary import RecognitionRequest, SpeechCancelled, SynthesisRequest
from g2_spike.cancellation import trial
from g2_spike.evidence import footprint, validate_manifest
sys.modules.setdefault("soundfile", types.SimpleNamespace())
import g2_spike.sherpa_provider


class Samples:
    ndim = 1
    def __len__(self): return 3


class FakeStream:
    def __init__(self): self.result = types.SimpleNamespace(text="  local transcript ")
    def accept_waveform(self, rate, samples): self.accepted = (rate, samples)


class FakeRecognizer:
    def create_stream(self): return FakeStream()
    def decode_stream(self, stream): pass


class FakeTts:
    def __init__(self, config): self.config = config
    def generate(self, text, sid, speed):
        return types.SimpleNamespace(samples=Samples(), sample_rate=16000)


def sherpa_module(recognizer=None):
    recognizer = recognizer or FakeRecognizer()
    class OfflineRecognizer:
        @staticmethod
        def from_transducer(**kwargs):
            OfflineRecognizer.kwargs = kwargs
            return recognizer
    class Config:
        def __init__(self, **kwargs): self.kwargs = kwargs
    return types.SimpleNamespace(OfflineRecognizer=OfflineRecognizer,
        OfflineTtsVitsModelConfig=Config, OfflineTtsModelConfig=Config,
        OfflineTtsConfig=Config, OfflineTts=FakeTts)


class ProviderTests(unittest.TestCase):
    def provider(self, recognizer=None):
        fake = sherpa_module(recognizer)
        with patch.dict(sys.modules, {"sherpa_onnx": fake}):
            from g2_spike.sherpa_provider import SherpaProvider
            provider = SherpaProvider({"tokens":"t", "encoder":"e", "decoder":"d", "joiner":"j"},
                                      {"model":"m", "tokens":"t"})
        self.assertEqual(fake.OfflineRecognizer.kwargs["provider"], "cpu")
        self.assertEqual(provider._tts.config.kwargs["model"].kwargs["provider"], "cpu")
        return provider

    @patch("g2_spike.sherpa_provider.sf")
    def test_recognition_and_synthesis_success(self, sf):
        sf.read.return_value = (Samples(), 16000)
        provider = self.provider()
        self.assertEqual(provider.recognize(RecognitionRequest("r", Path("x")), Event()).transcript,
                         "local transcript")
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "out.wav"
            sf.write.side_effect = lambda path, *_args, **_kwargs: path.write_bytes(b"wav")
            result = provider.synthesize(SynthesisRequest("s", "text", target), Event())
            self.assertEqual((result.sample_rate, result.sample_count), (16000, 3))
            self.assertTrue(target.exists())

    @patch("g2_spike.sherpa_provider.sf")
    def test_invalid_non_mono_input(self, sf):
        sf.read.return_value = (types.SimpleNamespace(ndim=2), 16000)
        with self.assertRaisesRegex(ValueError, "mono"):
            self.provider().recognize(RecognitionRequest("r", Path("x")), Event())

    @patch("g2_spike.sherpa_provider.sf")
    def test_pre_and_post_native_cancellation_suppress_output(self, sf):
        cancel = Event(); provider = self.provider()
        cancel.set()
        with self.assertRaises(SpeechCancelled):
            provider.recognize(RecognitionRequest("r", Path("x")), cancel)
        cancel.clear()
        provider._tts.generate = lambda *_args, **_kwargs: (cancel.set() or types.SimpleNamespace(samples=Samples(), sample_rate=1))
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "out.wav"; target.write_bytes(b"stale")
            with self.assertRaises(SpeechCancelled):
                provider.synthesize(SynthesisRequest("s", "text", target), cancel)
            self.assertFalse(target.exists())
            sf.write.assert_not_called()

    @patch("g2_spike.sherpa_provider.sf")
    def test_recognition_post_call_cancellation(self, sf):
        cancel = Event(); sf.read.return_value = (Samples(), 16000)
        class CancellingRecognizer(FakeRecognizer):
            def decode_stream(self, stream): cancel.set()
        with self.assertRaises(SpeechCancelled):
            self.provider(CancellingRecognizer()).recognize(RecognitionRequest("r", Path("x")), cancel)


class EvidenceTests(unittest.TestCase):
    def manifest(self, digest):
        return {"schema": 1, "runtime": {"name":"sherpa", "version":"1", "license":"owner-supplied", "source":"https://example.invalid"},
                "models": [{"name":"owner model", "source":"https://example.invalid/model", "license":"owner-reviewed",
                            "artifacts":[{"path":"models/a.bin", "sha256":digest}]}], "archives": []}

    def test_validation_and_deterministic_footprint(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); artifact = root / "models/a.bin"; artifact.parent.mkdir(); artifact.write_bytes(b"abc")
            venv = root / ".venv"; venv.mkdir(); (venv / "package").write_bytes(b"12345")
            digest = hashlib.sha256(b"abc").hexdigest(); document = self.manifest(digest)
            verified = validate_manifest(document, root)
            result = footprint(root, document, verified, venv)
            self.assertEqual(result["combined_bytes"], 8)
            self.assertEqual(result["artifacts"][0]["sha256"], digest)
            self.assertEqual(json.dumps(result, sort_keys=True), json.dumps(result, sort_keys=True))

    def test_rejects_incomplete_malformed_and_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / "models").mkdir(); (root / "models/a.bin").write_bytes(b"abc")
            for digest in ("", "REQUIRED", "0" * 64):
                with self.subTest(digest=digest), self.assertRaises(ValueError):
                    validate_manifest(self.manifest(digest), root)
            incomplete = self.manifest(hashlib.sha256(b"abc").hexdigest()); incomplete["models"][0]["license"] = ""
            with self.assertRaises(ValueError): validate_manifest(incomplete, root)


class CancellationTests(unittest.TestCase):
    def test_during_call_reports_limitation_and_cleanup(self):
        stopped = Event()
        def call(cancelled):
            cancelled.wait(1)
            raise SpeechCancelled("x")
        report = trial(call, 0.001, stop=stopped.set)
        self.assertEqual(report["outcome"], "cancelled")
        self.assertTrue(report["device_stop_requested"])
        self.assertFalse(report["native_call_interruptible"])
        self.assertFalse(report["output_published"])


if __name__ == "__main__": unittest.main()
