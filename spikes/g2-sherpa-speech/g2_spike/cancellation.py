"""Bounded cancellation trials for the disposable G2 fixture."""
from __future__ import annotations

import time
from pathlib import Path
from threading import Event, Thread
from typing import Callable

from .boundary import SpeechCancelled


def trial(call: Callable[[Event], object], cancel_after: float,
          stop: Callable[[], None] | None = None, output: Path | None = None) -> dict[str, object]:
    """Request cancellation during a call and honestly report its terminal state.

    The worker cannot interrupt synchronous Sherpa native calls. Device callers
    can supply ``stop`` so capture/playback is actually stopped by this fixture.
    """
    cancelled = Event()
    state: dict[str, object] = {}
    started = time.perf_counter()

    def invoke() -> None:
        try:
            state["value"] = call(cancelled)
            state["outcome"] = "completed"
        except SpeechCancelled:
            state["outcome"] = "cancelled"
        except BaseException as error:  # recorded for evidence, re-raised by caller only if desired
            state["outcome"] = "error"
            state["error_type"] = type(error).__name__

    worker = Thread(target=invoke, name="g2-cancellation-trial")
    worker.start()
    time.sleep(max(0.0, cancel_after))
    requested = time.perf_counter()
    cancelled.set()
    if stop is not None:
        stop()
    worker.join()
    terminal = time.perf_counter()
    published = bool(output and output.exists())
    return {"schema": 1, "cancel_requested_ms": round((requested - started) * 1000, 3),
            "terminal_ms": round((terminal - started) * 1000, 3),
            "request_to_terminal_ms": round((terminal - requested) * 1000, 3),
            "outcome": state.get("outcome", "error"), "error_type": state.get("error_type"),
            "output_published": published, "queued_output": published,
            "device_stop_requested": stop is not None, "device_cleanup": "stop-called" if stop else "not-applicable",
            "native_call_interruptible": False,
            "limitation": "synchronous Sherpa native inference finishes before post-call cancellation is observed"}
