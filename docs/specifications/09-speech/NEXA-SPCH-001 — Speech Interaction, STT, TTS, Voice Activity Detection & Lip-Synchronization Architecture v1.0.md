# NEXA-SPCH-001 — Speech Interaction, STT, TTS, Voice Activity Detection & Lip-Synchronization Architecture v1.0

**Specification ID:** NEXA-SPCH-001
**System:** Nexa AI Training Tutor
**Version:** 1.0
**Status:** Baseline Draft
**Depends On:** NEXA-DOM-001, NEXA-EVT-001, NEXA-NBP-001, NEXA-ORCH-001, NEXA-TUTOR-001, NEXA-CBS-001
**Purpose:** Define Nexa’s full conversational speech pipeline, including microphone capture, voice activity detection, streaming speech recognition, interruption, speech synthesis, prosody, pronunciation, viseme generation, lip synchronization, provider abstraction, latency controls, local-first execution, and failure recovery.

---

## 1. Purpose

The Speech subsystem provides Nexa with a natural bidirectional voice interface.

It answers two runtime questions:

> **“What did the student say?”**

and:

> **“How should Nexa say her response?”**

The speech architecture SHALL support real-time conversation rather than treating speech as an offline file-conversion task.

---

# 2. Full Speech Architecture

```text
STUDENT
   │
   ▼
Microphone
   │
   ▼
Audio Capture
   │
   ▼
Input Processing
   │
   ├── resampling
   ├── noise reduction
   ├── echo cancellation
   └── gain control
   │
   ▼
Voice Activity Detection
   │
   ▼
Speech Segmenter
   │
   ▼
Streaming STT
   │
   ├── partial transcripts
   └── final transcript
   │
   ▼
NEXA-ORCH-001
   │
   ▼
NEXA-TUTOR-001
   │
   ▼
Committed Speech Chunks
   │
   ▼
TTS
   │
   ├── audio
   ├── phonemes
   ├── visemes
   └── timing
   │
   ▼
Playback + Avatar Runtime
   │
   ▼
NEXA SPEAKS
```

---

# 3. Core Responsibilities

The speech subsystem SHALL own or coordinate:

* audio-device discovery;
* microphone capture;
* output playback;
* sample-rate normalization;
* voice activity detection;
* speech segmentation;
* streaming STT;
* transcript confidence;
* endpoint detection;
* barge-in detection;
* speech synthesis;
* voice profile selection;
* prosody;
* technical pronunciation;
* acronym handling;
* code pronunciation;
* audio buffering;
* viseme generation;
* lip-sync timing;
* speech cancellation;
* playback interruption;
* speech telemetry;
* speech-provider failover.

---

# 4. Explicit Non-Responsibilities

The speech subsystem SHALL NOT determine:

* what Nexa teaches;
* whether an answer is correct;
* which pedagogical strategy is selected;
* whether a tool may execute;
* which low-level body animation is selected;
* competency mastery;
* final tutor-response content.

---

# 5. Primary Design Objective

The critical objective is low perceived conversational latency.

The system SHOULD optimize:

```text
student stops speaking
        ↓
Nexa reacts visually
        ↓
Nexa begins speaking
```

rather than merely optimizing total response completion time.

---

# 6. Speech Modes

```rust
pub enum SpeechMode {
    TextOnly,
    PushToTalk,
    VoiceActivated,
    FullDuplex,
}
```

The MVP SHOULD support:

```text
TextOnly
PushToTalk
```

followed by:

```text
VoiceActivated
FullDuplex
```

---

# 7. Full Duplex

Full-duplex conversation means:

```text
Student may speak
while
Nexa is speaking
```

This requires:

* echo cancellation;
* interruption detection;
* playback cancellation;
* STT filtering;
* orchestration coordination.

---

# 8. Audio Device Model

```rust
pub struct AudioDevice {
    pub id: AudioDeviceId,
    pub name: String,
    pub kind: AudioDeviceKind,
    pub channels: u16,
    pub supported_sample_rates: Vec<u32>,
    pub is_default: bool,
}
```

---

# 9. Device Kinds

```rust
pub enum AudioDeviceKind {
    Input,
    Output,
}
```

---

# 10. Device Discovery

The runtime SHOULD enumerate available devices and react to hot-plug events.

Examples:

```text
USB headset connected
Bluetooth microphone disconnected
default output changed
```

---

# 11. Device Failure

If the active microphone disappears:

```text
audio.device.disconnected
      ↓
speech.capture.cancelled
      ↓
fallback device?
```

If no device exists:

```text
text input remains available
```

---

# 12. Audio Format

The internal capture format SHOULD be normalized.

A reasonable baseline is:

```text
mono
PCM
16-bit or float32
16 kHz / 24 kHz / 48 kHz depending on pipeline
```

Provider adapters SHALL handle provider-specific conversion.

---

# 13. Audio Frame

```rust
pub struct AudioFrame {
    pub sequence: u64,
    pub timestamp: Timestamp,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: AudioSamples,
}
```

---

# 14. Audio Pipeline

```text
device-native audio
       ↓
channel conversion
       ↓
resampling
       ↓
gain normalization
       ↓
echo cancellation
       ↓
noise processing
       ↓
VAD
```

---

# 15. Resampling

Resampling SHALL occur at infrastructure boundaries.

STT and TTS domain logic SHOULD not care whether the hardware captures at 44.1 kHz or 48 kHz.

---

# 16. Automatic Gain Control

AGC MAY be used to compensate for inconsistent microphone levels.

It SHOULD avoid aggressive pumping or distortion.

