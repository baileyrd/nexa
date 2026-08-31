# G2 Sherpa-ONNX criterion evidence — Issue #126

Status: **INCOMPLETE — G2 is not passed and Sherpa-ONNX remains a candidate.**

Automated Linux tests at authoring time prove only neutral boundary routing and
pre-cancellation behavior. No representative CPU-only Windows run or owner quality
review is recorded in this commit. Never replace a blank owner observation with an
automated inference.

## Criterion-level record

| Criterion | Automated artifact | Required owner-observed Windows evidence | Current result |
|---|---|---|---|
| Exact environment/versions | `automated.json`, model hashes, computer info, Git status | Confirm clean exact head, Windows build, CPU/RAM, no GPU provider, tool/runtime/model versions | Missing |
| Runtime/model provenance and licensing | manifest rejects blank source/license/hash; runtime pinned to 1.13.6 (Apache-2.0) | Review redistribution compatibility for each exact model and all notices | Missing; model selection deliberately not fabricated |
| Locality/privacy | CPU provider is explicit; report states offline inference | Observe network activity; confirm recordings/transcripts stay local and deletion behavior | Missing |
| Microphone/input device | device helper records selected default and a local WAV | Confirm correct device, recording clarity, permission/error/recovery behavior | Missing |
| Recognition quality | exact transcript and per-item latency report | Speak all 3 governed fixtures; record substitutions/deletions/insertions and acceptability | Missing |
| Synthesis quality | 3 WAVs plus sample counts/rates and per-item latency | Listen on representative speaker; record intelligibility, pronunciation, naturalness, defects | Missing |
| Speaker/output device | helper enumerates default and plays a WAV | Confirm correct device, audibility, switching/error/recovery | Missing |
| Startup/latency/timing | startup and operation milliseconds, waveform duration | Repeat cold/warm runs; judge responsiveness and later lip-sync usefulness | Missing |
| Idle/active CPU and memory | process RSS before/after calls | Record Task Manager idle/ASR/TTS CPU and working set with method/precision | Missing |
| Package/model footprint | installed files and model hashes can be measured | Record venv, each archive/extracted model, and combined sizes; assess packaging viability | Missing |
| Input cancellation | automated pre-cancel test | Attempt before/during recognition and record terminal latency/audio/device release | Missing; in-call native cancellation unsupported by fixture |
| Output interruption | automated pre-cancel test | Attempt before/during synthesis and playback; record stop latency and queued audio behavior | Missing; native generation interruption unsupported by fixture |
| Lip-sync timing | output sample rate/count gives utterance duration | Judge available timing; no phoneme/viseme alignment is produced | Missing/known limitation |
| Accessible fallback | fixture text exists independently of speech output | Confirm text/transcript remains readable when devices/provider fail | Missing |

## Disposition

No G2 disposition is authorized by this evidence record. The owner/Chief Systems
Architect must review every missing measurement. If Sherpa-ONNX is disproved,
ADR-0069 requires removing it as candidate, promoting **only** whisper.cpp
recognition to a separate governed evidence gate, and making a separate TTS
selection; this spike must not substitute another stack.
