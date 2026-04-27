# Governance

This document describes how the project is run. It applies to all contributors and maintainers.

## Roles

| Role                | Description                                     |
|---------------------|-------------------------------------------------|
| **Contributor**     | Anyone who opens an issue or pull request       |
| **Maintainer**      | Has merge rights; reviews PRs and cuts releases |
| **Lead maintainer** | Breaks ties; owns the release process           |

## Decisions

- Day-to-day decisions (bug fixes, dependency bumps) are made by any maintainer.
- Significant changes (architecture, new dependencies, breaking API) require consensus among all active maintainers.
- If consensus cannot be reached, the lead maintainer has the deciding vote.
- Anyone can propose changes by opening an issue with the `proposal` label.

## Becoming a maintainer

Maintainership is offered by invitation or by request after meeting **one** of these criteria:

- 3 or more pull requests merged within any 6-month window, **or**
- A single high-impact contribution (new runtime driver, major feature, significant bug fix) accepted by the lead
  maintainer.

Maintainers who are inactive for 6 months may be moved to emeritus status.

## Releases

- Versions follow [Semantic Versioning](https://semver.org/).
- Nightly builds are published automatically to GitHub Releases on every push to `main` (see
  `.github/workflows/release.yml`).
- Stable releases are tagged `vX.Y.Z` by the lead maintainer after updating `CHANGELOG.md`.

## Changes to this document

Proposed changes to GOVERNANCE.md are opened as a pull request and require approval from all active maintainers.
