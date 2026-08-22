# ChatGPT–Codex Development Workflow

## Purpose

This document defines the human-coordinated workflow used to develop Nexa with ChatGPT and Codex.

- **ChatGPT** acts as the repository-aware planner, initial-instruction author, pull-request reviewer, correction-comment author, and merge gatekeeper. It posts implementation corrections as top-level `@codex` comments on the existing pull request.
- **Codex** implements the bounded initial increment and, when invoked by a correction comment, commits and pushes corrections to that pull request's existing branch.
- **The user** transfers only the initial new-increment instruction from ChatGPT to Codex, opens or updates pull requests, and provides the workflow trigger messages. The user does not transfer pull-request corrections into a separate Codex task.

This document governs the collaboration process. It does not supersede `AGENTS.md`, accepted ADRs, approved specifications, the specification registry, or other repository authorities.

## Optional autonomous operating mode

The human-coordinated workflow below remains the default. As an optional mode,
ChatGPT automations may act as planner, reviewer, and merge gate while Codex
Cloud performs implementation through the repository's GitHub connection. The
operating procedure and prerequisites are defined in
[`docs/automation/CODEX-CLOUD-AUTOMATION.md`](docs/automation/CODEX-CLOUD-AUTOMATION.md).

In this mode:

1. ChatGPT selects one bounded increment and dispatches it from one GitHub issue
   with a top-level `@codex` comment.
2. Codex Cloud implements and validates the increment, then creates exactly one
   pull request. Every correction is made on that pull request's existing
   branch in response to a top-level `@codex` pull-request comment.
3. GitHub pull-request events trigger immediate ChatGPT review. When exact-head
   CI is pending and no correction is required, the event-driven pull-request
   run stops cleanly. Re-review occurs on a later matching pull-request event or
   an explicit repository-owner command.
4. At most one implementation issue or pull request may be active. Never
   dispatch competing implementation work while that issue or pull request is
   active.
5. ChatGPT never merges when required CI is missing, pending, failed, stale, or
   associated with an older head commit.

These automation rules do not change the authority, validation, correction,
or exact-head merge safeguards elsewhere in this document.

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
- the complete actual diff and surrounding affected source;
- the approved task scope and exclusions;
- applicable specifications, ADRs, traceability, and architecture boundaries;
- implementation and test correctness;
- required documentation changes;
- GitHub Actions, with every required job and step associated with the exact reviewed head SHA;
- reviews, comments, and unresolved review threads;
- regressions, omissions, or unrelated changes.

If the exact reviewed head is correct, unchanged, and all required checks are green, ChatGPT may merge it.

If corrections are required, ChatGPT must prepare one complete, consolidated correction instruction and post it as a top-level, non-review comment on the existing pull request. It must begin with an implementation instruction such as:

> `@codex Fix the merge-blocking findings below on this PR’s existing branch.`

Do not use `@codex review` to request implementation. The correction comment must include:

- the exact reviewed head SHA;
- every finding and affected location;
- the evidence and violated invariant, where applicable;
- every required outcome;
- scope constraints and exclusions;
- required tests;
- exact validation commands;
- an explicit instruction to commit and push to the existing pull-request branch; and
- an explicit prohibition against creating another branch or pull request.

If connected GitHub access cannot post the comment, ChatGPT must give the user the exact, complete, paste-ready text, labeled **GitHub PR comment**, to paste into that existing pull request's comment box. It must not label or describe the fallback as a prompt for a separate Codex task or discussion.

### 5. Codex corrects the existing pull-request branch

The top-level `@codex` correction comment invokes Codex on the existing pull request. Corrections must not be routed through a separate Codex task.

Codex must:

1. Apply the corrections to the existing pull-request branch.
2. Avoid unrelated changes.
3. Rerun the applicable validation.
4. Review the resulting diff.
5. Commit and push the corrections to the existing pull-request branch.
6. Do not create another branch or pull request.
7. Report the new commit and validation results.

If Codex cannot push to the existing branch, verify branch ownership, permissions, head movement, and whether maintainers may modify it. Do not automatically create or recommend a replacement pull request. A replacement branch or pull request is permitted only after the existing branch is confirmed unusable and the user explicitly authorizes replacement.

While a correction task is expected to update the branch, avoid manual rebases, force pushes, branch renames, and GitHub **Update branch** operations. Never discard user changes to restore branch ownership.

The user then tells ChatGPT:

`branch updated`

### 6. ChatGPT re-reviews the updated branch

When the user says `branch updated`, ChatGPT must:

1. Re-fetch the pull request and record its new exact head.
2. Confirm that every requested correction was addressed.
3. Review the complete actual diff and surrounding affected source, including all newly changed and affected code.
4. Inspect reviews, comments, and unresolved review threads.
5. Check for regressions or scope expansion.
6. Associate CI with the new exact head and inspect every required job and step.
7. Immediately before merging, re-fetch the pull request and confirm that the head remains the exact reviewed commit, all required checks for that head are green, and no blocking feedback remains.
8. Use expected-head protection when the GitHub merge operation supports it.

If problems remain, ChatGPT posts another single consolidated top-level `@codex` correction comment on the same pull request, and the inner loop repeats. The user supplies `branch updated` after Codex updates the existing branch.

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
| `branch updated` | Re-fetch and re-review the corrected pull request's new exact head, complete affected surface, unresolved feedback, and head-specific CI; then merge or post one consolidated correction comment. |
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

Immediately before merging, ChatGPT must re-fetch the pull request and verify that its head is unchanged from the exact reviewed SHA, every required check and step is green for that SHA, and no blocking feedback remains. ChatGPT must never merge based on successful CI from an older SHA and must use expected-head protection when the merge operation supports it.

## Responsibility boundaries

ChatGPT must not:

- bypass Codex and implement the next increment when the established workflow calls for a Codex instruction prompt;
- treat prior conversation history as more authoritative than the repository;
- invent project status, requirements, or completed capabilities;
- silently resolve conflicts between implementation and governing documents;
- route corrections to an open pull request through a separate Codex task;
- automatically create or recommend a replacement branch or pull request for correction work;
- merge a commit different from the exact commit it reviewed.

Codex must not:

- expand the approved task without explicit authorization;
- silently resolve conflicting requirements;
- create another branch or pull request for correction work unless the existing branch is confirmed unusable and the user explicitly authorizes replacement;
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
