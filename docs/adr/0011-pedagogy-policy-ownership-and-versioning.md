# ADR-0011: Pedagogy policy ownership, versioning, and explanations

- **Status:** Accepted
- **Date:** 2026-08-19
- **Related:** NEXA-PED-001, NEXA-STU-001, NEXA-LESSON-001, NEXA-ASMT-001, NEXA-EVT-001, ADR-0010

## Context

NEXA-PED-001 requires structured, explainable, replayable instructional decisions while leaving its
thresholds configurable and illustrating a broader formal strategy catalog than the narrow verbs used
throughout its instructional flows. It does not define a first production threshold set, retry limit,
fallback order, or exact action-to-strategy mapping. The reconstructed text remains unchanged.

## Decision

`nexa-pedagogy` owns validated decision inputs, the closed instructional-option vocabulary, rationale
codes, and pure policies. It has no mutable student repository: v1 consumes a validated
`nexa-student::MasteryState` through a shared reference and never creates evidence or updates mastery.
Lesson and assessment engines remain authoritative for content execution, sequencing, scoring,
permissions, and assessment rules.

`PedagogyPolicyV1` is version `1.0`. Its governed boundaries are minimum evidence `2`, low confidence
below `0.60`, mastery threshold `0.85`, repeated failure at `2`, and retry exhaustion at `3` attempts.
Comparisons are explicitly `<` or `>=`; changing any value, rule priority, fallback ordering, or
rationale meaning requires a new policy version. V1 accepts only the `BoundedWeightedV1` projection
version and reports both input-policy and projection-policy mismatches as structured errors.

Every successful decision carries at least one stable rationale code. Preferred actions are resolved
against caller-provided availability; unavailable values are never selected and fallback is stable.
`nexa-events` owns a privacy-minimal semantic decision event so it does not depend on pedagogy. The
core has no database, network, executor, clock, random source, LLM, lesson executor, or event publisher.

## Decision priority

1. mastered competency; 2. insufficient evidence; 3. low model confidence; 4. retry exhaustion;
5. repeated failure; 6. first failure; 7. partial success; 8. mastery-score gate awaiting status;
9. success; 10. review fallback. Higher rules suppress lower rules deterministically.

## Vocabulary reconciliation

The v1 option vocabulary (`introduce`, `explain`, `demonstrate`, `practice`, `hint`, `clarify`,
`reinforce`, `review`, `challenge`, `assess`, `retry`, `advance`) captures action/routing verbs present
or implied by registered NEXA-PED-001 instructional flows. It does **not** promote those verbs into the
specification's distinct formal `PedagogyStrategy` catalog, and it does not define lesson content.

## Unresolved decisions

- A normative mapping from routing options to the formal NEXA-PED-001 strategy catalog.
- Authored per-competency thresholds, transfer/retention requirements, retry semantics, and policy hashes.
- Whether attempt count is scoped to an item, step, competency, or session; callers must supply one
  internally consistent scope to v1.
- Assessment/curriculum constraint composition, misconception state, hint levels, difficulty hysteresis,
  review scheduling, durable decision repositories, and event-envelope construction.
