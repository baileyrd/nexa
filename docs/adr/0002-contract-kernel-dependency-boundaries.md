# ADR-0002: Contract-kernel dependency boundaries

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

NEXA-DOM-001, NEXA-EVT-001, and NEXA-NBP-001 overlap in their illustrative envelopes. A cycle would make the contracts impossible to reuse independently and would leak transport or renderer concerns into domain types.

## Decision

`nexa-domain` is the dependency-light leaf and owns shared scalar/newtype representations. `nexa-events` and `nexa-nbp` may each depend on it, but not on one another. Neither contract crate may depend on the root runtime, an async runtime, GUI/GPU, storage, provider, or networking package. Event-to-NBP conversion belongs in a future composition/adaptor crate above both leaves.

The root `nexa-3d-runtime` remains unchanged. Public contract features may add implementation support but must not change wire representation.

## Consequences

The DAG is mechanically checkable through Cargo metadata. Some apparently shared concepts remain intentionally distinct: event envelope versions and NBP protocol versions use the same domain representation, but have separate semantic ownership.
