---
description: Start a new OpenSpec change scaffold
---

Start a new change. Creates the change folder and `.openspec.yaml`.

**Input**: Change name (kebab-case) or description to derive one.

**Steps**

1. **Determine name** — from input or AskUserQuestion
2. **Create scaffold**
   ```bash
   openspec new change "<name>"
   ```
3. **Show next steps** — use `/opsx-continue` or `/opsx-ff` to create artifacts

**Guardrails**
- Use descriptive kebab-case names
- If change exists, ask to continue instead
