---
name: meta:agent-prompt-writer
version: 1.0.0
description: Creates Claude agent prompts in canonical Anthropic format with Role, Context,
  Task, Constraints, and Delivery Format sections. Activate when the user asks to write,
  create, or generate a prompt for an agent, assistant, or AI pipeline. Trigger phrases:
  "create a prompt for agent X", "write instructions for model to do Y", "build system
  prompt for Z", "I need a prompt that does W autonomously".
  Do NOT activate for: executing an existing prompt, using an already-configured agent,
  reviewing code or documentation without prompt creation intent, or creating skills (SKILL.md).
prerequisites: bash, present_files
---

# meta:agent-prompt-writer

Produces Claude agent prompts ready for autonomous execution following the canonical Anthropic
structure: Role + Context + Task + Constraints + Delivery Format. Eliminates unintentional
variation between runs and ensures the agent executes without requiring follow-up or clarification.

## Discovery

### Operating mode

Determine the mode before gathering data:

- **Interactive**: ask directly for gaps. Do not infer where there is real ambiguity.
- **Autonomous**: infer what is missing and record each inference at the top of the generated
  prompt: `> ⚠️ Inference: [what was inferred] — [reason]. Revise if incorrect.`

### What to gather

Extract from history or the user, applying the mode rule for gaps:

**1. Agent identity**
Who is the agent? What expertise or role does it hold? One precise sentence.
Example: "security engineer specializing in OAuth2/OIDC authentication" — not "technical assistant".

**2. Required context**
What does the agent need to know before acting that is not in the task itself?
Include: tech stack, project conventions, current system state, environment constraints.
Omit history that does not affect execution.

**3. Core task**
What should the agent do, in one clear imperative sentence? If the task has substeps,
list them in the order they must be executed.

**4. Constraints**
What is out of scope? Which decisions should the agent not make alone? What formats, tools,
or behaviors are prohibited?
Prefer positive formulation: "include only X" rather than "do not include Y".

**5. Delivery format**
How should the output be structured? File, prose, JSON, numbered list, diagram?
If more than one format is possible, define the condition for each.

**6. Examples (optional)**
Are there input/output pairs that demonstrate expected behavior? If so, include them as
few-shot to calibrate the model.

**7. Optional sections**
Evaluate whether the prompt needs:

- `<thinking>` — internal reasoning block before response (useful for complex analytical
  tasks where intermediate reasoning affects response quality)
- `## Examples` — few-shot section when expected behavior is hard to specify with text alone
- `## Tools` — list of available tools when the agent operates with function calling or MCPs

Required sections for type `agent`: Role, Task, Constraints, Delivery Format.
Context is required when there is system state or non-obvious conventions.
Omit any section that would be empty — an empty section is worse than a missing one.

## Production

### Steps

1. Determine the agent name in kebab-case from user input.

2. Write the frontmatter:
   ```yaml
   ---
   agent: [agent-name-in-kebab-case]
   version: 1.0.0
   model: claude-sonnet-4-20250514
   ---
   ```

3. If there are inferences, record them just after the frontmatter as `> ⚠️ Inference: ...`
   blocks before any section.

4. Write each section following the template below and the writing rules.

5. Run the validation checklist before delivering.

6. Save the generated prompt to `.claude/prompts/[agent-name]-prompt.md`.

### Generated prompt template

```markdown
---
agent: agent-name
version: 1.0.0
model: claude-sonnet-4-20250514
---

## Role

[One sentence in present tense. Who the agent is and what expertise it holds.
Specific enough to calibrate the tone and technical level of the response.]

## Context

[What the agent needs to know before acting: stack, conventions, system state.
Omit if there is no relevant context beyond what is in the task.]

## Task

[Direct imperative. What to do, without ambiguity. If there are substeps, number
them in execution order.]

## Constraints

[What is out of scope, which decisions the agent does not make alone, what formats
or behaviors are prohibited. Positive formulation when possible:
"include only..." rather than "do not include...".]

## Delivery Format

[Exact output structure. Show an example of the structure — do not describe in prose.
Use a code block, template with placeholders, or a real example.]
```

## Writing rules

**Role in one sentence, present tense.**

```
✓  You are a security engineer specializing in OAuth2/OIDC authentication.
✗  You should act as a security expert who has knowledge about OAuth.
```

**Task in imperative, without delegating structure to the model.**

```
✓  Analyze the diff and produce a report with three sections: Critical Issues,
   Suggested Improvements, and Positive Points.
✗  Analyze the code and provide useful feedback on what you find relevant.
```

**Constraints in positive form when possible.**

```
✓  Include only changes with direct impact on security or performance.
✗  Do not include comments about style or personal formatting preferences.
```

**Delivery Format with a concrete example, never prose description.**

```
✓  Produce a markdown file with the structure:
   # [Title]
   ## Critical Issues
   ## Suggested Improvements
   ## Positive Points

✗  Produce a well-structured markdown report with appropriate sections.
```

**XML tags for long or independently-processed sections.**
When Context or Constraints are extensive, delimit with tags to isolate processing:

```xml
<context>
...
</context>
```

**Omit empty sections.** If Context is not needed, remove the entire section.
Sections with unsubstituted placeholders confuse the model about what is a real instruction
versus template structure.

**No tentative language.** "Try to", "if possible", "preferably" signal that the instruction
is not yet defined — define it before including.

## Validation

<checklist>

**Generated prompt structure**

- [ ] Frontmatter present with `agent`, `version`, `model`
- [ ] All required sections present: Role, Task, Constraints, Delivery Format
- [ ] No empty sections or unsubstituted placeholders
- [ ] Inferences recorded as `> ⚠️` blocks when applicable

**Role**

- [ ] One sentence, present tense, with specific expertise
- [ ] Does not use "assistant", "helper", or generic terms without qualification

**Task**

- [ ] Direct imperative, without ambiguity
- [ ] Does not delegate structure decisions to the model that should already be defined
- [ ] No "try to", "if possible", "preferably"

**Constraints**

- [ ] Do not contradict the Task
- [ ] Positive formulation when semantically equivalent

**Delivery Format**

- [ ] Contains concrete example of the structure (template, code block, or real example)
- [ ] Not prose description of what the output should look like

**Token economy**

- [ ] Context omitted if there is no system state or non-obvious conventions
- [ ] XML tags used only where block isolation adds precision
- [ ] No repetition of the same instruction across different sections

</checklist>

## Output format

```
meta:agent-prompt-writer — prompt generated successfully.

File: .claude/prompts/[agent-name]-prompt.md
Sections: Role, Context (if applicable), Task, Constraints, Delivery Format
Inferences: [N] (0 = none)
Checklist: [N]/[total] items validated
```
