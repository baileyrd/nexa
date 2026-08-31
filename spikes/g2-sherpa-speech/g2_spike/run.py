import argparse, json, os, platform, subprocess, sys, time
from pathlib import Path
from threading import Event

import psutil

from .boundary import RecognitionRequest, SynthesisRequest
from .sherpa_provider import SherpaProvider


def timed(call):
    process = psutil.Process()
    before = process.memory_info().rss
    start = time.perf_counter()
    value = call()
    elapsed = (time.perf_counter() - start) * 1000
    return value, {"elapsed_ms": round(elapsed, 3), "rss_before": before,
                   "rss_after": process.memory_info().rss}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run disposable local G2 speech evidence")
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--fixtures", default=Path("fixtures.json"), type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args(argv)
    config = json.loads(args.config.read_text(encoding="utf-8"))
    fixtures = json.loads(args.fixtures.read_text(encoding="utf-8"))
    root = args.fixtures.parent
    started = time.time()
    provider, startup = timed(lambda: SherpaProvider(config["asr"], config["tts"]))
    cancelled = Event()
    recognition, synthesis = [], []
    for item in fixtures["recognition"]:
        result, metric = timed(lambda item=item: provider.recognize(
            RecognitionRequest(item["id"], root / item["wav"]), cancelled))
        recognition.append({**item, "actual": result.transcript, **metric})
    for item in fixtures["synthesis"]:
        target = args.output.parent / "audio" / f'{item["id"]}.wav'
        result, metric = timed(lambda item=item, target=target: provider.synthesize(
            SynthesisRequest(item["id"], item["text"], target), cancelled))
        synthesis.append({**item, "wav": str(result.output_wav), "sample_rate": result.sample_rate,
                          "sample_count": result.sample_count, **metric})
    packages = subprocess.check_output([sys.executable, "-m", "pip", "freeze"], text=True).splitlines()
    report = {"schema": 1, "git_head": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
              "started_unix": started, "platform": platform.platform(), "python": sys.version,
              "cpu_count": os.cpu_count(), "packages": packages, "config": config,
              "startup": startup, "recognition": recognition, "synthesis": synthesis,
              "locality": "offline provider=cpu; the fixture opens no network connection during inference"}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
