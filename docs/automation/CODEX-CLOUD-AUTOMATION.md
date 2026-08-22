# Codex Cloud automation

## Purpose and prerequisites

This optional operating mode automates the existing ChatGPT–Codex workflow; it
does not replace the human-coordinated mode in
[`CHATGPT_WORKFLOW.md`](../../CHATGPT_WORKFLOW.md). ChatGPT automations remain
the planner, reviewer, and merge gate. Codex Cloud performs only the bounded
implementation that ChatGPT dispatches.

Before enabling the mode, the repository owner must connect the GitHub
repository to Codex Cloud and grant it permission to write a task branch and
open or update a pull request. The route consumes the user's ChatGPT/Codex
allowance. It requires neither an OpenAI API credential nor the API-backed
Codex GitHub Action.

## Issue-to-pull-request flow

1. Confirm that no other automation implementation issue or pull request is
   active. Never dispatch competing implementation work while an implementation
   issue or pull request is active.
2. Create one bounded GitHub issue containing the complete implementation
   contract, repository instructions, scope, exclusions, acceptance criteria,
   and required validation.
3. Dispatch the work with one top-level `@codex` issue comment. Codex Cloud
   starts from the latest `main`, implements the contract on one task branch,
   runs the required validation, commits, and opens exactly one pull request
   targeting `main`.
4. Treat that issue and pull request as the sole active implementation work
   until the pull request is merged or the owner closes the work.

Codex must report its branch, commit, changed files, validation results, and
blockers. It must not create a second branch or pull request for the task.

## Review and correction flow

GitHub pull-request events trigger immediate ChatGPT review of the actual diff,
feedback, and CI for the exact pull-request head. A correction request is one
consolidated, top-level `@codex` comment on the existing pull request. It names
the reviewed head, required outcomes, scope constraints, and validation. Codex
commits and pushes every correction to the same pull-request branch; the
updated head receives a fresh review.

When exact-head CI is pending and no correction is required, the event-driven
pull-request run stops cleanly. Re-review occurs on a later matching
pull-request event or an explicit repository-owner command. No older run's
result may substitute for the exact-head checks.

## Merge guards and owner blockers

ChatGPT may merge only after re-fetching the pull request and proving that the
head is unchanged from the reviewed commit, all required checks for that exact
head passed, no blocking feedback remains, and the diff satisfies the issue.
Missing, pending, failed, stale, or older-head CI always blocks a merge.

Automation must also enforce the single-active-work limit: only one
implementation issue or pull request may be active, and it must never dispatch
competing implementation work while that issue or pull request remains active.

Ambiguous authority, conflicting requirements, unavailable permissions,
unexpected head movement, unresolved feedback, scope expansion, and a
single-active-work violation are owner blockers. Stop and ask the repository
owner instead of weakening a guard, silently resolving a conflict, or starting
parallel work.

## Publication failure and recovery

If Codex cannot push or open the pull request, record the error and verify the
repository connection, branch ownership, write and pull-request permissions,
head movement, and whether maintainers may modify the branch. Retry publication
on the same task branch only after the owner corrects the blocker.

Do not automatically create a replacement branch or pull request. Replacement
is allowed only when the existing branch is confirmed unusable and the
repository owner explicitly authorizes it. Until recovery succeeds or the
owner closes the work, the issue remains the single active implementation and
no merge is counted.
