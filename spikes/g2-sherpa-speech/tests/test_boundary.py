import tempfile, unittest
from pathlib import Path
from threading import Event

from g2_spike.boundary import RecognitionRequest, RecognitionResult, SpeechCancelled, SynthesisRequest, SynthesisResult


class FakeProvider:
    def recognize(self, request, cancelled):
        if cancelled.is_set(): raise SpeechCancelled(request.operation_id)
        return RecognitionResult(request.operation_id, "local transcript")
    def synthesize(self, request, cancelled):
        if cancelled.is_set(): raise SpeechCancelled(request.operation_id)
        request.output_wav.write_bytes(b"RIFF")
        return SynthesisResult(request.operation_id, request.output_wav, 16000, 1)


class BoundaryTests(unittest.TestCase):
    def test_input_and_output_cross_neutral_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            provider, cancel = FakeProvider(), Event()
            self.assertEqual(provider.recognize(RecognitionRequest("r1", Path("in.wav")), cancel).transcript, "local transcript")
            result = provider.synthesize(SynthesisRequest("s1", "accessible text", Path(directory) / "out.wav"), cancel)
            self.assertTrue(result.output_wav.exists())

    def test_both_paths_honor_pre_cancel(self):
        cancel = Event(); cancel.set(); provider = FakeProvider()
        with self.assertRaises(SpeechCancelled): provider.recognize(RecognitionRequest("r", Path("x")), cancel)
        with self.assertRaises(SpeechCancelled): provider.synthesize(SynthesisRequest("s", "text", Path("x")), cancel)


if __name__ == "__main__": unittest.main()