---

# 17. Noise Reduction

Noise processing MAY reduce:

```text
fans
keyboard noise
HVAC
background hum
low-level room noise
```

It SHOULD not materially distort speech.

---

# 18. Echo Cancellation

Acoustic echo cancellation becomes mandatory for natural full-duplex interaction.

Without AEC:

```text
Nexa's voice
   ↓
microphone
   ↓
STT
   ↓
Nexa transcribes herself
```

This SHALL be prevented.

---

# 19. Echo Reference

Playback audio SHOULD be available to the AEC pipeline as a reference signal.

```text
TTS playback
    │
    ├──────────────► speakers
    │
    └──────────────► echo canceller reference
```

---

# 20. Voice Activity Detection

VAD determines:

```text
speech
versus
silence / background noise
```

VAD SHALL be provider-independent.

---

# 21. VAD Contract

```rust
pub trait VoiceActivityDetector {
    fn process(
        &mut self,
        frame: &AudioFrame,
    ) -> VoiceActivityResult;
}
```

---

# 22. Voice Activity Result

```rust
pub struct VoiceActivityResult {
    pub probability: Confidence,
    pub state: VoiceActivityState,
}
```

---

# 23. VAD States

```rust
pub enum VoiceActivityState {
    Silence,
    PossibleSpeech,
    Speech,
}
```

---

# 24. Speech Start

Speech SHOULD not begin after one isolated noisy frame.

A debounce window SHOULD establish speech onset.

```text
possible
possible
speech
speech
      ↓
student.speech.started
```

---

# 25. Speech Endpointing

Speech completion SHALL balance:

```text
fast response
vs
not cutting the student off
```

---

# 26. Endpoint Policy

```rust
pub struct EndpointPolicy {
    pub speech_start_frames: u32,
    pub minimum_speech: Duration,
    pub trailing_silence: Duration,
    pub maximum_utterance: Duration,
}
```

---

# 27. Adaptive Endpointing

Future versions MAY adapt silence thresholds based on:

```text
student speaking pace
question type
language
long-form explanation
```

---

# 28. Speech Segment

```rust
pub struct SpeechSegment {
    pub id: SpeechSegmentId,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub audio_ref: ArtifactId,
}
```

---

# 29. Streaming STT

Speech recognition SHOULD operate while the student speaks.

```text
audio
  ↓
partial transcript
  ↓
partial transcript
  ↓
partial transcript
  ↓
final transcript
```

---

# 30. STT Provider Contract

```rust
#[async_trait]
pub trait SpeechRecognitionProvider: Send + Sync {
    async fn start_stream(
        &self,
        config: SttConfig,
    ) -> SpeechResult<SttStream>;
}
```

---

# 31. STT Configuration

```rust
pub struct SttConfig {
    pub language: Option<LanguageCode>,
    pub sample_rate: u32,
    pub punctuation: bool,
    pub word_timestamps: bool,
    pub vocabulary: Vec<SpeechVocabularyEntry>,
}
```

---

# 32. STT Events

```rust
pub enum SttStreamEvent {
    Started,
    Partial(SttTranscript),
    Final(SttTranscript),
    Error(SpeechError),
    Completed,
}
```

---

# 33. Transcript Model

```rust
pub struct SttTranscript {
    pub text: String,
    pub confidence: Option<Confidence>,
    pub words: Vec<RecognizedWord>,
}
```

---

# 34. Word Timing

```rust
pub struct RecognizedWord {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: Option<Confidence>,
}
```

Word timing is useful for diagnostics and future synchronized UI.

---

# 35. Partial Transcript

Partial transcripts SHALL generally NOT become final TutorRequests.

They MAY be used for:

* live subtitles;
* early intent prediction;
* latency reduction;
* interruption classification.

---

# 36. Final Transcript

The finalized utterance SHALL become:

```rust
StudentInput::SpeechTranscript(...)
```

for the orchestrator.

---

# 37. STT Confidence

Low-confidence transcripts MAY trigger:

```text
confirmation
retranscription
alternate hypothesis
```

rather than silently assuming correctness.

---

# 38. Technical Vocabulary

Training systems frequently contain terms such as:

```text
std::mem::replace
kubectl
SYN-ACK
Win32
HRESULT
KerML
SysML
DuckDB
ConPTY
```

Generic speech recognition often performs poorly on these.

Nexa SHALL support domain vocabulary.

---

# 39. Vocabulary Entry

```rust
pub struct SpeechVocabularyEntry {
    pub written_form: String,
    pub spoken_variants: Vec<String>,
    pub boost: Option<f32>,
}
```

---

# 40. Course Vocabulary

Courses SHOULD be able to supply vocabulary packs.

```text
networking course
   ↓
TCP
UDP
SYN
ACK
CIDR
ARP
```

---

# 41. Dynamic Vocabulary

Active lesson context MAY temporarily boost relevant terminology.

This SHOULD improve recognition without permanently biasing all sessions.

---

# 42. Transcript Normalization

Spoken:

> "syn ack"

may normalize to:

```text
SYN-ACK
```

when lesson context supports that interpretation.

Normalization SHALL preserve access to raw transcription.

---

# 43. Raw Versus Normalized Transcript

```rust
pub struct NormalizedTranscript {
    pub raw: String,
    pub normalized: String,
    pub transformations: Vec<TranscriptTransformation>,
}
```

---

# 44. Transformation Example

```text
raw:
"tcp syn ack"

normalized:
"TCP SYN-ACK"

reason:
course vocabulary
```

---

