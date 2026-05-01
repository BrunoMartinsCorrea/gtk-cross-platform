---
name: meta:create-prompt
version: 1.0.0
description: Generate a high-quality, self-contained AI prompt for a given context. Usage: /meta:create-prompt <context-description>
---

# meta:create-prompt

Generate a production-ready AI prompt for the context described in `$ARGUMENTS`.
This command is self-contained — run it fresh without prior conversation context.

**Usage:** `/meta:create-prompt <context-description>`
**Output:** `.claude/prompts/<slug>.md`

---

## Input Acceptance Criteria

Before doing any work, verify the input is valid. If any criterion fails, stop immediately and
report which criterion failed — do not attempt to generate a prompt.

| #  | Criterion                                        | How to verify                                                                                                                                                                                                                                                                    |
|----|--------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| I1 | `$ARGUMENTS` is non-empty                        | If empty or whitespace-only, fail with: "Provide a context description. Example: `/meta:create-prompt audit de segurança para APIs REST`"                                                                                                                                        |
| I2 | `$ARGUMENTS` is at least 5 words                 | Count space-separated tokens; if fewer than 5, fail with: "Context too vague — add more detail about domain, audience, and goal."                                                                                                                                                |
| I3 | `$ARGUMENTS` describes a specific domain or task | Verify it contains at least one noun identifying a domain (e.g., "testes", "CI", "segurança", "refactor", "documentação", "tests", "security", "pipeline"); if purely generic ("fazer algo bom", "do something good"), fail with: "Context must name a specific domain or task." |
| I4 | The `.claude/prompts/` directory exists          | Run `ls .claude/prompts`; if missing, create it — this is a setup action, not a blocking failure.                                                                                                                                                                                |

---

## Execution Plan

Execute in this exact order. Do not skip steps; each step gates the next.

### Step 1 — Analyse the context and derive metadata

Parse `$ARGUMENTS` to extract:

| Field           | How to derive                                                                                                                                                                                                                                                                                                                      |
|-----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Domain**      | The subject area (e.g., "GTK4", "API security", "post-mortem analysis")                                                                                                                                                                                                                                                            |
| **Goal**        | What outcome the prompt should produce (e.g., "gap report", "refactored code")                                                                                                                                                                                                                                                     |
| **Audience**    | Who will execute this prompt (default: Claude Code agent; override if context implies a specific persona)                                                                                                                                                                                                                          |
| **Trigger**     | When should this prompt be used?                                                                                                                                                                                                                                                                                                   |
| **Output type** | File, report, code, table, list, explanation?                                                                                                                                                                                                                                                                                      |
| **Language**    | Dominant language of `$ARGUMENTS` — count Portuguese-specific tokens (e.g., words ending in -ção, -agem, -dade, or common PT verbs like "analisar", "criar", "verificar") vs English tokens; if ≥ 60 % are Portuguese, language = "pt"; otherwise language = "en". This determines every section language in the generated prompt. |

**Slug derivation** — apply these rules in order:

1. Lowercase the entire string.
2. Transliterate accented characters: á→a, à→a, ã→a, â→a, é→e, ê→e, í→i, ó→o, õ→o, ô→o, ú→u, ü→u, ç→c, ñ→n.
3. Replace spaces and underscores with hyphens.
4. Remove any character that is not a lowercase letter, digit, or hyphen.
5. Collapse consecutive hyphens into one.
6. Truncate to 40 characters, cutting at a hyphen boundary if possible.

Example: "Auditoria de Segurança para APIs REST" → `auditoria-de-seguranca-para-apis-rest`

**Title derivation** — Title Case, 3–6 words, no accents required (it is display text, not a filename).

**Collision check** — Run `ls .claude/prompts/<slug>.md`. If the file exists:

- Print: "Prompt `<slug>.md` already exists (version `<version from frontmatter>`). Overwriting with version
  `<current_version + 0.1>`."
- Increment the version field in the new file's frontmatter by 0.1 (e.g., 1.0 → 1.1).
- Do not abort — continue with the updated version.

### Step 2 — Research the domain

Apply this decision tree:

```
Does $ARGUMENTS mention a file path, module name, or infrastructure component? → YES: read that file/module (max 2 files)
Does the domain involve a custom pattern specific to this codebase?           → YES: read CLAUDE.md §<relevant section> (1 file)
Does the domain reference a workflow, CI, or build system?                    → YES: read .github/workflows/ci.yml or Makefile (1 file)
Is the domain purely general (e.g., "revisão de PRs", "code review")?        → SKIP research entirely
Total files read ≤ 4. Stop at 4 even if more seem relevant.
```

Record which files were read — reference them in the `## Contexto` section of the generated prompt.

### Step 3 — Draft the prompt using the standard template

Apply the **RISEN + CO-STAR hybrid** framework. Use the Output Template (§Output Template) exactly — do not omit or
reorder sections.

**Language rule:** every section heading and body text must be in the language determined in Step 1. Do not mix
languages. If language = "pt", use the Portuguese heading names in the template; if language = "en", translate them to
English equivalents.

