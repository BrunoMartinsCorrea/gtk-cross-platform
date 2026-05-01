---
name: meta:mermaid-creator
version: 1.0.0
description: Creates Mermaid.js diagrams — flowcharts, sequence, state, ER, Gantt,
  and class diagrams. Activate when the user asks for a diagram, process flow, state
  map, data model, timeline, or any declarative structural visualization.
  Trigger phrases: "create a flowchart of X", "model the authentication flow",
  "draw the ER diagram", "make a sequence diagram for Y", "state map of the order",
  "project timeline in Gantt".
  Do NOT activate for: charts with real numeric/interactive data, high-fidelity C4
  diagrams for formal documentation, or free-form vector illustrations.
prerequisites: present_files
---

# meta:mermaid-creator

Produces syntactically valid Mermaid.js diagrams that render without errors on GitHub,
Notion, GitBook, MkDocs, and the Claude artifact environment. Prevents the most common
silent failures — labels without quotes, emojis, `end` as node text — that produce a
blank screen with no useful error message.

## Discovery

**1. Diagram type**
Which structure does the user want to represent?

| Intent                           | Mermaid type      |
|----------------------------------|-------------------|
| Process with decisions           | `flowchart`       |
| Message exchange between systems | `sequenceDiagram` |
| Entity lifecycle                 | `stateDiagram-v2` |
| Database model                   | `erDiagram`       |
| Project schedule                 | `gantt`           |
| OOP class structure              | `classDiagram`    |
| Set proportions                  | `pie`             |

**2. Direction**
For flowcharts: `LR` (left→right) for horizontal flows and pipelines;
`TD` (top→down) for hierarchies, trees, and vertical flows.

**3. Complexity and scope**
Approximately how many nodes/actors/entities? Diagrams with more than 20 nodes become
illegible — suggest splitting into subdiagrams if needed.

**4. Render target**
GitHub, Notion, GitBook, `.mermaid` file, or inline in chat? Affects delivery format
(code block vs file).

## Production rules

### Mandatory opening — type on the first line

```
flowchart LR
sequenceDiagram
stateDiagram-v2
erDiagram
gantt
classDiagram
pie title Distribution
```

The type must be the first line of the block — no comments or whitespace before it.

### Rules by type

**flowchart / graph**

```mermaid
flowchart LR
    A["Start"] --> B{"User authenticated?"}
    B -- Yes --> C["Dashboard"]
    B -- No --> D["Login"]
    D --> E["Validate credentials"]
    E --> B

    subgraph auth["Authentication"]
        D
        E
    end
```

Critical rules:

- Labels with spaces, commas, parentheses, or special characters: **always in quotes**
  `A["Label with space"]` — without quotes the parser fails silently
- `end` as node text breaks the parser — use `finish`, `done`, `End`, `Terminate`
- Subgraphs: `subgraph id["Label"] ... end` — `end` is a reserved keyword here, not node text
- No emojis anywhere

**sequenceDiagram**

```mermaid
sequenceDiagram
    participant U as User
    participant A as App
    participant S as Auth Server

    U->>A: POST /login
    A->>S: Validate credentials
    S-->>A: JWT Token
    A-->>U: 200 OK + token
```

Rules:

- `participant X as Alias` for labels with spaces
- `->>` synchronous, `-->>` response, `--)` asynchronous
- No `Note` before declaring the corresponding `participant`
- Activate/deactivate with `activate`/`deactivate` — do not nest more than 2 levels

**stateDiagram-v2**

```mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Processing: start
    Processing --> Approved: success
    Processing --> Rejected: failure
    Approved --> [*]
    Rejected --> [*]

    state Processing {
        [*] --> Validating
        Validating --> Executing
        Executing --> [*]
    }
```

Rules:

- Initial state: `[*]` — not `start` or `initial`
- Always `stateDiagram-v2` — v1 has different and limited syntax
- Composite states with `state Name { ... }`

**erDiagram**

```mermaid
erDiagram
    USER {
        int id PK
        string email UK
        string name
    }
    ORDER {
        int id PK
        int user_id FK
        decimal amount
        string status
    }
    USER ||--o{ ORDER : "places"
```

Cardinalities: `||--||` (1:1), `||--o{` (1:N), `}o--o{` (N:M)

**gantt**

```mermaid
gantt
    title Q2 Schedule
    dateFormat YYYY-MM-DD
    section Backend
        Authentication    :done, auth, 2025-04-01, 2025-04-14
        Payments API      :active, api, 2025-04-15, 2025-04-30
    section Frontend
        Dashboard         :front, 2025-04-20, 2025-05-10
```

Rules:

- `dateFormat` mandatory before any task
- Status: `done`, `active`, `crit` (critical) — optional

### Delivery

Simple diagrams (< 30 lines): deliver as ` ```mermaid ` block inline in chat — renders visually.
Complex diagrams: save to `/mnt/user-data/outputs/[name].mermaid` and call `present_files`.

## Validation

<checklist>

**General syntax**

- [ ] Diagram type declared on the first line of the block
- [ ] No emojis anywhere
- [ ] No `end` as node text in flowchart — only as subgraph closing keyword
- [ ] Labels with spaces, commas, or parentheses enclosed in double quotes

**By type**

- [ ] flowchart: direction declared (LR, TD, RL, BT)
- [ ] stateDiagram: version `stateDiagram-v2`, not `stateDiagram`
- [ ] stateDiagram: initial state `[*]`, not `start`
- [ ] sequenceDiagram: `participant` declared before `Note` for the same actor
- [ ] erDiagram: cardinalities using correct notation (`||--o{`, etc.)
- [ ] gantt: `dateFormat` declared before the first task

**Readability**

- [ ] Diagram has at most ~20 nodes — if more, split or justify
- [ ] Subgraphs used to group related nodes when there are > 8 nodes

**Negative assertions**

- [ ] No emoji in Mermaid code
- [ ] No label with special characters without quotes
- [ ] No `stateDiagram` (v1) — only `stateDiagram-v2`

</checklist>
