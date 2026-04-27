# Security Policy

## Supported versions

| Version          | Supported |
|------------------|-----------|
| `main` (nightly) | Yes       |
| 0.1.x            | Yes       |

## Scope

This policy covers vulnerabilities in the **Rust application code** in this repository.

Out of scope (report upstream instead):

- Bugs in Docker, Podman, or containerd runtimes
- Vulnerabilities in GTK4, LibAdwaita, or GNOME Platform libraries
- Flatpak sandbox escape bugs (report to [flatpak/flatpak](https://github.com/flatpak/flatpak/security))

## Reporting a vulnerability

**Do not open a public issue.**
Use [GitHub Private Security Advisories](https://github.com/example/gtk-cross-platform/security/advisories/new) to
report privately.

Include:

- Description of the vulnerability and affected component
- Steps to reproduce or a proof-of-concept
- Potential impact

## Response timeline

| Milestone                      | Commitment      |
|--------------------------------|-----------------|
| Initial acknowledgement        | Within 72 hours |
| Triage and severity assessment | Within 7 days   |
| Fix and coordinated disclosure | Within 90 days  |

## Disclosure

After a fix is released:

1. A CVE is requested and published.
2. The `Security` section of [CHANGELOG.md](CHANGELOG.md) is updated.
3. The advisory is made public.

Credit is given to the reporter in the advisory unless they prefer to remain anonymous.
