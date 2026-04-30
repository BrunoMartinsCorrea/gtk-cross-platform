# Prompts Index

Generated prompts live here. Each file is a self-contained AI prompt created via `/project:create-prompt`.

To add a new prompt: `/project:create-prompt <context-description>`

---

<!-- entries are sorted alphabetically by title -->
- [Code Review with Report Generation](code-review-with-reporting-file.md) — Review changed Rust/UI files against codebase standards and write a severity-classified Markdown report to docs/review-<date>.md
- [Migrate GTK4 UI XML to Blueprint](migrate-gtk4-adwaita-xml-to-blueprint.md) — Convert all GtkBuilder XML (.ui) files to Blueprint (.blp) format and wire blueprint-compiler into build.rs, preserving the exact rendered UI
- [Rancher Desktop Runtime Detection Mismatch](diagnose-why-containers-are-visible-in.md) — Diagnose why the GTK4 app reports RuntimeNotAvailable while containers are visible in Rancher Desktop; maps socket/binary/namespace failures to code fixes
