# ADR-0012: Governed curriculum ownership and lesson transitions

- **Status:** Accepted
- **Date:** 2026-08-19
- **Governing specifications:** NEXA-LESSON-001, NEXA-PED-001, NEXA-DOM-001, NEXA-EVT-001

## Context

The reconstructed lesson specification separates authored definitions from progress and requires
curriculum-constrained adaptive routing, deterministic branches, prerequisites, resume, and
versioning. It illustrates richer content, completion, and branching shapes without resolving their
wire semantics. This slice must not turn those examples into tutor, assessment, media, or
orchestration behavior.

## Decision

`nexa-lessons` owns validated curriculum, course, module, lesson, step, prerequisite, objective
mapping, and progress contracts. `nexa-domain` owns only canonical UUID identities needed across
events and subsystem boundaries. A validated `Curriculum` exposes no mutation API: publishing a
change creates a separately validated authored definition. Student-specific mutable
`LessonProgress` is separate and may be stored through a synchronous repository port.

Version 1.0 uses lesson-to-lesson prerequisites. Every prerequisite must resolve within the
curriculum; self edges and cycles are invalid. A lesson can start only when the caller's read-only set
of completed lessons contains every prerequisite. Stable UUID ordering breaks topological ties.
Cross-course prerequisites, competency-threshold prerequisites, and what invalidates a previously
satisfied prerequisite remain unresolved.

`LessonPolicyV1` is pure and explicitly versioned as `1.0`. The caller supplies time and identity;
the policy does not use clocks, randomness, I/O, or global state. It reads a validated
`nexa-pedagogy::PedagogyDecision`, maps only options explicitly authored on the current step, and
returns a new progress value. It never mutates pedagogy or mastery. Assessment routing is
incompatible because scoring/evidence are not implemented. Failures return structured errors and
leave the input value untouched.

The baseline's `NotStarted`, `InProgress`, `Paused`, `Completed`, and `Abandoned` are represented by
`NotStarted`, `Active`, `Waiting`, `Completed`, and `Abandoned`. `Blocked` is added as a terminal,
explicit failure outcome rather than silently overloading `Invalidated`; only a future policy/ADR may
define recovery. `Completed`, `Blocked`, and `Abandoned` cannot reactivate in v1. Waiting retains its
cursor and may resume.

`nexa-events` owns privacy-minimal lifecycle and transition facts using domain identifiers and
semantic keys, avoiding a dependency on `nexa-lessons`. Event envelopes remain caller-owned.

## Consequences and unresolved decisions

- Lessons do not select pedagogy, score assessment, generate content, update mastery, orchestrate
  sessions, or execute tutor/media/avatar behavior.
- Required and certification labels are preserved, but skip/completion evidence semantics are
  deferred; sequential advance never skips an authored step.
- Rich conditions, priorities, freeform mode, branch and lesson-version migration, objective
  completion, invalidation, and blocked recovery remain unresolved.
- Concrete databases, networking, async runtimes, scheduling, and publication adapters are out of scope.