**Section constraints:**

| Section                          | Constraint                                                                                                                                                                                                                            |
|----------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `Objetivo`                       | One declarative sentence, ≤ 25 words.                                                                                                                                                                                                 |
| `Restrições`                     | 4–8 bullets. Ordering: (1) scope boundary, (2) forbidden operations, (3) quality floor, (4) layer/boundary rule, (5) output format/length. Each bullet starts with an imperative verb.                                                |
| `Passos`                         | 4–8 ordered steps. Each step: `**<Verb phrase>** — <what to do>. Done when: <objective criterion>.`                                                                                                                                   |
| `Exemplos`                       | Include only if the domain meets ≥ 1 of: has a lookup table, involves format transformation, or has non-obvious input→output mapping. If none apply, omit the section entirely. Include 2 examples (contrasting cases) when included. |
| `Formato de Saída`               | Always present. Specify: file type, Markdown heading levels, table columns or JSON schema or bullet depth, max length. Include a skeleton/schema.                                                                                     |
| `Critérios de Aceite de Entrada` | 2–5 rows. Each row: objectively verifiable by the model without user interaction.                                                                                                                                                     |
| `Critérios de Aceite de Saída`   | 3–6 rows. Always include ≥ 1 format-level check (structure, not content).                                                                                                                                                             |

### Step 4 — Write the file

Write the complete prompt to `.claude/prompts/<slug>.md` in a single Write operation.
Do not split across multiple writes.

### Step 5 — Update the index

Read `.claude/prompts/INDEX.md` (create it if it does not exist). Add one line:

```
- [<title>](<slug>.md) — <one-line description of when to use this prompt>
```

Sort all entries alphabetically by title (case-insensitive). If the slug already existed, replace its existing line
rather than adding a duplicate. Write the updated file.

### Step 6 — Self-validate before reporting

Before printing the terminal summary, verify each output criterion below. For every failure:

- Correct the issue immediately (re-write the file or index).
- If a criterion cannot be corrected automatically, append it to the terminal summary under `## Validation failures`.

| #   | Criterion                                                                                                                                                   | How to verify                                                                                                         |
|-----|-------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------|
| O1  | `.claude/prompts/<slug>.md` exists                                                                                                                          | Use Read tool on the path                                                                                             |
| O2  | All required sections present: Role, Contexto, Objetivo, Restrições, Passos, Formato de Saída, Critérios de Aceite de Entrada, Critérios de Aceite de Saída | Scan for each heading                                                                                                 |
| O3  | Objetivo section is ≤ 25 words                                                                                                                              | Count words in the Objetivo paragraph                                                                                 |
| O4  | Input Acceptance Criteria table has ≥ 2 rows                                                                                                                | Count table rows                                                                                                      |
| O5  | Output Acceptance Criteria table has ≥ 3 rows, including ≥ 1 format-level check                                                                             | Count rows and identify format check                                                                                  |
| O6  | Slug contains only `[a-z0-9-]`, no consecutive hyphens, ≤ 40 chars                                                                                          | Validate with regex `^[a-z0-9]+(-[a-z0-9]+)*$` and length                                                             |
| O7  | `.claude/prompts/INDEX.md` contains exactly one line referencing `<slug>.md`                                                                                | Grep for slug; confirm count = 1                                                                                      |
| O8  | Prompt language matches the language determined in Step 1                                                                                                   | If language = "pt", section headings must include "Papel", "Contexto", "Objetivo"; if "en", check English equivalents |
| O9  | Each step in `Passos` contains a "Done when:" clause                                                                                                        | Grep for "Done when:" in the Passos section                                                                           |
| O10 | `Restrições` bullets begin with imperative verbs                                                                                                            | Check first word of each bullet is a verb                                                                             |

### Step 7 — Print terminal summary

```
Prompt created:  .claude/prompts/<slug>.md
Title:           <title>
Domain:          <domain>
Audience:        <audience>
Language:        <pt | en>
Version:         <version>
Usage:           <how to invoke or reference the prompt>
Validation:      <"All criteria passed" or list of any O-criteria that failed and were auto-corrected>
```

---

## Output Template

Write `.claude/prompts/<slug>.md` with this exact structure:

````markdown
---
name: <title>
description: <one-line description — used to decide relevance, so be specific>
domain: <domain keyword>
audience: <who runs this prompt>
language: <pt | en>
version: 1.0
created: YYYY-MM-DD
---

# <Title>

> **Contexto:** <domain/purpose in one sentence>
> **Audiência:** <who executes this prompt>
> **Uso:** `<invocation example>`

## Papel (Role)

<Specific expert persona. Concrete: "Você é um engenheiro de software sênior especializado em
GTK4/Rust com 10 anos de experiência em GNOME." Not: "You are a helpful assistant.">

## Contexto

<Background the model needs. Include: relevant architecture, constraints already decided,
what must NOT be changed, domain-specific vocabulary, and any files read in Step 2.>

## Objetivo

<Single declarative sentence stating the desired outcome. ≤ 25 words.>

## Restrições