# 45. Student Interruption

While Nexa is speaking:

```text
microphone detects speech
        ↓
is it student speech?
        ↓
interruption policy
        ↓
cancel playback
        ↓
cancel behavior
        ↓
Nexa → listening
```

---

# 46. Barge-In Detector

A dedicated detector SHOULD differentiate:

```text
student interruption
background noise
Nexa echo
short acknowledgment
```

---

# 47. Interruption Types

```rust
pub enum BargeInType {
    FullInterruption,
    Backchannel,
    Noise,
    Unknown,
}
```

---

# 48. Backchannel

Students may say:

```text
yeah
uh-huh
right
okay
```

while Nexa speaks.

These should not always cancel Nexa.

---

# 49. Backchannel Policy

A short acknowledgment MAY:

```text
leave speech running
+
slightly alter avatar acknowledgment
```

rather than trigger a new workflow.

---

# 50. Explicit Interruptions

Utterances such as:

```text
stop
wait
hold on
no
what?
```

SHOULD receive high interruption priority.

---

# 51. TTS Architecture

```text
TutorResponse
      ↓
Speech Planner
      ↓
Text Normalization
      ↓
Pronunciation Planning
      ↓
Prosody Planning
      ↓
TTS Provider
      ↓
Audio + timing
```

---

# 52. TTS Provider Contract

```rust
#[async_trait]
pub trait SpeechSynthesisProvider: Send + Sync {
    async fn synthesize(
        &self,
        request: TtsRequest,
    ) -> SpeechResult<TtsResult>;

    async fn stream(
        &self,
        request: TtsRequest,
    ) -> SpeechResult<TtsStream>;
}
```

---

# 53. TTS Request

```rust
pub struct TtsRequest {
    pub speech_id: SpeechId,
    pub text: String,

    pub voice: VoiceProfileId,
    pub style: SpeechStyle,

    pub pace: SpeechRate,
    pub pitch: SpeechPitch,
    pub energy: SpeechEnergy,

    pub pronunciation: Vec<PronunciationDirective>,
    pub request_visemes: bool,
}
```

---

# 54. Nexa Voice Identity

Nexa SHALL have a canonical voice identity independent of provider implementation.

Conceptually:

```yaml
voice:
  persona: nexa
  age_presentation: young_adult
  warmth: high
  confidence: high
  articulation: high
  technical_clarity: high
  energy: moderate
  baseline_pace: moderate
```

---

# 55. Voice Profile

```rust
pub struct VoiceProfile {
    pub id: VoiceProfileId,
    pub character_id: CharacterId,
    pub provider_voice_map: HashMap<ProviderId, String>,
    pub baseline: VoiceCharacteristics,
}
```

---

# 56. Provider Voice Mapping

Nexa may use different concrete voices across providers while preserving similar characteristics.

```text
Nexa canonical voice
     ├── local provider → voice A
     ├── cloud provider → voice B
     └── fallback       → voice C
```

---

# 57. Voice Consistency

Provider fallback SHOULD minimize abrupt personality changes.

Fallback selection SHOULD consider:

```text
timbre
pitch
pace
energy
accent
```

where possible.

---

# 58. Speech Style

Speech style comes from semantic tutoring behavior.

```rust
pub enum SpeechStyle {
    Neutral,
    Instructional,
    Conversational,
    Encouraging,
    Questioning,
    Serious,
    Warning,
    Excited,
    Reflective,
}
```

---

# 59. Prosody

Prosody SHOULD be controlled by high-level parameters.

```rust
pub struct Prosody {
    pub pace: SpeechRate,
    pub pitch: SpeechPitch,
    pub energy: SpeechEnergy,
    pub pauses: PausePolicy,
}
```

---

# 60. Contextual Prosody

Examples:

```text
complex concept
    → slower

warning
    → deliberate

celebration
    → higher energy

question
    → interrogative prosody
```

---

# 61. No Cartoon Exaggeration

Emotional speech SHOULD remain believable for a technical tutor.

Nexa SHOULD not dramatically alter voice for routine state changes.

---

# 62. Pronunciation Engine

Technical education requires explicit pronunciation planning.

---

# 63. Pronunciation Directive

```rust
pub struct PronunciationDirective {
    pub token: String,
    pub pronunciation: Pronunciation,
}
```

---

# 64. Pronunciation Types

```rust
pub enum Pronunciation {
    Phoneme(String),
    SpokenForm(String),
    SpellOut,
    Acronym,
    Literal,
}
```

---

# 65. Acronym Examples

```text
TCP
    → "T C P"

SYN
    → "sin" or "S Y N" depending course policy

API
    → "A P I"

SQL
    → configurable "sequel" / "S Q L"
```

Course-specific pronunciation SHALL be supported.

---

# 66. Code Pronunciation

Code SHOULD not always be spoken literally as raw syntax.

Example:

```rust
std::mem::replace
```

might be spoken:

> "std mem replace"

while display text retains the exact code.

---

# 67. Speech Versus Display Text

TutorResponse SHOULD permit:

```text
display:
std::mem::replace

speech:
"std mem replace"
```

This distinction is important.

---

# 68. Speech Normalization

Raw display text:

```text
Use `cargo test --workspace`.
```

Speech normalization may become:

> "Use cargo test dash dash workspace."

or a more natural technical pronunciation policy.

---

# 69. URL Handling

Long URLs SHOULD generally not be spoken verbatim unless explicitly useful.

Prefer:

> "I've displayed the documentation link."

---

# 70. Numbers

Number pronunciation SHOULD depend on context.

