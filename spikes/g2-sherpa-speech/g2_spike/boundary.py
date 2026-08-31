"""Spike-local implementations of Nexa's provider-neutral speech concepts.

This module deliberately keeps Sherpa types out of requests and results.  It is
evidence code, not a new production contract.
"""
from dataclasses import dataclass
from pathlib import Path
from threading import Event
from typing import Protocol


@dataclass(frozen=True)
class RecognitionRequest:
    operation_id: str
    wav: Path


@dataclass(frozen=True)
class RecognitionResult:
    operation_id: str
    transcript: str


@dataclass(frozen=True)
class SynthesisRequest:
    operation_id: str
    text: str
    output_wav: Path


@dataclass(frozen=True)
class SynthesisResult:
    operation_id: str
    output_wav: Path
    sample_rate: int
    sample_count: int


class SpeechProvider(Protocol):
    def recognize(self, request: RecognitionRequest, cancelled: Event) -> RecognitionResult: ...
    def synthesize(self, request: SynthesisRequest, cancelled: Event) -> SynthesisResult: ...


class SpeechCancelled(RuntimeError):
    pass

