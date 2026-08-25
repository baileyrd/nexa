# ADR-0064: Headless Speech-interaction cancellation binding

- Status: Accepted
- Date: 2026-08-25
- Scope: Fourth bounded Phase 5 application cancellation binding

## Context

ADR-0063 deliberately leaves Speech unbound and excludes Behavior-owned speech-dependent gestures from its four-surface evidence. The repository owner approved one application operation that may report its bounded result only after both complete ADR-0063 evidence and correlated existing Behavior cancellation evidence exist beneath the same workflow owner.

## Decision

`apps/nexa-headless` owns one exact Speech-interaction cancellation composition. Construction completely preflights an already-cancelled workflow and exact identities, a non-nil exact `SpeechId` and V1 request, exactly one supported cancellable participant for each canonical Speech surface, the existing Behavior cancellation capability and deterministic cancelled preview, and one canonical plan containing exactly Speech then Behavior with `RequestCancellation`.

Execution terminalizes before creating a single `WorkflowTaskGroup`. Exactly one Speech-target task and one Behavior-target task wait on their private target tokens. The Speech task invokes the fully preflighted ADR-0063 coordinator once; the Behavior task invokes the exact existing `AvatarPort` cancellation once. Exact-plan execution cancels and joins both. Success requires matching runtime execution, canonical complete Speech evidence, and exact cancelled Behavior request/report evidence. Identical success repeats return immutable evidence without dependency mutation; conflict, failure, or caller-future drop cannot restart work. All failures are closed and content-free.

The result proves only that the workflow-owned Speech control futures terminalized and the exact Behavior adapter cancellation completed and joined. It does not prove any provider, device, process, external request, audio implementation, renderer, or unrelated behavior stopped.

## Consequences and deferrals

Speech is bound only at this bounded application-control level. ADR-0062, ADR-0063, orchestrator/runtime semantics, and existing Behavior, Tutor Generation, and Retrieval bindings are unchanged. No TTS/STT, audio queue, playback, viseme, device, provider, networking, storage, telemetry, retry, timeout, recovery, authentication, authorization, secret, migration, or persistence capability is added.

Tool Execution and its security policy remain deferred. Phase 5 and the five-subsystem cancellation gate therefore remain incomplete, as does concrete external cancellation.
