"""Executable Windows cancellation evidence path."""
import argparse
import json
from pathlib import Path

import sounddevice as sd
import soundfile as sf

from .boundary import RecognitionRequest, SynthesisRequest
from .cancellation import trial
from .sherpa_provider import SherpaProvider


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=("capture", "recognition", "synthesis", "playback"))
    parser.add_argument("--input", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--cancel-after", type=float, default=0.1)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    config = json.loads(args.config.read_text(encoding="utf-8"))
    stop = None
    output = args.output
    if args.stage == "capture":
        if output is None:
            parser.error("capture requires --output")
        def call(cancelled):
            samples = sd.rec(16000 * 30, samplerate=16000, channels=1, dtype="float32")
            sd.wait()
            if not cancelled.is_set():
                sf.write(output, samples, 16000, subtype="PCM_16")
        stop = sd.stop
    elif args.stage == "playback":
        if args.input is None:
            parser.error("playback requires --input")
        samples, rate = sf.read(args.input, dtype="float32")
        def call(cancelled):
            sd.play(samples, rate); sd.wait()
        stop = sd.stop
        output = None
    else:
        provider = SherpaProvider(config["asr"], config["tts"])
        if args.stage == "recognition":
            if args.input is None:
                parser.error("recognition requires --input")
            call = lambda cancelled: provider.recognize(RecognitionRequest("trial-recognition", args.input), cancelled)
            output = None
        else:
            if output is None:
                parser.error("synthesis requires --output")
            call = lambda cancelled: provider.synthesize(SynthesisRequest("trial-synthesis", "Cancellation trial.", output), cancelled)
    result = trial(call, args.cancel_after, stop=stop, output=output)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(args.report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