```text
192.168.1.1
```

should be spoken as an IP address rather than:

> "one hundred ninety-two point one hundred sixty-eight..."

---

# 71. Version Numbers

```text
Rust 1.82.0
```

should use a version-aware pronunciation policy.

---

# 72. Hexadecimal

```text
0xFF
```

may be spoken:

> "hex F F"

depending on lesson context.

---

# 73. Equations

Speech normalization SHOULD support mathematical notation where applicable.

---

# 74. TTS Streaming

The preferred interactive pipeline is:

```text
committed sentence
     ↓
TTS begins
     ↓
first audio chunk
     ↓
playback begins
     ↓
remaining audio streams
```

---

# 75. First-Audio Latency

The key metric is:

```text
speech chunk committed
      ↓
first playable audio
```

This SHALL be monitored.

---

# 76. Audio Stream

```rust
pub enum TtsStreamEvent {
    Started,
    Audio(TtsAudioChunk),
    Viseme(VisemeEvent),
    Boundary(SpeechBoundary),
    Completed,
    Error(SpeechError),
}
```

---

# 77. Audio Chunk

```rust
pub struct TtsAudioChunk {
    pub sequence: u64,
    pub offset_ms: u64,
    pub samples: AudioSamples,
}
```

---

# 78. Speech Boundaries

Providers SHOULD expose where possible:

```text
word
phrase
sentence
```

boundaries.

These support interruption and avatar timing.

---

# 79. Boundary Model

```rust
pub enum SpeechBoundaryType {
    Word,
    Phrase,
    Sentence,
}
```

---

# 80. Visemes

Lip synchronization SHOULD primarily use visemes.

Baseline set:

```text
REST
A
E
I
O
U
MBP
FV
L
WQ
TH
CHSH
R
```

Adapters MAY map provider-specific visemes into this canonical set.

---

# 81. Viseme Event

```rust
pub struct VisemeEvent {
    pub speech_id: SpeechId,
    pub viseme: Viseme,
    pub offset_ms: u64,
    pub duration_ms: u64,
    pub intensity: Option<f32>,
}
```

---

# 82. Lip-Sync Pipeline

```text
TTS
 │
 ├── audio ─────────────► playback
 │
 └── visemes
        ↓
     NBP adapter
        ↓
avatar mouth parameters
```

---

# 83. Synchronization Clock

Audio playback SHOULD normally be the master clock for lip synchronization.

Do not independently free-run lip animation.

---

# 84. Playback Position

```rust
pub struct PlaybackClock {
    pub speech_id: SpeechId,
    pub position_ms: u64,
}
```

Visemes SHALL be selected based on this clock.

---

# 85. Lookahead

The avatar runtime MAY receive a small future viseme buffer.

Example:

```text
audio position = 100 ms

available visemes:
100
160
240
310 ms
```

This enables smoother interpolation.

---

# 86. Viseme Blending

Hard switching:

```text
A → MBP → O
```

may look robotic.

The avatar system SHOULD interpolate mouth shapes where supported.

---

# 87. Missing Viseme Data

If the TTS provider supplies no visemes, fallback options include:

```text
phoneme alignment
text-to-phoneme prediction
audio-driven mouth opening
```

Preference:

```text
provider visemes
   ↓
phoneme alignment
   ↓
audio amplitude fallback
```

---

# 88. Audio-Driven Fallback

Amplitude-based lip sync is acceptable only as a degraded mode.

It cannot accurately represent mouth shapes.

---

# 89. Speech-Behavior Synchronization

A behavior plan may specify:

```text
gaze → target
pause
gesture
speech
```

NBP and speech systems SHALL share timing identifiers.

---

# 90. Shared Speech ID

```text
speech_id = sp_123
behavior_id = beh_456
correlation_id = interaction_789
```

The relationship SHALL be traceable.

---

# 91. Speech Start Event

Playback emits:

```text
speech.playback.started
```

The Behavior Engine may use that to enter:

```text
speaking / explaining
```

---

# 92. Speech Completion

```text
speech.playback.completed
      ↓
avatar mouth → REST
      ↓
follow-up behavior
```

---

# 93. Speech Cancellation

Cancellation SHALL stop:

* future TTS generation where possible;
* queued audio;
* current playback;
* future visemes;
* speech-dependent gestures as appropriate.

---

# 94. Cancellation Flow

```text
student interruption
       ↓
speech.cancel
       ↓
TTS generation cancelled
       ↓
audio buffer cleared
       ↓
viseme stream cancelled
       ↓
mouth → REST
       ↓
Nexa → listening
```

---

# 95. Audio Buffering

Too little buffering causes:

```text
audio underruns
```

Too much buffering increases:

```text
interruption latency
```

The runtime SHALL balance both.

---

# 96. Playback Buffer

```rust
pub struct PlaybackBufferConfig {
    pub minimum_buffer: Duration,
    pub target_buffer: Duration,
    pub maximum_buffer: Duration,
}
```

---

# 97. Adaptive Buffering

Buffer size MAY adapt to:

```text
provider jitter
network quality
local provider latency
device performance
```

---

# 98. Speech Queue

Nexa SHOULD generally have only one foreground speech stream.

```text
active speech
queued phrase
```

Large queues SHOULD be avoided because they make interruptions awkward.

---

# 99. Sentence Scheduling

Streaming Tutor output:

```text
Sentence 1
   ↓ TTS

Sentence 2
   ↓ TTS while sentence 1 plays

Sentence 3
```

can reduce gaps while preserving interruption.

---

