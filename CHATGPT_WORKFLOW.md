# ChatGPT–Codex Development Workflow

## Purpose

This document defines the human-coordinated workflow used to develop Nexa with ChatGPT and Codex.

- **ChatGPT** acts as the repository-aware planner, instruction author, pull-request reviewer, correction author, and merge gatekeeper.
- **Codex** implements the bounded task described in the instruction prompt.
- **The user** transfers instructions between ChatGPT and Codex, opens or updates pull requests, and provides the workflow trigger messages.

This document governs the collaboration process. It does not supersede `AGENTS.md`, accepted ADRs, approved specifications, the specification registry, or other repository authorities.

## Source of truth

Every workflow cycle begins from the repository’s current state.

ChatGPT and Codex must:

1. Treat the latest `main` branch as authoritative.
2. Read and follow [`AGENTS.md`](AGENTS.md).
3. Read [`docs/PROJECT-STATUS.md`](docs/PROJECT-STATUS.md).
4. Follow the authority and document-routing instructions contained in those files.
5. Inspect relevant changes merged after any recorded checkpoint.
6. Report conflicts or stale documentation rather than silently resolving them.
7. Avoid relying on prior conversation history when repository evidence is available.

## Outer implementation loop

### 1. The user says `next`

ChatGPT must:

1. Inspect the latest `main`.
2. Read `AGENTS.md`, `docs/PROJECT-STATUS.md`, and the applicable authorities routed to by them.
3. Check the current roadmap, specifications, ADRs, traceability evidence, implementation state, and open work.
4. Select the next smallest evidence-backed, independently reviewable increment.
5. Produce one complete, task-specific instruction prompt for Codex.
6. State the objective, governing authorities, precise scope, exclusions, affected boundaries, acceptance criteria, documentation impact, risks, and required validation.
7. Avoid implementing the change itself.
8. Avoid combining unrelated, speculative, or separately deferred capabilities.

### 2. The user gives the instruction prompt to Codex

Codex must:

1. Start from the latest repository state.
2. Read and follow `AGENTS.md`.
3. Read `docs/PROJECT-STATUS.md` and the applicable authorities.
4. Implement only the bounded increment in the instruction prompt.
5. Add or update the required tests, documentation, traceability, specifications, or ADRs when the approved scope requires them.
6. Run every applicable validation command required by `AGENTS.md`.
7. Review its own diff for correctness, scope, regressions, and unintended changes.
8. Commit the completed work to its task branch.
9. Report the resulting commit, changed files, validation results, and unresolved issues.

### 3. The user opens a pull request

After Codex finishes, the user opens the pull request and tells ChatGPT:

`PR created`

## Inner review and correction loop

### 4. ChatGPT reviews the pull request

When the user says `PR created`, ChatGPT must inspect the actual open pull request rather than relying only on Codex’s summary.

The review must include:

- pull-request base and head branches;
- the exact head commit;
- the complete diff;
- the approved task scope and exclusions;
- applicable specifications, ADRs, traceability, and architecture boundaries;
- implementation and test correctness;
- required documentation changes;
- GitHub Actions and all required checks;
- unresolved review comments;
- regressions, omissions, or unrelated changes.

If the exact reviewed head is correct and all required checks are green, ChatGPT may merge it.

If corrections are required, ChatGPT must provide a complete correction prompt for Codex. The correction prompt must identify:

- each finding;
- the required outcome;
- relevant constraints and authorities;
- prohibited scope expansion;
- tests and validation that must be rerun.

### 5. Codex corrects the existing pull-request branch

The user gives the correction prompt to Codex.

Codex must:

1. Apply the corrections to the existing pull-request branch.
2. Avoid unrelated changes.
3. Rerun the applicable validation.
4. Review the resulting diff.
5. Commit the corrections.
6. Report the new commit and validation results.

The user then tells ChatGPT:

`branch updated`

### 6. ChatGPT re-reviews the updated branch

When the user says `branch updated`, ChatGPT must:

1. Fetch the pull request’s new exact head.
2. Confirm that every requested correction was addressed.
3. Inspect all newly changed and affected code.
4. Check for regressions or scope expansion.
5. Verify required CI against the new exact head.
6. Merge only when the exact reviewed commit is correct and green.

If problems remain, ChatGPT provides another correction prompt and the inner loop repeats.

## Completion and continuation

After merging, ChatGPT must:

1. Report the pull-request number and merge result.
2. Identify any intentionally deferred work without automatically beginning it.
3. Wait for the user’s next workflow trigger.

The user says:

`next`

This starts a new outer implementation loop based on the newly updated `main`.

## Trigger phrases

| User message | Required ChatGPT action |
| --- | --- |
| `next` | Inspect current `main` and produce the next bounded Codex instruction prompt. |
| `PR created` | Locate and review the new pull request’s exact head, complete diff, approved scope, and CI. |
| `branch updated` | Re-review the corrected pull-request head and either merge it or provide further corrections. |
| `Is it green yet?` | Check the current pull request’s latest CI state without assuming previous results remain valid. |

Trigger phrases are case-insensitive. Minor punctuation or formatting differences do not change their meaning.

## Review and merge safeguards

ChatGPT must not merge when:

- required checks are failing, pending, missing, or attached to an older commit;
- the pull-request head changed after the reviewed commit;
- requested corrections remain unresolved;
- the pull request contains unexplained scope expansion;
- repository authorities conflict with the implementation;
- the implementation contradicts the approved instruction;
- required documentation or traceability is missing;
- the pull request cannot be verified using available repository evidence.

If the pull-request head changes after approval or review, ChatGPT must review the new exact head before merging.

A previously green check does not establish that a newer commit is green.

## Responsibility boundaries

ChatGPT must not:

- bypass Codex and implement the next increment when the established workflow calls for a Codex instruction prompt;
- treat prior conversation history as more authoritative than the repository;
- invent project status, requirements, or completed capabilities;
- silently resolve conflicts between implementation and governing documents;
- merge a commit different from the exact commit it reviewed.

Codex must not:

- expand the approved task without explicit authorization;
- silently resolve conflicting requirements;
- treat reconstructed or draft material as approved authority unless the repository governance documents say so;
- declare completion without running and reporting applicable validation.

The user remains the human coordinator and may stop, revise, or redirect the workflow at any point.

## Starting a fresh ChatGPT conversation

Use the following bootstrap prompt in a new ChatGPT conversation:

> Continue development of the GitHub repository `baileyrd/nexa`.
>
> Read and follow `/CHATGPT_WORKFLOW.md`, `/AGENTS.md`, and `/docs/PROJECT-STATUS.md` from the current `main` branch.
>
> Follow the document-routing and authority rules contained in those files. Treat the repository’s current state as authoritative rather than relying on previous conversation history.
>
> Adopt the ChatGPT role defined in `CHATGPT_WORKFLOW.md`.
>
> Inspect the current repository and pull-request state, briefly report where the workflow currently stands, and wait for the appropriate workflow trigger unless I have already provided one.
