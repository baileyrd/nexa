# Issue 19 authority conflict

**Status:** Blocked before R3 contract work
**Scope:** Issue 19 request to define the Rusty Data OS EXP-0001 R3 lifecycle contract
**Authoritative base inspected:** `origin/main` at `6dbe84d88d0f5aa10d64f7b359dba019937b848e`
**Evidence classification:** Repository-state and workflow conflict record only; not a design decision, implementation, test result, or benchmark evidence

## Requested increment

Issue 19 requests a documentation-only Rusty Data OS record resolving EXP-0001
blockers BLK-004, BLK-005, BLK-011, and BLK-012 while constraining BLK-007.
The request explicitly requires a task branch from the latest `main`, treats that
branch as authoritative, and prohibits silently inventing authority when a
consequential conflict prevents a defensible R3 decision.

## Exact conflict

The latest `main` is the **Nexa** repository, not the Rusty Data OS repository
described by issue 19:

- root `AGENTS.md` identifies itself as the "Nexa agent guide," routes design
  work through `docs/BASELINE.md` and `docs/SPECIFICATION-REGISTRY.md`, and
  requires preservation of Nexa's accepted ADR and specification authority;
- `docs/PROJECT-STATUS.md` identifies the project as Nexa and records Phase 5
  work through ADR-0065;
- the requested Rusty Data OS authorities `docs/VISION.md`,
  `docs/PRINCIPLES.md`, `docs/ARCHITECTURE.md`, and
  `docs/RESEARCH-ROADMAP.md` do not exist on the authoritative branch;
- the requested EXP-0000 semantic contracts, EXP-0001 experiment definition,
  R1/R2 readiness records, requirements, glossary, unknowns, research
  questions, and traceability registry do not exist on the authoritative
  branch; and
- the remote configured for this checkout is `https://github.com/baileyrd/nexa.git`.

Consequently there is no authoritative EXP-0001 semantic envelope, blocker
registry, R1/R2 record, or traceability/status set on latest `main` against which
an R3 identity, time, gap, and retry lifecycle can be checked or synchronized.
Adding the issue-provided Rusty Data OS document set to Nexa, or reconstructing
its missing decisions from non-`main` working-tree history, would silently
invent or import authority and would violate both the issue's conflict rule and
Nexa's preservation rule.

## Decision and bounded outcome

R3 is not authored, and BLK-004, BLK-005, BLK-011, BLK-012, and BLK-007 receive
no status change. No Nexa authority, status, specification, ADR, traceability
record, code, Cargo file, executable test, benchmark harness, or later R4-R9
work is changed.

Work can resume only after a repository owner supplies a latest `main` that
contains the governed Rusty Data OS authority chain, or explicitly authorizes a
reviewed repository migration/import that establishes that chain. At that point
issue 19 should be restarted from the corrected latest `main`; this conflict
record must not be treated as an R3 contract or completion evidence.
