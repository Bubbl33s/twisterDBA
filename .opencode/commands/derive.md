---
description: Parse a project doc and generate atomic OpenSpec changes with user stories and tasks
---

Parse a project description document and generate OpenSpec changes with user stories, specs, and granular atomic tasks.

**Input**: Path to a project description markdown file (e.g., `mvp-terminal-db-admin.md`).
If no path given, scan root for `mvp-*.md` files and ask.

**Steps**

1. **Read the document** fully. Identify phases, features, tasks, dependencies, risks, tech stack.

2. **Extract user stories** per phase:
   ```
   As a <role>, I want <capability> so that <benefit>
   ```

3. **For each phase, create an OpenSpec change**:
   ```bash
   openspec new change "phase-N-<slug>"
   ```

4. **Generate artifacts per phase** (read `openspec/config.yaml` first):
   - **proposal.md**: Motivation, scope, impact from document context
   - **specs/**: Delta specs per capability. Every requirement has ≥1 WHEN/THEN scenario using `#### Scenario:` (exactly 4 hashtags)
   - **design.md**: Architectural decisions, Rust patterns, ADRs from the document
   - **tasks.md**: Break document tasks into atomic subtasks (max 2h each). Reference src/ file paths. Include cargo verification steps.

5. **Task granularity rules**:
   - Atomic, verifiable, ≤2h each
   - Format: `- [ ] X.Y Description` under `## X. Group Name`
   - Reference specific `src/` module paths
   - Every task group ends with: cargo build, clippy, fmt, test

6. **Output summary**: phases processed, changes created, total tasks, spec domains.

**Guardrails**
- Always read `openspec/config.yaml` before generating
- Read the document fully for domain context
- Use `openspec instructions` for artifact templates
- Never copy context/rules blocks into artifacts