# 100. Speech Ahead Limit

The runtime SHOULD limit how far ahead TTS is synthesized.

This avoids wasting work when the student interrupts.

---

# 101. Audio Cache

Repeated fixed phrases MAY be cached.

Examples:

```text
"Welcome back."
"Ready when you are."
```

Dynamic instructional speech generally should not require caching.

---

# 102. Cache Key

Speech cache keys SHOULD include:

```text
normalized text
voice profile
provider
style
pace
pitch
pronunciation directives
```

---

# 103. Voice Asset Privacy

Custom trained voice models MAY contain sensitive or licensed assets.

Voice model storage and licensing SHALL be handled separately from ordinary audio caching.

---

# 104. Local-First STT

The architecture SHOULD support local speech recognition providers.

Potential implementations may include Whisper-compatible or other local runtimes.

The contract SHALL not depend on one engine.

---

# 105. Local-First TTS

Likewise, local TTS SHALL be first class.

The runtime SHOULD permit:

```text
fully offline tutoring
```

when local models and course knowledge are installed.

---

# 106. Provider Registry

```rust
pub struct SpeechProviderRegistry {
    pub stt: HashMap<ProviderId, Arc<dyn SpeechRecognitionProvider>>,
    pub tts: HashMap<ProviderId, Arc<dyn SpeechSynthesisProvider>>,
}
```

---

# 107. Provider Selection

Selection MAY consider:

```text
latency
privacy
quality
language
viseme support
streaming
availability
hardware
```

---

# 108. STT Provider Capabilities

```rust
pub struct SttCapabilities {
    pub streaming: bool,
    pub partial_results: bool,
    pub word_timestamps: bool,
    pub vocabulary_biasing: bool,
    pub languages: Vec<LanguageCode>,
}
```

---

# 109. TTS Provider Capabilities

```rust
pub struct TtsCapabilities {
    pub streaming: bool,
    pub visemes: bool,
    pub phonemes: bool,
    pub word_boundaries: bool,
    pub styles: Vec<SpeechStyle>,
    pub languages: Vec<LanguageCode>,
}
```

---

# 110. Provider Failover

Example:

```text
local TTS
    ↓ fails
local fallback TTS
    ↓ fails
approved remote TTS
```

Failover SHALL obey deployment privacy policy.

---

# 111. No Remote Surprise

A local-only configuration SHALL NOT silently send speech or transcripts to a cloud provider.

---

# 112. Privacy Classification

Speech configurations SHOULD support:

```rust
pub enum SpeechPrivacyMode {
    LocalOnly,
    PreferLocal,
    ApprovedRemoteAllowed,
}
```

---

# 113. Recording Policy

Live microphone audio SHALL NOT automatically be retained permanently.

Retention SHALL be explicit.

Possible modes:

```rust
pub enum AudioRetentionPolicy {
    DoNotStore,
    SessionTemporary,
    StoreWithConsent,
}
```

---

# 114. Transcript Retention

Transcript retention may differ from raw-audio retention.

The session system MAY store text while discarding audio.

---

# 115. Speech Events

Canonical events include:

```text
audio.device.connected
audio.device.disconnected

speech.capture.started
speech.capture.completed
speech.capture.cancelled

speech.vad.started
speech.vad.ended

speech.transcription.started
speech.transcription.partial
speech.transcription.completed
speech.transcription.failed

speech.synthesis.requested
speech.synthesis.started
speech.synthesis.completed
speech.synthesis.failed
speech.synthesis.cancelled

speech.playback.started
speech.playback.completed
speech.playback.cancelled
speech.playback.underrun

speech.viseme.emitted

speech.barge_in.detected
```

---

# 116. Capture Started Example

```json
{
  "event_type": "speech.capture.started",
  "payload": {
    "device_id": "mic-primary",
    "sample_rate": 48000
  }
}
```

---

# 117. Transcription Completed Example

```json
{
  "event_type": "speech.transcription.completed",
  "payload": {
    "segment_id": "seg-112",
    "text": "What happens after SYN?",
    "confidence": 0.94
  }
}
```

---

# 118. TTS Completion Example

```json
{
  "event_type": "speech.synthesis.completed",
  "payload": {
    "speech_id": "sp-829",
    "duration_ms": 4120,
    "viseme_count": 84
  }
}
```

---

# 119. Speech Metrics

Measure:

```text
speech-start detection latency
endpoint latency
STT first-partial latency
STT final latency
word error rate
technical-term accuracy
TTS first-audio latency
TTS real-time factor
playback underruns
barge-in detection latency
speech cancellation latency
lip-sync offset error
```

---

# 120. Word Error Rate

General WER alone is insufficient.

Nexa SHOULD separately track:

```text
technical vocabulary accuracy
```

because one wrong technical term can alter meaning dramatically.

---

# 121. Technical Term Accuracy

Example:

```text
expected:
SYN-ACK

recognized:
sync act
```

This SHALL count as a domain-recognition failure even if general sentence WER is low.

---

# 122. Pronunciation Evaluation

TTS testing SHOULD include terms such as:

```text
SYN-ACK
CIDR
HRESULT
std::mem::replace
ConPTY
KerML
SysML
DuckDB
```

---

# 123. Lip-Sync Metric

A useful metric:

```text
absolute timing difference
between
audio phoneme onset
and
avatar viseme onset
```

---

# 124. Synchronization Target

The acceptable threshold SHALL depend on runtime quality goals, but visible drift SHOULD be minimized.

---

# 125. Latency Budget

A conversational latency budget MAY be decomposed:

