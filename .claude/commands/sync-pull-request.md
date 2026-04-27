---
description: Create or update a pull request for the current branch — detects whether a PR exists and acts accordingly, enforcing title/body/base contracts and reporting final state, review status, and CI checks.
---

# Sync Pull Request

> This command is self-contained — run it without prior conversation context.
>
> "Sync" means: **create** the PR if none exists for the current branch, or **update** it
> (title, body, labels, reviewers) if one already exists. Never force-push or rebase — only
> metadata is mutated on an existing PR.

---

## PR Lifecycle (reference model)

```
 draft ──► open ──► review ──► approved ──► merged
              │         │
              │    changes_requested
              │         │
              └─── (push fix) ◄──────────────┘
              │
           closed  (abandoned)
```

| Phase                 | Trigger                                      | What sync does                               |
|-----------------------|----------------------------------------------|----------------------------------------------|
| **pre-creation**      | Branch has commits, no PR                    | Creates PR (draft or open)                   |
| **open**              | PR exists, no reviews yet                    | Updates metadata if inputs changed           |
| **review**            | At least one review posted                   | Updates body/labels; never changes reviewers |
| **approved**          | All required reviews approved, checks green  | Reports ready-to-merge; no mutation          |
| **changes_requested** | At least one blocking review                 | Updates body with context; warns about state |
| **merged/closed**     | Terminal state                               | Exits with error — no action possible        |

---

## Step 0 — Baseline

Run these commands. Independent queries may run in parallel.

```bash
# 1. Current branch
git branch --show-current

# 2. Commits ahead of base (run in parallel with 3 and 4)
git log --oneline origin/main..HEAD 2>/dev/null || git log --oneline main..HEAD

# 3. Remote tracking + dirty tree (run in parallel with 2 and 4)
git status --short --branch

# 4. Existing PR for this branch (run in parallel with 2 and 3)
gh pr view --json number,title,body,state,url,isDraft,reviewDecision,statusCheckRollup,labels,reviews \
  2>/dev/null || echo "NO_PR_FOUND"

# 5. Merge conflicts with base (run after 1)
git diff --name-only --diff-filter=U 2>/dev/null | head -20

# 6. PR template presence (run independently)
[ -f .github/PULL_REQUEST_TEMPLATE.md ] && cat .github/PULL_REQUEST_TEMPLATE.md || echo "NO_TEMPLATE"
```

Record every output. Proceed to Pre-flight immediately after.

---

## Protected Files — Never Stage or Commit

**This command must never stage, commit, or otherwise include the following files in any git
operation, regardless of their working-tree state:**

```
.claude/          (entire directory — settings, commands, local overrides)
CLAUDE.md
```

These files are maintained exclusively by the user and must only be committed manually. Before
running any `git add`, `git push`, or related staging command, verify that no file matching
`.claude/**` or `CLAUDE.md` would be included. If such files appear in `git status --short`, skip
them silently — **do not abort, do not warn, do not suggest staging them**.

---

## Pre-flight — Fail Fast

Execute checks in this exact order. Stop at the first failure.

```
1. gh CLI available?        → command -v gh && gh auth status
2. branch ≠ main/master?    → if yes: E_NOT_A_FEATURE_BRANCH
3. ≥ 1 commit ahead?        → if no:  E_NO_COMMITS_AHEAD
4. merge conflicts?         → if yes: E_MERGE_CONFLICTS — list conflicting files
5. branch on remote?        → if no and tree clean: push with git push -u origin HEAD
                              if no and tree dirty: E_DIRTY_TREE_ABORTED (create path only)
6. existing PR terminal?    → if state = merged|closed: E_TERMINAL_STATE
```

For the **update path** (PR already exists): a dirty working tree is a **warning only** — metadata
updates do not require a clean tree.

---

## Derivation Rules

### Title derivation (in order of priority)

When no title is provided:

1. **Conventional Commit from branch name** — strip prefixes and convert to sentence:
   - `feat/add-runtime-switcher` → `feat: add runtime switcher`
   - `fix/icon-viewbox` → `fix: icon viewbox`
   - `chore/update-deps` → `chore: update deps`
2. **First commit subject on this branch** — `git log --format=%s origin/main..HEAD | tail -1`
3. **Branch name as-is** — replace hyphens/underscores with spaces, title-case

Apply title rules: ≤ 72 chars (truncate with `…`); strip trailing period; no confirmation needed
— report the derived title in OC-1.

### Body derivation

When no body is provided, build the body in this order:

```markdown
## Summary

<one or two sentences from commit messages; auto-detect "Closes #N" / "Fixes #N" / "Refs #N"
 in commits and append them as separate lines under the Summary>

## Checklist

<copy verbatim from .github/PULL_REQUEST_TEMPLATE.md if it exists;
 otherwise use the project default checklist below>

- [ ] `make fmt` passes
- [ ] `make lint` passes
- [ ] `make test` passes
- [ ] Commit messages follow Conventional Commits

🤖 Generated with [Claude Code](https://claude.ai/code)
```

Auto-detect issue references: `git log --format="%b" origin/main..HEAD | grep -Eo "(Closes|Fixes|Refs) #[0-9]+"`.

---

## Input Acceptance Criteria

### IC-1 — Branch state

| Check | Rule | Action on failure |
|-------|------|-------------------|
| Current branch ≠ `main`/`master` | PRs must come from feature branches | **ABORT** `E_NOT_A_FEATURE_BRANCH` |
| Branch has ≥ 1 commit ahead of base | Empty branches produce empty PRs | **ABORT** `E_NO_COMMITS_AHEAD` |
| No merge conflicts | Conflicted tree cannot produce a clean PR | **ABORT** `E_MERGE_CONFLICTS` — list files |
| Branch is pushed to remote | PR must reference a remote ref | Push with `git push -u origin HEAD` if clean; else **ABORT** (create path only) |
| Dirty working tree | Tree not committed | **ABORT** on create; **WARN** on update |

### IC-2 — Title

| Check | Rule | Action on failure |
|-------|------|-------------------|
| Non-empty | A blank title is rejected by GitHub | **ABORT** `E_EMPTY_TITLE` |
| ≤ 72 characters | Titles over 72 chars truncate in list views | Truncate automatically with `…` |
| Imperative mood | Conventional Commits convention | Warn only — do not block |
| No trailing period | GitHub convention | Strip automatically |

### IC-3 — Body

| Check | Rule | Action on failure |
|-------|------|-------------------|
| Non-empty | An empty body gives reviewers no context | **ABORT** `E_EMPTY_BODY` |
| Contains `## Summary` | Minimum structure for reviewers | Inject scaffold if missing |
| Contains `## Checklist` | Required by PR template when present | Inject empty checklist if missing |
| No unfilled placeholders (`<!--`, `[TODO]`, `[describe]`) | Left-over template text wastes reviewer time | Warn and list each placeholder — do not block |

### IC-4 — Base branch

| Check | Rule | Action on failure |
|-------|------|-------------------|
| Base branch exists on remote | Prevents dangling PRs | **ABORT** `E_BASE_NOT_FOUND` — list available branches |
| Base branch ≠ current branch | Cannot target itself | **ABORT** |
| Default: `main` | Use unless `--base` argument is given | Use `main` silently |

### IC-5 — Reviewers (optional)

| Check | Rule | Action on failure |
|-------|------|-------------------|
| Each handle is a valid GitHub username | `gh api users/<handle>` → 200 | Skip invalid handle with warning |
| Not adding reviewers to an already-reviewed PR | GitHub blocks reviewer mutation post-review | Warn and skip; do not fail |

### IC-6 — Labels (optional)

| Check | Rule | Action on failure |
|-------|------|-------------------|
| Each label exists in the repository | `gh label list` | Skip unknown label with warning |

---

## Execution — Create path

Run this path when Step 0 returned `NO_PR_FOUND`.

```bash
# 1. Push branch if not yet on remote
git push -u origin HEAD

# 2. Build and run gh pr create
#    --draft if: dirty tree confirmed, or title starts with "WIP:", or --draft flag given
#    --reviewer only if IC-5 passed
#    --label only if IC-6 passed
gh pr create \
  --title "<DERIVED_TITLE>" \
  --body "$(cat <<'EOF'
<DERIVED_BODY>
EOF
)" \
  --base main \
  [--draft] \
  [--reviewer handle1,handle2] \
  [--label label1,label2]

# 3. Fetch created PR metadata
gh pr view --json number,title,state,url,isDraft,reviewDecision,statusCheckRollup
```

---

## Execution — Update path

Run this path when Step 0 returned an existing PR.

Before calling `gh pr edit`, diff the current PR fields against derived fields:

```bash
# Fetch current values (already done in Step 0)
CURRENT_TITLE=$(gh pr view --json title -q .title)
CURRENT_BODY=$(gh pr view --json body -q .body)

# Compare and build minimal edit arguments
CHANGED_FIELDS=""
[ "$CURRENT_TITLE" != "$DERIVED_TITLE" ] && CHANGED_FIELDS="$CHANGED_FIELDS title"
[ "$CURRENT_BODY"  != "$DERIVED_BODY"  ] && CHANGED_FIELDS="$CHANGED_FIELDS body"
```

Only call `gh pr edit` if at least one field changed. Report exactly which fields changed in OC-2.

```bash
# Update only changed fields
gh pr edit \
  [--title "<DERIVED_TITLE>"]   # only if title changed
  [--body "$(cat <<'EOF'
<DERIVED_BODY>
EOF
)"]                              # only if body changed
  [--add-label label1] \
  [--remove-label old_label]

# Convert draft → ready if IC-2/IC-3 now fully satisfied and --ready flag given
gh pr ready   # only if --ready was explicitly requested

# Re-fetch metadata for output
gh pr view --json number,title,state,url,isDraft,reviewDecision,statusCheckRollup
```

> **Never** call `gh pr edit --reviewer` on a PR that has already received at least one review —
> GitHub blocks it. Check `reviews` from Step 0 first.

---

## Output Acceptance Criteria

The command is complete only when all of the following are reported.

### OC-1 — Identity

```
PR #<number>: <title>
URL: <url>
```

If the title was derived (not provided), append `(derived from: <source>)` to indicate the source
(branch name, commit subject, or branch prefix).

### OC-2 — Action taken

One of:

```
ACTION: created (draft)
ACTION: created (open)
ACTION: updated (title)
ACTION: updated (body)
ACTION: updated (title + body)
ACTION: updated (labels)
ACTION: updated (labels + body)
ACTION: no-op (PR unchanged)
```

Report the minimum set of changed fields. Never silently skip — always print `no-op` if nothing changed.

### OC-3 — Current state

```
STATE: open | draft | merged | closed
REVIEW: none | approved | changes_requested | review_required
```

If state is `merged` or `closed`, print `E_TERMINAL_STATE` and exit non-zero.

### OC-4 — CI checks

```
CHECKS: <n> passing / <m> failing / <k> pending
```

Derive from `statusCheckRollup` in the `gh pr view` JSON response. If no checks have run yet,
print `CHECKS: not yet started`.

If any check is **failing**, list each by name:

```
  ✗ ci / build (conclusion: failure)
  ✗ ci / lint  (conclusion: failure)
```

### OC-5 — Next action

Based on the combination of STATE + REVIEW + CHECKS, print exactly one next-action line:

| Condition | Next action |
|-----------|-------------|
| `draft`, checks green | `NEXT: mark ready — gh pr ready` |
| `open`, review `none`, checks green | `NEXT: request review` |
| `open`, review `changes_requested` | `NEXT: address review comments, then re-push` |
| `open`, review `approved`, checks green | `NEXT: merge — gh pr merge --squash` |
| `open`, checks failing | `NEXT: fix failing checks (listed above)` |
| `open`, checks pending | `NEXT: wait for CI to complete` |
| `merged` or `closed` | `NEXT: none — PR is in a terminal state` |

---

## Error taxonomy

| Code | Meaning | Recovery |
|------|---------|----------|
| `E_GH_CLI_MISSING` | `gh` not installed or not authenticated | Run `gh auth login` |
| `E_NOT_A_FEATURE_BRANCH` | Current branch is `main`/`master` | Checkout a feature branch |
| `E_NO_COMMITS_AHEAD` | Branch has no commits ahead of base | Commit changes first |
| `E_MERGE_CONFLICTS` | Branch has unresolved merge conflicts | Resolve conflicts, then re-run |
| `E_DIRTY_TREE_ABORTED` | Uncommitted changes on create path | Commit or stash changes |
| `E_EMPTY_TITLE` | Title could not be derived and was not provided | Provide `--title` |
| `E_EMPTY_BODY` | Body could not be derived and was not provided | Provide `--body` |
| `E_BASE_NOT_FOUND` | Base branch does not exist on remote | Check base branch name |
| `E_TERMINAL_STATE` | PR is merged or closed | No action possible |

---

## Exit criteria

The command exits **0** (success) when:

- All pre-flight checks pass
- All input criteria are satisfied
- The PR was created or updated (or confirmed no-op)
- All five output criteria (OC-1 through OC-5) are printed

The command exits **non-zero** on any `E_*` error code or when `gh` returns a non-zero status.
