"""Explicit Windows microphone/speaker observation helper."""
import argparse, json, time
from pathlib import Path
import sounddevice as sd
import soundfile as sf


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--record", type=Path, required=True)
    p.add_argument("--seconds", type=float, default=5)
    p.add_argument("--play", type=Path)
    p.add_argument("--report", type=Path, required=True)
    a = p.parse_args()
    defaults = sd.default.device
    started = time.perf_counter()
    audio = sd.rec(round(a.seconds * 16000), samplerate=16000, channels=1, dtype="float32")
    sd.wait(); record_ms = (time.perf_counter() - started) * 1000
    sf.write(a.record, audio, 16000, subtype="PCM_16")
    play_ms = None
    if a.play:
        samples, rate = sf.read(a.play, dtype="float32")
        started = time.perf_counter(); sd.play(samples, rate); sd.wait()
        play_ms = (time.perf_counter() - started) * 1000
    report = {"default_input_index": defaults[0], "default_output_index": defaults[1],
              "devices": [dict(x) for x in sd.query_devices()], "record_ms": record_ms,
              "played": str(a.play) if a.play else None, "play_ms": play_ms,
              "owner_must_confirm_audibility": True}
    a.report.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__": main()