```text
endpoint detection
      +
STT finalization
      +
Tutor first chunk
      +
TTS first audio
      =
student-to-Nexa response latency
```

Each component SHALL be measurable separately.

---

# 126. Visual Latency Masking

The avatar may enter `THINKING` immediately after student speech ends.

This provides feedback while the Tutor Engine and TTS pipeline operate.

---

# 127. Speech Failure Classification

```rust
pub enum SpeechError {
    DeviceUnavailable,
    CaptureFailed,
    VadFailed,
    SttUnavailable,
    SttFailed,
    TtsUnavailable,
    TtsFailed,
    PlaybackFailed,
    UnsupportedLanguage,
    UnsupportedVoice,
    Timeout,
    Cancelled,
}
```

---

# 128. STT Failure Fallback

If STT fails:

```text
student may use text input
```

Nexa SHOULD communicate the degraded capability without terminating the session.

---

# 129. TTS Failure Fallback

If TTS fails:

```text
display TutorResponse text
+
optional silent avatar behavior
```

---

# 130. Playback Failure

If output device fails during speech:

```text
pause/cancel audio
      ↓
notify orchestrator
      ↓
switch output if available
```

---

# 131. Viseme Failure

If viseme generation fails:

```text
speech continues
+
fallback lip sync
```

The tutoring interaction SHALL not fail solely due to lip-sync degradation.

---

# 132. Degradation Hierarchy

Preferred:

```text
voice + accurate visemes
        ↓
voice + predicted visemes
        ↓
voice + amplitude lip sync
        ↓
voice only
        ↓
text + avatar
        ↓
text only
```

---

# 133. Speech State Machine

```text
IDLE
 ↓
CAPTURING
 ↓
TRANSCRIBING
 ↓
WAITING_FOR_RESPONSE
 ↓
SYNTHESIZING
 ↓
PLAYING
 ↓
IDLE
```

With interruption:

```text
PLAYING
   ↓
BARGE_IN
   ↓
CANCELLING
   ↓
CAPTURING
```

---

# 134. Speech Runtime State

```rust
pub enum SpeechRuntimeState {
    Idle,
    Capturing,
    Transcribing,
    Waiting,
    Synthesizing,
    Playing,
    Cancelling,
    Degraded,
    Failed,
}
```

---

# 135. Speech Session

```rust
pub struct SpeechSession {
    pub session_id: SessionId,
    pub state: SpeechRuntimeState,

    pub active_capture: Option<SpeechSegmentId>,
    pub active_speech: Option<SpeechId>,

    pub input_device: Option<AudioDeviceId>,
    pub output_device: Option<AudioDeviceId>,
}
```

---

# 136. Threading Model

Audio capture and playback SHALL not block AI or orchestration threads.

Real-time audio processing SHOULD run through dedicated asynchronous or audio-safe execution paths.

---

# 137. Real-Time Safety

Audio callbacks SHOULD avoid:

```text
blocking I/O
large allocations
database calls
LLM requests
```

They SHOULD push frames into bounded buffers.

---

# 138. Backpressure

Audio streams SHALL use bounded queues.

If consumers fall behind:

```text
capture overflow
```

SHOULD produce telemetry rather than unbounded memory growth.

---

# 139. Dropping Audio

Raw live audio frames may sometimes be dropped under overload.

Critical lifecycle events SHALL not be dropped.

---

# 140. Speech Worker Architecture

```text
Audio I/O
   │
   ▼
Real-Time Worker
   │
   ▼
Bounded Audio Queue
   │
   ▼
Speech Processing Worker
   │
   ├── VAD
   ├── STT
   └── events
```

---

# 141. TTS Worker

```text
Speech Plan
    ↓
TTS Worker
    ↓
Audio Buffer
    ↓
Playback Worker
```

---

# 142. Speech Provider Isolation

Provider crashes or slowdowns SHOULD not block the session actor itself.

---

# 143. Language Support

Speech configuration SHOULD allow explicit course/session language.

```rust
pub struct SpeechLanguageContext {
    pub spoken_language: LanguageCode,
    pub technical_vocabulary_language: Option<LanguageCode>,
}
```

---

# 144. Multilingual Technical Training

A learner may speak one language while source-code identifiers remain English.

The pronunciation system SHALL preserve identifiers.

---

# 145. Speaker Identity

MVP assumes one student.

Future versions MAY support multiple speakers.

```rust
pub struct SpeakerId(String);
```

---

# 146. Speaker Diarization

Classroom or multi-user mode MAY require speaker diarization.

This is outside MVP scope but SHALL remain architecturally possible.

---

# 147. Emotion Recognition

Nexa SHOULD NOT initially infer learner emotion directly from voice.

Such classification can be unreliable and unnecessary.

Future optional signals SHALL remain probabilistic and governed.

---

# 148. Voice Cloning Boundary

If custom voice cloning is ever supported, it SHALL be a separate controlled capability with explicit authorization and provenance.

The speech architecture does not require voice cloning.

---

# 149. Canonical Nexa Voice Asset

Nexa SHOULD have:

```text
voice specification
+
pronunciation lexicon
+
provider mappings
+
test utterances
```

stored as versioned assets.

---

# 150. Pronunciation Lexicon

Recommended structure:

```text
speech/
└── lexicons/
    ├── nexa-core.yaml
    ├── networking.yaml
    ├── rust.yaml
    ├── cybersecurity.yaml
    └── course-specific/
```

---

# 151. Lexicon Example

