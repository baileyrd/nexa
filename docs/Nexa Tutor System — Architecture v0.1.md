# Nexa Tutor System — Architecture v0.1

  ```text
  ┌─────────────────────────────────────────────────────────────────────┐
  │                           NEXA CLIENT                               │
  │                                                                     │
  │   ┌──────────────┐   ┌─────────────────┐   ┌───────────────────┐   │
  │   │ Nexa Avatar  │   │ Training Canvas │   │ Terminal / Labs   │   │
  │   │              │   │                 │   │                   │   │
  │   │ face/body    │   │ diagrams        │   │ code              │   │
  │   │ lip sync     │   │ slides          │   │ shell             │   │
  │   │ expressions  │   │ annotations     │   │ simulations       │   │
  │   └──────┬───────┘   └────────┬────────┘   └─────────┬─────────┘   │
  │          │                    │                      │             │
  └──────────┼────────────────────┼──────────────────────┼─────────────┘
            │                    │                      │
            └────────────────────┼──────────────────────┘
                                  ▼
  ┌─────────────────────────────────────────────────────────────────────┐
  │                       SESSION ORCHESTRATOR                          │
  │                                                                     │
  │ conversation │ lesson │ student state │ tool state │ avatar state  │
  └───────────┬────────────┬───────────────┬────────────┬────────────────┘
              │            │               │            │
        ┌────▼────┐ ┌─────▼─────┐  ┌─────▼─────┐ ┌───▼─────────┐
        │ Tutor   │ │ Pedagogy  │  │ Student   │ │ Behavior    │
        │ Engine  │ │ Engine    │  │ Model     │ │ Engine      │
        └────┬────┘ └─────┬─────┘  └─────┬─────┘ └───┬─────────┘
              │            │               │            │
              ▼            ▼               ▼            ▼
  ┌─────────────────────────────────────────────────────────────────────┐
  │                         INTELLIGENCE                                │
  │                                                                     │
  │ LLM │ RAG │ Knowledge Graph │ Lesson Planner │ Evaluator │ Memory │
  └─────────────────────────────────────────────────────────────────────┘
              │
              ▼
  ┌─────────────────────────────────────────────────────────────────────┐
  │                       TRAINING PLATFORM                             │
  │                                                                     │
  │ Content │ Courses │ Labs │ Quizzes │ Competencies │ Analytics      │
  └─────────────────────────────────────────────────────────────────────┘
  ```
  
  ### The central design rule
  
  The LLM should **never directly control animation**.
  
  Instead, Nexa's intelligence produces semantic behavior:
  
  ```json
  {
    "speech": "Exactly. SYN is the first step of the TCP handshake.",
    "intent": "explain",
    "emotion": "encouraging",
    "attention_target": "tcp_diagram",
    "gesture": "point",
    "importance": 0.7
  }
  ```
  
  The avatar subsystem decides what animation corresponds to that.
  
  ```text
  explain
    +
  encouraging
    +
  point
    +
  tcp_diagram
  
          ↓
  
  Behavior Engine
  
          ↓
  
  expression = smile_soft
  eyes       = tcp_diagram
  body       = lean_forward
  right_arm  = point_screen
  voice      = warm_explanatory
  animation  = explain_point_03
  ```
  
  That means we can replace Live2D with 3D someday without rewriting Nexa's brain.
  
  ---
  
  # 1. Nexa's runtime states
  
  I want Nexa to have a formal state machine rather than random LLM-driven behavior.
  
  ```text
                          ┌───────────────┐
                          │    START      │
                          └───────┬───────┘
                                  ▼
                          ┌───────────────┐
                    ┌─────►│     IDLE      │◄──────────┐
                    │      └───────┬───────┘           │
                    │              │                   │
                    │              ▼                   │
                    │      ┌───────────────┐           │
                    │      │   LISTENING   │           │
                    │      └───────┬───────┘           │
                    │              ▼                   │
                    │      ┌───────────────┐           │
                    │      │   THINKING    │           │
                    │      └───────┬───────┘           │
                    │              ▼                   │
                    │      ┌───────────────┐           │
                    │      │   SPEAKING    │───────────┘
                    │      └───────┬───────┘
                    │              │
        ┌──────────┼──────────────┼───────────────┐
        ▼          ▼              ▼               ▼
    EXPLAINING   ASKING        CORRECTING      CELEBRATING
        │          │              │               │
        └──────────┴──────────────┴───────────────┘
                              │
                              ▼
                              IDLE
  ```
  
  Additional states can later include:
  
  * `demonstrating`
  * `observing_lab`
  * `waiting_for_answer`
  * `hinting`
  * `warning`
  * `debugging`
  * `reviewing`
  * `challenging`
  * `encouraging`
  * `summarizing`
  * `storytelling`
  
  Each state gets an allowable set of expressions and gestures.
  
  ---
  
  # 2. Nexa Behavior Engine
  
  This becomes one of the most important components.
  
  The Tutor Engine might say:
  
  ```text
  emotion = curious
  activity = listening
  attention = student
  ```
  
  The Behavior Engine turns that into:
  
  ```text
  eyes           → student/camera
  head           → slight tilt
  mouth          → closed
  brows          → raised slightly
  body           → relaxed
  idle animation → listening_02
  ```
  
  The AI doesn't micromanage the character.
  
  That makes Nexa feel much more natural.
  
  ---
  
  # 3. Emotion model
  
  Rather than giving Nexa dozens of unrelated emotions, I'd use a small composable model.
  
  ```text
  Valence
  negative ◄──────────────► positive
  
  Arousal
  calm     ◄──────────────► excited
  
  Confidence
  uncertain ◄─────────────► confident
  
  Engagement
  passive  ◄──────────────► highly engaged
  ```
  
  An emotional state can therefore be represented numerically:
  
  ```json
  {
    "valence": 0.72,
    "arousal": 0.38,
    "confidence": 0.91,
    "engagement": 0.84
  }
  ```
  
  The avatar system derives facial behavior from that.
  
  This gives us subtle variations rather than eight permanently canned faces.
  
  ---
  
  # 4. Speech pipeline
  
  The voice loop should eventually be fully streaming.
  
  ```text
  Student speaks
        │
        ▼
  Microphone
        │
        ▼
  Voice Activity Detection
        │
        ▼
  Speech-to-Text
        │
        ▼
  Session Orchestrator
        │
        ▼
  Tutor Engine
        │
        ▼
  Streaming response
        │
        ├──────────────► Avatar Behavior
        │
        ▼
  Streaming Text-to-Speech
        │
        ▼
  Phonemes / Visemes
        │
        ▼
  Nexa lip synchronization
        │
        ▼
  Audio output
  ```
  
  Ideally Nexa starts speaking before the entire response has been generated.
  
  That dramatically improves the feeling of a real conversation.
  
  ---
  
  # 5. Nexa's Tutor Engine
  
  The tutor LLM should not merely receive:
  
  ```text
  User: Explain TCP.
  ```
  
  Its context should look more like:
  
  ```yaml
  student:
    skill_level: beginner
    preferred_explanation_depth: moderate
  
  course:
    name: Networking Fundamentals
  
  lesson:
    topic: TCP
    objective: Understand connection establishment
  
  current_competencies:
    osi_model: 0.82
    ip_addressing: 0.76
    tcp: 0.31
  
  recent_errors:
    - confused TCP with UDP
    - forgot SYN/ACK ordering
  
  teaching_strategy:
    mode: guided_discovery
    difficulty: adaptive
  
  nexa:
    current_emotion: encouraging
    state: explaining
  ```
  
  Now Nexa knows **who she is teaching**, not merely what question was asked.
  
  ---
  
  # 6. Pedagogy Engine
  
  This separates teaching logic from general-purpose LLM reasoning.
  
  Instead of Nexa constantly lecturing, the Pedagogy Engine controls the interaction pattern.
  
  For example:
  
  ```text
  INTRODUCE
      ↓
  EXPLAIN
      ↓
  DEMONSTRATE
      ↓
  ASK
      ↓
  OBSERVE
      ↓
  EVALUATE
      ↓
      ├── incorrect → HINT → RETRY
      │
      ├── partial   → CLARIFY → RETRY
      │
      └── correct   → REINFORCE
                          ↓
                    NEXT CONCEPT
  ```
  
  That will make her behave like an instructor instead of ChatGPT with an animated face.
  
  ---
  
  # 7. Student Model
  
  Every learner gets a persistent profile.
  
  Instead of simply recording scores, we'll maintain a competency graph.
  
  ```text
  Networking
  │
  ├── OSI Model                 92%
  │
  ├── IPv4                      84%
  │
  ├── Subnetting                63%
  │
  ├── TCP                       41%
  │   ├── handshake             71%
  │   ├── sequencing            36%
  │   ├── congestion control    22%
  │   └── connection teardown   48%
  │
  └── UDP                       73%
  ```
  
  Nexa can therefore say:
  
  > "Before we move into congestion control, I want to reinforce TCP sequencing because that's where you're still having trouble."
  
  That's a much richer training experience.
  
  ---
  
  # 8. Memory
  
  Nexa should have multiple memory scopes.
  
  | Memory           |     Lifetime | Example                     |
  | ---------------- | -----------: | --------------------------- |
  | Turn memory      |      seconds | Current sentence            |
  | Conversation     |      session | Current discussion          |
  | Lesson memory    |       lesson | Questions and mistakes      |
  | Course memory    | weeks/months | Training progression        |
  | Learner memory   |   persistent | Preferences and competency  |
  | Knowledge memory |   persistent | Training/reference material |
  
  Critically, **student memory and Nexa's knowledge base remain separate**.
  
  ---
  
  # 9. Knowledge architecture
  
  I would use hybrid retrieval.
  
  ```text
  Training Material
        │
        ├── documents
        ├── diagrams
        ├── source code
        ├── videos/transcripts
        ├── standards
        ├── lab instructions
        └── instructor notes
              │
              ▼
        Content Processor
              │
        ┌─────┼────────┐
        ▼     ▼        ▼
      Text  Metadata Concepts
        │     │        │
        ▼     ▼        ▼
      Vector  SQL   Knowledge Graph
        │     │        │
        └─────┼────────┘
              ▼
              RAG
              │
              ▼
          Tutor Engine
  ```
  
  The knowledge graph becomes particularly useful for prerequisites.
  
  ```text
  TCP
  ├── requires → IP
  ├── requires → ports
  ├── contrasts_with → UDP
  ├── contains → handshake
  └── contains → congestion_control
  ```
  
  ---
  
  # 10. Interactive training tools
  
  This is where Nexa can become much more interesting than existing AI tutors.
  
  The initial platform should ultimately support **code execution, terminal interaction, diagrams, quizzes, simulations, file inspection, sandbox labs, guided exercises, hints, competency testing, adaptive lesson generation, and progress analytics**.
  
  For a cybersecurity course, for example, Nexa could say:
  
  > "I've started a sandbox machine for you. Your task is to identify which ports are exposed. Tell me what command you're going to use first."
  
  Then she watches what happens.
  
  ```text
  Student
    │
    ▼
  Terminal Sandbox
    │
    ├── command
    ├── stdout
    ├── stderr
    └── exit status
            │
            ▼
        Lab Observer
            │
            ▼
        Tutor Engine
            │
            ▼
          Nexa
  ```
  
  She doesn't have to tell the student the answer immediately.
  
  ---
  
  # 11. Proposed internal event bus
  
  We should make the entire application event-driven.
  
  Examples:
  
  ```text
  student.speech.started
  student.speech.completed
  
  student.answer.submitted
  student.answer.correct
  student.answer.incorrect
  
  tutor.response.started
  tutor.response.token
  tutor.response.completed
  
  lesson.started
  lesson.completed
  
  lab.command.executed
  lab.error.detected
  
  avatar.state.changed
  
  speech.started
  speech.viseme
  speech.completed
  ```
  
  Individual modules subscribe only to the events they need.
  
  That will make this architecture highly extensible.
  
  ---
  
  # 12. Nexa Behavior Protocol
  
  I'm going to suggest we formalize a protocol between the tutor and avatar.
  
  Call it:
  
  **NBP — Nexa Behavior Protocol**
  
  Example message:
  
  ```json
  {
    "version": "1.0",
    "event": "nexa.behavior",
    "timestamp": 1787000000,
    "behavior": {
      "state": "explaining",
      "emotion": {
        "valence": 0.72,
        "arousal": 0.42,
        "confidence": 0.88
      },
      "gaze": {
        "target": "diagram.tcp_handshake"
      },
      "gesture": {
        "type": "point",
        "target": "diagram.syn_ack"
      },
      "speech": {
        "text": "The server responds with SYN-ACK.",
        "style": "instructional"
      }
    }
  }
  ```
  
  That protocol becomes a boundary between the AI and rendering systems.
  
  ---
  
  # 13. Avatar implementation
  
  For Nexa v1, I would start with:
  
  ```text
  Nexa Artwork
      ↓
  Layer Separation
      ↓
  2D Rig
      ↓
  Facial Parameters
      ↓
  Body Parameters
      ↓
  Animation Controller
      ↓
  Behavior Engine
  ```
  
  Typical rig controls:
  
  ```text
  HeadX
  HeadY
  HeadZ
  
  EyeX
  EyeY
  
  EyeOpenLeft
  EyeOpenRight
  
  BrowLeft
  BrowRight
  
  MouthOpen
  MouthShape
  
  BodyX
  BodyY
  
  Breathing
  
  HairPhysics
  ClothingPhysics
  AccessoryPhysics
  ```
  
  Later we can replace this with a full 3D Nexa.
  
  ---
  
  # 14. Application boundaries
  
  I'd structure the source code into independently replaceable subsystems:
  
  ```text
  nexa/
  │
  ├── avatar/
  ├── behavior/
  ├── speech/
  ├── tutor/
  ├── pedagogy/
  ├── student/
  ├── knowledge/
  ├── memory/
  ├── lessons/
  ├── assessment/
  ├── labs/
  ├── tools/
  ├── orchestration/
  ├── events/
  ├── persistence/
  ├── api/
  └── ui/
  ```
  
  That is deliberately modular because Nexa may eventually become a platform rather than a single application.
  
  ---
  
  # 15. First working Nexa
  
  The first milestone should **not** try to implement everything.
  
  Our first vertical slice should be:
  
  ```text
                    ┌───────────────┐
                    │     USER      │
                    └───────┬───────┘
                            │ text
                            ▼
                    ┌───────────────┐
                    │ Tutor Engine  │
                    └───────┬───────┘
                            │
                ┌────────────┴────────────┐
                │                         │
                ▼                         ▼
          response text             behavior state
                │                         │
                ▼                         ▼
            TTS Engine              Nexa Avatar
                │                         │
                ├──── visemes ────────────►
                │
                ▼
              audio
  ```
  
  That gives us the first genuinely useful Nexa:
  
  **she receives a question → reasons about it → answers → speaks → lip-syncs → changes expression and gesture while teaching.**
  
  Once that loop works, everything else can grow around it.
