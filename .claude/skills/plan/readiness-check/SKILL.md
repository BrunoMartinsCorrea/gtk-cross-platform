---
name: plan:readiness-check
version: 1.0.0
description: Verifies prerequisites before starting a feature — environment configured, ports defined, acceptance criteria drafted, baseline tests passing.
---

# plan:readiness-check

Invoke with `/plan:readiness-check` before writing the first line of implementation code.

## When to use

- At the very start of a session that begins a new feature
- When returning to an abandoned branch to verify it's still valid
- When the environment may have drifted (dependencies updated, schema changed)

## Checklist

### Environment

- [ ] `make build` passes (baseline compiles)
- [ ] `make test` passes (baseline tests green)
- [ ] `make lint` passes (no clippy warnings)
- [ ] Git working tree is clean (`git status` shows no uncommitted changes) OR changes are intentional

### Ports and interfaces

- [ ] If new port methods are needed: draft their signature in `src/ports/` before touching adapters
- [ ] If `IContainerDriver` is changing: `MockContainerDriver` is the first adapter to update
- [ ] All acceptance criteria are concrete and testable (from `/plan:scope-definition`)

### Dependencies

- [ ] No circular dependencies in the planned task order (from `/plan:dependency-mapping`)
- [ ] If new Cargo dependencies: `make audit` passes
- [ ] If new UI files: listed in `data/resources/resources.gresource.xml`

### i18n readiness

- [ ] If adding user-visible strings: source file will be added to `po/POTFILES`
- [ ] Text domain is `config::GETTEXT_PACKAGE` (bound in `main()`)

### Branch health

- [ ] Branch is based on `main` (or rebased if stale)
- [ ] No merge conflicts with `main`
- [ ] CI is green on `main` (check with `gh run list --branch main --limit 1`)

## Output

READY or BLOCKED.

If BLOCKED: list exactly which items failed and what action is needed before proceeding.
