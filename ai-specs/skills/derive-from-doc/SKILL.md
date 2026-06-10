---
name: derive-from-doc
description: Parse a project description document and generate atomic OpenSpec changes with user stories, specs, and granular tasks. Use when the user provides a project description markdown file and wants to derive structured specs and implementation tasks from it.
author: twisterDBA
version: 1.0.0
---

# derive-from-doc Skill

Use when the user wants to extract structured user stories and atomic tasks
from a project description document.

## Instructions

**Input**: A path to a project description markdown file (e.g., `mvp-terminal-db-admin.md`).
If no path is given, scan the project root for `mvp-*.md` or `*-spec.md` files and ask.

**Steps**

1. **Read the project description document**

   Parse the full document. Identify:
   - Phases/sections (look for `## Phase` or `## Fase` headings)
   - Features per phase
   - Technical tasks listed under each phase
   - Dependencies between phases
   - Estimated timelines, risk notes, architectural diagrams
   - Technology stack mentions (Rust crates, patterns, etc.)

2. **Extract user stories per phase**

   For each phase in the document, derive user stories in this format:

   ```
   As a <role>, I want <capability> so that <benefit>
   ```

   Roles include: "database administrator", "developer", "terminal user", "system operator".

   Each user story must map to observable terminal behavior.

3. **For each phase, create an OpenSpec change**

   ```bash
   openspec new change "phase-N-<slug>"
   ```

   Derive a kebab-case slug from the phase name (e.g., `phase-1-foundations`).

4. **Generate artifacts for each phase change**

   For each phase change, generate in order:

   a. **proposal.md**: Capture the motivation from the document, scope, and impact.
      List capabilities as kebab-case spec domains.

   b. **specs/**: Create delta spec files per capability.
      - Each requirement derived from the document's task descriptions
      - Every requirement has ≥1 WHEN/THEN scenario
      - Use `#### Scenario:` for scenarios (exactly 4 hashtags)

   c. **design.md**: Extract architectural decisions from the document's code
      examples, patterns, and risk sections. Document ADRs, async patterns,
      TEA state management, ownership model.

   d. **tasks.md**: Break each technical task from the document into atomic
      subtasks (max 2h each). Reference specific src/ file paths.
      Include cargo verification steps at the end of each task group.

5. **Task granularity rules**

   Each task must be:
   - Atomic: one concrete action (e.g., "Define AppEvent enum in src/events.rs")
   - Verifiable: has a clear done condition (compiles, test passes, renders)
   - ≤2 hours: split larger items into subtasks
   - Numbered: `- [ ] X.Y Description` under `## X. Group Name` headings
   - File-referencing: mentions the specific module path

6. **Save summary**

   After generating all phase changes, output a summary to the user showing:
   - How many phases were processed
   - How many changes created (one per phase)
   - Total atomic tasks across all phases
   - Which spec domains were created

**Output Example**

```
Derived 6 changes from mvp-terminal-db-admin.md:

phase-1-foundations      (12 tasks) — Event loop, FSM, layout
phase-2-data-layer       (10 tasks) — SQLx, connections, cancellation
phase-3-schema-explorer   (8 tasks) — Tree, lazy loading
phase-4-sql-editor       (15 tasks) — Vim editor, history, highlight
phase-5-result-grid      (12 tasks) — Virtual scroll, export
phase-6-polish            (8 tasks) — Config, distribution
─────────────────────────────────────────
Total: 65 atomic tasks across 6 changes
Spec domains: core, database, explorer, editor, results, ux
```

**Guardrails**
- Always read `openspec/config.yaml` before generating artifacts
- Read `mvp-terminal-db-admin.md` for full domain context
- Use `openspec instructions <artifact-id> --change "<name>" --json` for templates
- Never copy `<context>` or `<rules>` blocks into artifact files
- Tasks must be implementable from the description alone
- If a phase is unclear, note it but make reasonable assumptions
