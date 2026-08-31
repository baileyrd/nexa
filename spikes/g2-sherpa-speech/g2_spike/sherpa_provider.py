"""Narrow disposable Sherpa-ONNX adapter behind spike-neutral boundaries."""
from pathlib import Path
from threading import Event

import soundfile as sf

from .boundary import RecognitionRequest, RecognitionResult, SpeechCancelled, SynthesisRequest, SynthesisResult


class SherpaProvider:
    def __init__(self, asr: dict[str, str | int], tts: dict[str, str | int]):
        import sherpa_onnx

        self._asr = sherpa_onnx.OfflineRecognizer.from_transducer(
            tokens=str(asr["tokens"]), encoder=str(asr["encoder"]),
            decoder=str(asr["decoder"]), joiner=str(asr["joiner"]),
            num_threads=int(asr.get("num_threads", 2)), provider="cpu",
        )
        vits = sherpa_onnx.OfflineTtsVitsModelConfig(
            model=str(tts["model"]), lexicon=str(tts.get("lexicon", "")),
            tokens=str(tts["tokens"]), data_dir=str(tts.get("data_dir", "")),
        )
        model = sherpa_onnx.OfflineTtsModelConfig(
            vits=vits, num_threads=int(tts.get("num_threads", 2)), provider="cpu",
        )
        self._tts = sherpa_onnx.OfflineTts(sherpa_onnx.OfflineTtsConfig(model=model))

    def recognize(self, request: RecognitionRequest, cancelled: Event) -> RecognitionResult:
        if cancelled.is_set():
            raise SpeechCancelled(request.operation_id)
        samples, sample_rate = sf.read(request.wav, dtype="float32", always_2d=False)
        if getattr(samples, "ndim", 1) != 1:
            raise ValueError("recognition fixture must be mono")
        stream = self._asr.create_stream()
        stream.accept_waveform(sample_rate, samples)
        self._asr.decode_stream(stream)
        if cancelled.is_set():
            raise SpeechCancelled(request.operation_id)
        return RecognitionResult(request.operation_id, stream.result.text.strip())

    def synthesize(self, request: SynthesisRequest, cancelled: Event) -> SynthesisResult:
        # A prior run must never make a cancelled request appear published.
        request.output_wav.unlink(missing_ok=True)
        if cancelled.is_set():
            raise SpeechCancelled(request.operation_id)
        audio = self._tts.generate(request.text, sid=0, speed=1.0)
        if cancelled.is_set():
            request.output_wav.unlink(missing_ok=True)
            raise SpeechCancelled(request.operation_id)
        request.output_wav.parent.mkdir(parents=True, exist_ok=True)
        sf.write(request.output_wav, audio.samples, audio.sample_rate, subtype="PCM_16")
        return SynthesisResult(request.operation_id, request.output_wav, audio.sample_rate, len(audio.samples))
