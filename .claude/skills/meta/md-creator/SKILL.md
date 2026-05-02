---
name: meta:md-creator
version: 1.0.0
description: Creates well-structured Markdown documents — READMEs, technical documentation,
  ADRs, guides, changelogs, blog posts, and any textual content with semantic formatting.
  Activate when the user asks for a Markdown document, a README, documentation, a usage
  guide, or when the target is GitHub, Notion, GitBook, or any Markdown renderer.
  Trigger phrases: "write a README for my project", "create documentation for X",
  "build a Y guide in markdown", "generate a changelog", "document the API for Z".
  Do NOT activate for: documents for printing or formal delivery (pdf-creator),
  slides or presentations, content rendered as interactive HTML, or short inline
  responses that do not need a file.
prerequisites: bash, present_files
---

# meta:md-creator

Produces Markdown documents with correct semantic structure, valid heading hierarchy, and
formatting consistent with the target renderer. Eliminates the most common errors — multiple
H1s, skipped heading levels, code blocks without language, inconsistently indented lists —
that cause incorrect rendering or confusing hierarchy across different parsers.

## Discovery

**1. Document type**
README, API documentation, ADR, user guide, changelog, blog post, technical report,
specification? Determines expected section structure.

**2. Render target**
GitHub (GFM), Notion, GitBook, MkDocs, Hugo, Docusaurus, or generic renderer?
Determines which extensions to use (GFM alerts, YAML frontmatter, MDX directives, etc.).

**3. Audience**
Developers (can read dense technical content), end users (need accessible language),
or executives (executive summary at top)? Defines tone and information density.

**4. Existing assets**
Are there diagrams, screenshots, code examples, or links to include?
Relative links only work in a repository context — verify whether the target supports
relative paths.

**5. Frontmatter**
Does the target use YAML frontmatter (Jekyll, Hugo, Docusaurus, Notion API)?
Gather required fields: title, date, author, tags, description.

## Production rules

### Heading hierarchy

One single `# H1` per document — the main title. Never skip levels:

```
✓  # Title
   ## Section
   ### Subsection

✗  # Title
   ### Subsection  ← skips H2
```

Do not use headings for visual emphasis — use them for semantic structure only.
If content does not warrant a navigable section, use bold or a paragraph, not a heading.

### Semantic formatting

| Element      | When to use                                             | When not to use             |
|--------------|---------------------------------------------------------|-----------------------------|
| **Bold**     | Key terms, important warnings                           | Generic decorative emphasis |
| *Italic*     | Technical terms on first occurrence, titles of works    | General emphasis            |
| `code`       | Commands, file names, variables, literal values         | Visual emphasis             |
| > Blockquote | Quotes, notes, callouts (when alerts are not supported) | Visual indentation          |

### Code blocks

Always declare the language — without it, no syntax highlighting:

````markdown
```typescript
const x: number = 42
```

```bash
npm install --save-dev typescript
```

```json
{ "name": "my-project", "version": "1.0.0" }
```
````

Common languages: `typescript`, `javascript`, `python`, `kotlin`, `java`,
`go`, `bash`, `shell`, `sql`, `yaml`, `json`, `markdown`, `dockerfile`,
`makefile`, `html`, `css`, `jsx`, `tsx`.

### Lists

Ordered when sequence matters (installation steps, priority).
Unordered when sequence does not matter (features, dependencies).
Sublist indentation: 2 spaces (GFM) or 4 spaces (CommonMark).

### Tables

```markdown
| Column A | Column B | Column C |
|----------|----------|----------|
| value    | value    | value    |
```

Separator row `|---|` is mandatory — without it the table does not render in GFM.

### GitHub Flavored Markdown (when target is GitHub)

```markdown
> [!NOTE]
> Supplementary information.

> [!WARNING]
> Attention — may cause issues.

> [!IMPORTANT]
> Critical information for the user.

- [x] Completed task
- [ ] Pending task
```

### YAML frontmatter (when target supports it)

```yaml
---
title: Document Title
date: 2025-04-30
author: Author Name
description: One-line summary for SEO and previews
tags: [typescript, authentication, oidc]
---
```

### Document type templates

**README:**

```
# Project Name
One-line description.

## Installation
## Usage
## Configuration
## Contributing
## License
```

**Changelog (Keep a Changelog):**

```
# Changelog

## [Unreleased]

## [1.2.0] - 2025-04-30
### Added
### Changed
### Fixed
### Removed
```

**ADR (Architecture Decision Record — Nygard):**

```
# ADR-NNNN: Decision Title

## Status
Accepted | Proposed | Superseded by ADR-XXXX

## Context
## Decision
## Alternatives Considered
## Consequences
```

### Delivery

Long documents (> 100 lines): save to `.claude/prompts/[name].md` or the appropriate
path for the document type, and deliver via `present_files`.
Short documents: deliver inline in the chat as a Markdown code block.

## Validation

<checklist>

**Hierarchy**

- [ ] Exactly one `# H1` in the document
- [ ] No skipped heading levels (H1→H3 without intermediate H2)
- [ ] Headings used for semantic structure, not visual emphasis
- [ ] No H4 or lower headings — restructure hierarchy if needed

**Code**

- [ ] Every code block has a language declared after the three backticks
- [ ] Inline code (`backtick`) used for commands, files, variables
- [ ] No code blocks without a language when the language is identifiable

**Lists**

- [ ] Consistent indentation in sublists (2 or 4 spaces, never mixed)
- [ ] Ordered lists used only when sequence matters

**Tables**

- [ ] Separator row `|---|` present in all tables
- [ ] Header row present in all tables

**Target**

- [ ] GFM alerts (`> [!NOTE]`) used only when target is GitHub
- [ ] Frontmatter present when target uses it (Jekyll, Hugo, Notion API)
- [ ] Relative links used only when target is a git repository
- [ ] No images with relative paths when target is not a git repository

</checklist>
