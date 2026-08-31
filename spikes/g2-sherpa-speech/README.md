# Disposable G2 Sherpa-ONNX speech spike

This directory is the bounded Issue #126 evidence fixture. It is not production
speech integration. It exercises recognition and synthesis behind spike-local,
provider-neutral requests/results shaped after `nexa-speech`; Sherpa objects do
not cross that boundary. The accessible fixture text remains usable if every
speech call fails.

## Release-route statement

- **Blocker:** Sherpa-ONNX suitability for required bundled CPU-only Windows speech.
- **Authority:** ADR-0069 decisions 10–11; NEXA-ARCH-002; NEXA-R1; G2 in both roadmaps.
- **E2E step:** local microphone audio -> neutral recognition request -> transcript,
  and accessible lesson text -> neutral synthesis request -> local speaker audio.
- **Maturity:** candidate evidence was missing before; this fixture adds automated
  evidence capability only. It does not establish `Concrete Adapter Implemented`,
  `System Verified`, `User Accepted`, candidate selection, or a G2 pass.
- **Required evidence:** the criterion table in `evidence/G2-EVIDENCE.md`.

## Windows evidence procedure

1. Use a clean checkout at the exact PR head on the representative CPU-only PC.
2. Select one English Sherpa transducer ASR model and one VITS TTS model. Copy
   `models.example.json`, then record exact archive/model names, canonical source
   URLs, licenses, and SHA-256 values. Extract them under `models/`, copy
   `config.example.json`, and adjust paths only. Do not use a substitute engine.
3. Record the three governed recognition utterances into the WAV paths in
   `fixtures.json` (mono PCM). Place any audible WAV at
   `evidence/windows/prompt.wav` for the explicit speaker check.
4. Run from PowerShell 7:

   `./scripts/validate-windows.ps1 -Config config.windows.json -ModelManifest models.windows.json`

5. Preserve the generated raw reports outside Git if they contain voice/device
   data. Copy non-sensitive measurements into the evidence record, listen to all
   synthesis fixtures, calculate word error observations, and complete every
   owner-only field. Repeat active/idle Task Manager observations and cancellation
   trials as documented there.

The script intentionally stops on incomplete provenance. Inference selects the
ONNX CPU provider and opens no remote inference connection. `pip` and model setup
do require network access; microphone recordings, transcripts, device names, and
generated voice are local potentially sensitive evidence and must not be committed.

## Known fixture limits

Sherpa's offline Python recognition and VITS calls are synchronous. Cancellation
is checked before and after each native call, so this fixture can prove queued or
post-call suppression but **cannot prove prompt interruption inside an active
native inference call**. That is an explicit G2 limitation, not success evidence.
The VITS path emits waveform timing but no phoneme/viseme alignment; usefulness
for later lip-sync must be judged from the recorded timing evidence and limitation.

