"""Bounded cancellation trials for the disposable G2 fixture."""
from __future__ import annotations

import time
from pathlib import Path
from threading import Event, Thread
from typing import Callable

from .boundary import SpeechCancelled


def trial(call: Callable[[Event], object], cancel_after: float, *, stage: str,
          terminal_timeout: float = 5.0, stop: Callable[[], None] | None = None,
          output: Path | None = None) -> dict[str, object]:
    """Request cancellation during a call and honestly report its terminal state.

    The worker cannot interrupt synchronous Sherpa native calls. Device callers
    can supply ``stop`` so capture/playback is actually stopped by this fixture.
    """
    cancelled = Event()
    if stage not in {"capture", "recognition", "synthesis", "playback"}:
        raise ValueError(f"unknown cancellation stage: {stage}")
    if terminal_timeout < 0:
        raise ValueError("terminal_timeout must be non-negative")
    state: dict[str, object] = {}
    finished = Event()
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
        finally:
            finished.set()

    worker = Thread(target=invoke, name="g2-cancellation-trial", daemon=True)
    worker.start()
    finished.wait(max(0.0, cancel_after))
    requested = time.perf_counter()
    completed_before_request = finished.is_set()
    cancelled.set()
    stop_error = None
    if stop is not None:
        try:
            stop()
        except BaseException as error:
            stop_error = type(error).__name__
    worker.join(terminal_timeout)
    observed = time.perf_counter()
    published = bool(output and output.exists())
    still_running = worker.is_alive()
    if stop_error:
        outcome = "error"
        state["error_type"] = stop_error
    elif completed_before_request:
        outcome = "completed-before-request" if state.get("outcome") == "completed" else state.get("outcome", "error")
    elif still_running and stage in {"recognition", "synthesis"}:
        outcome = "non-interruptible-at-deadline"
    elif still_running:
        outcome = "error"
        state["error_type"] = "TerminalDeadlineExceeded"
    elif state.get("outcome") == "error":
        outcome = "error"
    elif stage in {"capture", "playback"} and stop is not None:
        outcome = "stopped-after-request"
    else:
        outcome = "cancelled-after-request" if state.get("outcome") == "cancelled" else "completed-after-request"
    result: dict[str, object] = {"schema": 1, "stage": stage,
            "terminal_deadline_ms": round(terminal_timeout * 1000, 3),
            "cancel_requested_ms": round((requested - started) * 1000, 3),
            "deadline_observed_ms": round((observed - started) * 1000, 3),
            "terminal_ms": None if still_running else round((observed - started) * 1000, 3),
            "request_to_terminal_ms": None if still_running else round((observed - requested) * 1000, 3),
            "outcome": outcome, "error_type": state.get("error_type"),
            "output_published": published, "queued_output": published,
            "device_stop_requested": stop is not None,
            "device_cleanup": ("stop-failed" if stop_error else "stop-completed") if stop else "not-applicable"}
    if stage in {"recognition", "synthesis"}:
        result["native_call_interruptible"] = False
        result["limitation"] = ("synchronous Sherpa native inference did not terminate by the deadline"
                                if still_running else None)
    return result