```yaml
entries:
  - token: "SYN-ACK"
    spoken: "sin ack"

  - token: "TCP"
    spoken: "T C P"

  - token: "DuckDB"
    spoken: "duck D B"

  - token: "ConPTY"
    spoken: "con P T Y"
```

Pronunciations SHALL be configurable.

---

# 152. Speech Markup

The system MAY internally support provider-neutral speech markup.

Example:

```text
[pause=200ms]
[emphasis=moderate]
[pronounce token="TCP" form="T C P"]
```

Provider adapters translate it where possible.

---

# 153. Provider-Neutral Speech Plan

```rust
pub struct SpeechPlan {
    pub segments: Vec<SpeechSegmentPlan>,
}
```

---

# 154. Speech Segment Plan

```rust
pub enum SpeechSegmentPlan {
    Text(String),
    Pause(Duration),
    Emphasis {
        text: String,
        level: EmphasisLevel,
    },
    Pronounced {
        display: String,
        spoken: String,
    },
}
```

---

# 155. Display/Speech Separation Example

Display:

```text
cargo test --workspace
```

Speech:

> "cargo test dash dash workspace"

Canvas remains exact.

Voice becomes understandable.

---

# 156. Avatar Integration

Speech SHALL expose:

```text
speech_started
viseme_stream
word boundaries
speech_completed
speech_cancelled
```

to the avatar subsystem.

---

# 157. Facial Speech Behavior

Speaking does not only control the mouth.

The avatar MAY also use:

```text
subtle head motion
eye behavior
blinks
breathing
gesture timing
```

through NBP.

Speech does not own those behaviors.

---

# 158. Lip Sync Ownership

Speech system owns:

```text
linguistic timing
```

Avatar runtime owns:

```text
visual parameter mapping
```

This boundary SHALL remain stable.

---

# 159. NBP Integration

Conceptually:

```text
Speech System
    ↓
canonical VisemeEvents
    ↓
NBP/Avatar Adapter
    ↓
ParamMouthOpen
ParamMouthForm
etc.
```

The speech subsystem SHALL not depend on Live2D-specific parameters.

---

# 160. Replay

Recorded speech timing SHOULD support avatar replay without regenerating TTS.

Store where appropriate:

```text
audio artifact
viseme timeline
speech boundaries
voice profile version
```

---

# 161. Deterministic Playback Test

A recorded speech fixture SHOULD reproduce:

```text
same audio
same viseme timing
same completion boundaries
```

for avatar regression tests.

---

# 162. Speech Test Fixtures

Recommended fixtures include:

```text
short sentence
long sentence
technical acronym
code
IP address
version number
interrupted speech
rapid student interruption
background noise
low-confidence STT
provider failure
```

---

# 163. STT Regression Set

A technical corpus SHOULD contain recordings for:

```text
Rust
networking
cybersecurity
shell commands
Windows APIs
architecture terminology
```

---

# 164. TTS Golden Set

Canonical Nexa phrases SHOULD be evaluated across providers for:

```text
clarity
pronunciation
pace
emotion
consistency
```

---

# 165. Speech Architecture Crate

Recommended structure:

```text
crates/
└── nexa-speech/
    ├── src/
    │   ├── lib.rs
    │   ├── service.rs
    │   ├── device.rs
    │   ├── capture.rs
    │   ├── playback.rs
    │   ├── audio.rs
    │   ├── resample.rs
    │   ├── vad.rs
    │   ├── endpoint.rs
    │   ├── echo.rs
    │   ├── stt.rs
    │   ├── transcript.rs
    │   ├── normalize.rs
    │   ├── vocabulary.rs
    │   ├── tts.rs
    │   ├── voice.rs
    │   ├── prosody.rs
    │   ├── pronunciation.rs
    │   ├── viseme.rs
    │   ├── lipsync.rs
    │   ├── interruption.rs
    │   ├── buffering.rs
    │   ├── cache.rs
    │   ├── metrics.rs
    │   ├── errors.rs
    │   └── providers/
    │       ├── mod.rs
    │       ├── mock_stt.rs
    │       ├── mock_tts.rs
    │       ├── local_stt.rs
    │       └── local_tts.rs
    └── tests/
        ├── vad.rs
        ├── endpoint.rs
        ├── transcription.rs
        ├── pronunciation.rs
        ├── interruption.rs
        ├── viseme.rs
        ├── playback.rs
        └── latency.rs
```

---

# 166. Dependency Direction

```text
                   nexa-domain
                       │
                       ▼
                   nexa-speech
                    /       \
                   ▼         ▼
              nexa-events  provider adapters
                   │
                   ▼
            nexa-orchestrator
                   │
                   ▼
                nexa-nbp
                   │
                   ▼
              avatar runtime
```

---

# 167. MVP Scope

The first speech implementation SHOULD support:

```text
one microphone
one output device

push-to-talk

one STT provider
one TTS provider

final transcription
no full duplex initially

technical vocabulary

streamed or low-latency TTS

basic visemes

playback cancellation

text fallback
```

---

# 168. MVP Vertical Slice

```text
Student presses microphone button
       ↓
capture begins
       ↓
student speaks
       ↓
release button
       ↓
STT
       ↓
"What happens after SYN?"
       ↓
Tutor Engine
       ↓
"The server responds with SYN-ACK."
       ↓
TTS
       ↓
audio + visemes
       ↓
Nexa lip-syncs response
```

That is the first complete spoken interaction.

---

# 169. Phase 2

Add:

```text
automatic VAD
endpointing
partial transcripts
sentence-streamed TTS
technical pronunciation packs
adaptive buffering
```

---