- <Scope boundary: what this prompt covers and what it excludes>
- <Forbidden operation: what the model must not do, with reason when non-obvious>
- <Quality floor: minimum standard that must be met>
- <Layer/boundary rule if applicable>
- <Output constraint: format, length, or naming rule>

## Passos

1. **<Verb phrase>** — <what to do and how>. Done when: <objective criterion>.
2. **<Verb phrase>** — <what to do and how>. Done when: <objective criterion>.
3. **<Verb phrase>** — <what to do and how>. Done when: <objective criterion>.
4. **<Verb phrase>** — <what to do and how>. Done when: <objective criterion>.

## Exemplos

<!-- Include only if domain has lookup table, format transformation, or non-obvious mapping.
     Omit entirely otherwise. When included, always provide 2 contrasting examples. -->

**Entrada:**
```
<concrete input example>
```

**Saída esperada:**
```
<concrete expected output — shows structure, not just content>
```

---

**Entrada:**
```
<second example — contrasting case>
```

**Saída esperada:**
```
<expected output for second example>
```

## Formato de Saída

<Exact specification: file type, Markdown structure (heading levels), table columns, JSON schema,
or bullet-point format. Include max length.>

```
<Skeleton / schema — fill in the exact structure, not placeholder text>
```

## Input Acceptance Criteria

Before executing the main task, verify these criteria. If any fail, stop and report the failure.

| # | Criterion | How to verify |
|---|-----------|---------------|
| I1 | <Precondition that must be true before starting> | <Objective check: read file, count lines, grep for pattern> |
| I2 | <Second precondition> | <How to verify it> |

## Output Acceptance Criteria

After completing the task, verify these criteria. If any fail, append a `## Validation errors`
section to the output listing the failures.

| # | Criterion | How to verify |
|---|-----------|---------------|
| O1 | <Condition that must be true in the output> | <Objective check: count items, validate format, grep result> |
| O2 | <Second output condition> | <How to verify it> |
| O3 | <Format-level check: structure, heading count, schema validity> | <How to verify it> |
````

---

## Output Acceptance Criteria

These are the criteria verified in **Step 6**. Listed here for reference.

| #   | Criterion                                                                                                                                                   | How to verify                         |
|-----|-------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------|
| O1  | `.claude/prompts/<slug>.md` exists                                                                                                                          | Use Read tool on the path             |
| O2  | All required sections present: Role, Contexto, Objetivo, Restrições, Passos, Formato de Saída, Critérios de Aceite de Entrada, Critérios de Aceite de Saída | Scan for each heading                 |
| O3  | Objetivo section is ≤ 25 words                                                                                                                              | Count words in the Objetivo paragraph |
| O4  | Input Acceptance Criteria table has ≥ 2 rows                                                                                                                | Count table rows                      |
| O5  | Output Acceptance Criteria table has ≥ 3 rows, including ≥ 1 format-level check                                                                             | Count rows and identify format check  |
| O6  | Slug is `^[a-z0-9]+(-[a-z0-9]+)*$` and ≤ 40 chars                                                                                                           | Validate with regex and length        |
| O7  | INDEX.md contains exactly one line referencing `<slug>.md`                                                                                                  | Grep for slug; count = 1              |
| O8  | Prompt language matches language determined in Step 1                                                                                                       | Check section heading names           |
| O9  | Each step in Passos contains a "Done when:" clause                                                                                                          | Grep for "Done when:"                 |
| O10 | Restrições bullets begin with imperative verbs                                                                                                              | Check first word of each bullet       |

---

## Framework Reference (for generating prompts)

| Framework   | Best for                     | Key contribution                                          |
|-------------|------------------------------|-----------------------------------------------------------|
| **RISEN**   | Complex multi-step tasks     | Explicit Steps + End Goal + Narrowing prevent scope creep |
| **CO-STAR** | Creative or open-ended tasks | Context + Style + Tone + Audience align model persona     |
| **CARE**    | Quick operational prompts    | Context + Ask + Rules + Examples — minimal boilerplate    |
| **APE**     | Audience-sensitive output    | Forces explicit audience definition before writing begins |

**The most important principle:** specificity defeats cleverness. Explicit numeric constraints,
concrete examples, and measurable acceptance criteria outperform elaborate reasoning instructions.
Fill every section with measurable, verifiable statements — never leave sections as vague guidance.

**Anti-patterns to avoid in generated prompts:**

- ❌ "Be helpful and thorough" (not measurable)
- ❌ "Use your best judgment" (abdicates constraint definition)
- ❌ Examples section with only one example (fails to show contrast)
- ❌ Output format as prose description instead of a skeleton/schema
- ❌ Acceptance criteria that say "ask the user if unclear" (not self-contained)
- ❌ Steps without a "Done when:" clause (no objective completion signal)
- ❌ Steps that describe state instead of actions ("The code should be clean" vs "Remove all `#[allow(dead_code)]`
  attributes")
- ❌ Restrições with bullets that don't start with imperative verbs
- ❌ Slug with accented characters, spaces, or uppercase letters
