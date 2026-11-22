---
name: publish
description: Build and verify pbcp locally, create a concise PR, and auto-merge after required CI succeeds. Use only when the user explicitly asks to publish, ship, or auto-merge repository changes.
---

# Publish

Ship the current, user-authorized change in `github.com/adeshmukh/pbcp`. Invocation
authorizes committing, pushing, creating a PR, and enabling its auto-merge, not
publishing unrelated work or bypassing checks.

## Prepare

1. Read `AGENTS.md`, `docs/project/IMPLEMENTATION_PLAN.md`, and
   `docs/project/PROGRESS.md`. Follow the progress protocol and inspect
   `git status --short`, the staged/unstaged diff, and `git worktree list`.
2. Verify repository identity and `gh auth status --hostname github.com`.
   Pin GitHub commands to `--repo github.com/adeshmukh/pbcp` or
   `gh api --hostname github.com repos/adeshmukh/pbcp/...`.
3. Work on a feature branch in an exclusively owned worktree, never commit to
   `main`. If starting on `main`, create a fresh feature branch only when that
   worktree is yours; otherwise use an isolated worktree and preserve the original
   changes. Stop for ambiguous ownership or change scope. Never stash, discard,
   force-push, or include somebody else's work.
4. Fetch the current remote `main` through a verified repository URL. Reconcile
   base changes in the owned feature branch without rewriting others' commits.
   Validate branch names with `git check-ref-format --branch` and quote dynamic
   shell arguments. If the existing branch PR is merged, create a fresh successor
   from current `main` rather than reusing its old history.
5. Inspect the live repository's `allow_auto_merge` and `main` protection.
   Require auto-merge enabled, strict required CI checks, and administrator
   enforcement. The required check identities are in `main-protection.json`;
   every listed GitHub Actions check must still be required. Missing gates are a
   blocker, not permission to merge immediately. Do not silently change settings.

## Verify locally

Run from the owned worktree, using the same stable Rust toolchain as CI:

```sh
cargo +stable fmt --all -- --check
cargo +stable test --locked --all-targets
cargo +stable clippy --locked --all-targets -- -D warnings
cargo +stable tree --locked --edges normal
python3 -m unittest discover -s .github/scripts -p 'test_*.py'
bash -n .github/scripts/publish.sh
cargo +stable build --locked --release
git diff --check
```

Also enforce CI's dependency contract:

```sh
cargo +stable metadata --locked --format-version 1 | python3 -c \
  'import json,sys; m=json.load(sys.stdin); assert len(m["packages"]) == 1 and not m["packages"][0]["dependencies"], "unexpected Cargo dependency"'
```

If workflows changed, run `actionlint` too; resolve missing tooling explicitly.
Fix only failures caused by or tightly coupled to the change, then rerun affected
checks. Do not ship with failed or incomplete verification. A local native build
does not validate the other architecture or real clipboard delivery.

Update `PROGRESS.md` with completed implementation, actual local validation,
pending hosted verification, and one independently actionable next task before
committing. Do not call an unmerged PR or unpublished release completed. Record
the eventual remote outcome on the PR, linking its checks and release as
validation evidence.

## Create the PR

Stage explicit, reviewed paths only. Inspect the staged diff, check it for
accidental secrets/unrelated changes, and run `git diff --cached --check`.
Commit with a meaningful subject and the repository's required
`Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>` trailer,
unless the user requested otherwise.

Push only the validated feature branch. If SSH authentication fails, use
authenticated HTTPS without changing shared remote settings:

```sh
git -c credential.helper='!gh auth git-credential' push \
  https://github.com/adeshmukh/pbcp.git HEAD:refs/heads/<branch>
```

Find an existing open PR for that head branch and reuse it; otherwise create
one against `main` with `gh pr create`. Confirm its base repository/ref and exact
head SHA. Use a short imperative title and a body of two or three sentences:
what changed, why, and actual validation. Add a limitation only if material.
Avoid file inventories, long templates, and fabricated test results.

Example:

> Add CI-gated publishing so verified changes ship without waiting for review.
>
> Validation: local tests, Clippy, formatting, and release build passed.

## Auto-merge and verify

Resolve native stack membership before landing. This skill handles independent
PRs only; for an existing stack, use `pr-stack` with explicit authorization for
the dependency boundary.

Re-read PR state, base, head SHA, draft status, and live merge gates immediately
before enabling auto-merge. Stop if the head changed unexpectedly or the PR is
draft/closed. Use the allowed merge-commit method, pinned to the checked head:

```sh
gh pr merge <number> --repo github.com/adeshmukh/pbcp \
  --auto --merge --match-head-commit <verified-head-sha>
```

Never use `--admin`, disable required checks, or force a merge. Do not delete
branches/worktrees that another session may own.

Watch the PR checks with `gh pr checks --watch --fail-fast`. On failure, report
the failing check and leave the PR unmerged. If unrelated base updates make
strict CI stale, merge current `main` into the exclusively owned feature branch,
reverify, push, and let CI run again; never bypass the freshness requirement.
Confirm `state=MERGED` and the merge commit through `gh pr view` before claiming
success. Enabling auto-merge or entering a merge queue is not a completed merge.

Once merged, locate the main-push workflow by that merge commit, follow it to
completion, and verify a non-draft GitHub Release with both archives and
`SHA256SUMS`. Report a release URL only after publication. If release CI fails,
say the PR merged but publication failed; do not undo the merge or fake success.
Keep the final response to the PR/release links and any blocker.

## One-time GitHub configuration

The maintainer must explicitly authorize settings changes. Inspect existing
protection and rulesets first; preserve stronger safeguards and stop on a
conflict. `main-protection.json` is the initial configuration, not a payload to
blindly reapply on every invocation:

```sh
gh api --hostname github.com --method PATCH repos/adeshmukh/pbcp \
  -F allow_auto_merge=true
gh api --hostname github.com --method PUT \
  repos/adeshmukh/pbcp/branches/main/protection \
  --input .github/skills/publish/main-protection.json
```

This requires all six pre-publication CI jobs from the GitHub Actions app,
up-to-date branches, and the same checks for admins. There are no required review
approvals, force pushes, or branch deletions. Read back settings to verify them.
If job names change, update both the payload and live protection in the same
authorized rollout; otherwise auto-merge can wait for a nonexistent check.

`.github/scripts/publish.sh` is the CI release-asset publisher, not this PR
workflow. Do not run it locally to publish changes.