# 170. Phase 3

Add:

```text
full duplex
echo cancellation
barge-in
backchannel detection
provider failover
advanced prosody
multi-language
```

---

# 171. Acceptance Scenario

Student says:

> "Can you explain SYN-ACK again?"

Expected:

```text
student.speech.started
       ↓
capture
       ↓
STT final:
"Can you explain SYN-ACK again?"
       ↓
TutorResponse generated
       ↓
SpeechPlan:
spoken pronunciation for SYN-ACK
       ↓
TTS begins
       ↓
avatar → explaining
       ↓
viseme stream
       ↓
Nexa says:
"Sure. SYN-ACK is the server's response..."
       ↓
speech completed
       ↓
avatar → attentive
```

---

# 172. Interruption Acceptance Scenario

Nexa says:

> "The first phase begins when the client—"

Student says:

> "Wait, what does client mean here?"

Expected:

```text
student voice detected
       ↓
barge-in classified
       ↓
speech playback cancelled
       ↓
visemes cancelled
       ↓
avatar → listening
       ↓
STT
       ↓
new TutorRequest
```

The previous response remains recorded as interrupted rather than completed.

---

# 173. Technical Vocabulary Acceptance Scenario

Student says:

> "What's the difference between SYN and SYN-ACK?"

The transcript SHOULD NOT become:

```text
"what's the difference between sin and sync act"
```

when course vocabulary is active.

---

# 174. Lip-Sync Acceptance Scenario

Given:

```text
audio artifact
+
canonical viseme timeline
```

the avatar SHALL replay mouth movement with no visually significant drift under normal runtime conditions.

---

# 175. Speech Invariants

`NEXA-SPCH-001` establishes these invariants:

1. Speech SHALL remain separate from Tutor intelligence.
2. Audio-device details SHALL remain outside domain reasoning.
3. Speech recognition SHALL support local providers.
4. Speech synthesis SHALL support local providers.
5. Cloud speech SHALL not be mandatory.
6. Local-only privacy policies SHALL not silently use remote providers.
7. Raw microphone audio SHALL not be permanently retained by default.
8. Partial transcripts SHALL not normally become final learner input.
9. Technical vocabulary SHALL be first-class.
10. Raw and normalized transcripts SHOULD remain distinguishable.
11. TTS display text and spoken text MAY differ.
12. Pronunciation SHALL be explicitly controllable.
13. Speech style SHALL remain semantic and provider-independent.
14. Nexa's canonical voice identity SHALL remain provider-independent.
15. TTS provider implementations SHALL be replaceable.
16. STT provider implementations SHALL be replaceable.
17. Audio playback SHALL normally be the lip-sync master clock.
18. Visemes SHALL be canonicalized before avatar-specific mapping.
19. Accurate visemes SHOULD be preferred over amplitude lip sync.
20. Lip-sync failure SHALL degrade gracefully.
21. Student interruption SHALL support bounded cancellation.
22. Backchannels SHOULD not always interrupt Nexa.
23. Audio callbacks SHALL avoid blocking runtime work.
24. Audio queues SHALL be bounded.
25. Speech latency SHALL be observable at component boundaries.
26. First-audio latency SHALL be a primary performance metric.
27. Speech events SHALL integrate with NEXA-EVT-001.
28. Avatar behavior SHALL remain controlled through semantic behavior interfaces.
29. Speech cancellation SHALL cancel future viseme timing.
30. Speech testing SHALL include domain-specific technical terminology.

---

# 176. Architecture Status

Nexa now has:

```text
Student
   │
   ├──── voice ───────► NEXA-SPCH-001
   │                       │
   │                       ▼
   │                    transcript
   │                       │
   ▼                       ▼
NEXA-STU-001 ─────► NEXA-PED-001
                         │
                         ▼
                  NEXA-KNOW-001
                         │
                         ▼
                  NEXA-TUTOR-001
                         │
                         ▼
                  NEXA-ORCH-001
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Speech      Avatar     Canvas
```

We have now defined the full path from **the learner's spoken question to Nexa's synchronized spoken response**.

---

# 177. Next Specification

The next specification should be:

# **NEXA-AVTR-001 — Avatar Runtime, Rigging, Animation, Expression & Behavior Engine Architecture v1.0**

That specification should finally formalize Nexa's visual embodiment:

```text
canonical character assets
2D/2.5D architecture
layer decomposition
rig parameter model
face rig
eye/gaze system
mouth/viseme mapping
body rig
gesture system
idle behavior
micro-behaviors
hair/clothing physics
animation blending
expression blending
behavior arbitration
NBP adapter
state machine
canvas pointing
runtime capabilities
render loop
lip-sync synchronization
Live2D-style adapter
future VRM/3D adapter
asset versioning
performance budgets
fallback behavior
avatar testing
```

That is the next major milestone because it moves us from **Nexa having a voice** to **Nexa becoming an actually animated character on screen**.

---

## 2026-08-26 ADR-0069 v1 reconciliation

Speech input and output are required v1 capabilities. Nexa bundles and manages the selected speech models/runtime behind this provider-neutral boundary, and the result must operate on the CPU-only Windows reference PC. Sherpa-ONNX is an evidence-gated candidate, not an accepted adapter. Its bounded spike measures recognition accuracy, latency, synthesis quality, memory, package size, interruption/cancellation, and lip-sync timing. If disproved, `whisper.cpp` proceeds only as the recognition fallback candidate and TTS requires a separately governed selection. Existing contracts/cancellation evidence is preserved but does not prove a concrete adapter or system maturity.
