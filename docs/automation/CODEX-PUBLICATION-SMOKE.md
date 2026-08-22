# Codex publication smoke test

## Purpose

The `Codex publication smoke test` workflow is a narrow, supervised check of the
repository's publication path. It asks `openai/codex-action@v1` to create one
non-sensitive evidence file, then authenticates Git only after Codex exits to
commit that file, push a unique branch, and open one draft pull request.

This workflow is disposable operational evidence. It is not a product build,
release process, or general-purpose Codex automation.

## Run the smoke test

1. Confirm that the repository Actions secret `OPENAI_API_KEY` is configured.
   Never paste the key into a workflow input, prompt, log, file, commit, or pull
   request.
2. In the `baileyrd/nexa` repository on GitHub, open **Actions** and select
   **Codex publication smoke test**.
3. Choose **Run workflow** on `main`. The workflow accepts no custom prompt or
   other workflow input and permits only repository owner `baileyrd` to run its
   publication job.
4. Observe every workflow step. Any missing or unexpected file, failed commit,
   failed push, or missing/non-draft pull request causes the run to fail.

For workflow run `<run-id>`, the expected branch is
`automation/codex-publication-smoke-<run-id>`. The branch contains exactly one
new evidence file, `docs/automation/smoke-<run-id>.md`, and the workflow opens
one draft pull request targeting `main` with the fixed title
`chore: disposable Codex publication smoke evidence`.

## Meaning of success

A successful run proves, for that run and repository configuration, that:

- `openai/codex-action@v1` can authenticate using the configured
  `OPENAI_API_KEY` secret and make the one authorized workspace change;
- the workflow rejects additional or substituted repository changes and checks
  the expected file's exact non-sensitive content and whitespace;
- the GitHub Actions commit identity can create a commit and the authenticated
  publication step can push the unique branch without force-pushing; and
- the workflow token can create exactly one draft pull request targeting
  `main`.

Success does not establish a release pipeline, unattended issue-to-PR
automation, arbitrary-prompt execution, automatic review or merge, retry or
cleanup automation, or permission for Codex to change any other file. It also
does not merge or approve the generated pull request.

## Cleanup

After recording the successful run and inspecting the evidence, close the draft
pull request without merging it. Delete its
`automation/codex-publication-smoke-<run-id>` branch from the pull request page
or repository branch list. The evidence file then remains only in the closed
pull request's commit history and is not added to `main`.

## Security boundaries

The workflow is manual-only, has no prompt input, and fails closed unless both
the repository and triggering actor are the expected owner. Its job receives
only `contents: write` and `pull-requests: write`; Codex uses the workspace-only
permission profile as a dedicated ephemeral, unprivileged user. The checkout
does not persist Git credentials, `.git` remains inaccessible to that user, and
the user's only writable repository path is the pre-created run-specific
evidence file. The GitHub token is provided only to the later push and pull
request steps, and Git authentication is configured only after Codex exits. The
fixed prompt authorizes exactly that evidence file, and a post-action gate
rejects every other changed or untracked path before commit. Concurrency
prevents two attempts for the same workflow run from publishing at the same
time.

The API key must remain solely in the GitHub Actions secret store. Do not echo,
inspect, interpolate into content, or otherwise expose the key—or any other
secret—in logs, prompts, files, commits, branch names, pull request titles, or
pull request bodies. The generated evidence is deliberately limited to the
public repository name, workflow run ID, and a success statement.
