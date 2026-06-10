---
name: openspec-new-change
description: Start a new OpenSpec change scaffold for twisterDBA. Creates the change folder and waits for artifact generation.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.4.1"
---

Start a new change scaffold for twisterDBA.

**Input**: Change name (kebab-case) or description to derive one.

**Steps**

1. **Determine change name**
   - From kebab-case input or derive from description
   - Ask if no input provided

2. **Create the change directory**
   ```bash
   openspec new change "<name>"
   ```

3. **Show next steps**
   - Ready to create: proposal
   - Use `/opsx-continue` for step-by-step or `/opsx-ff` for all at once

**Guardrails**
- Use descriptive kebab-case names
- If change exists, ask to continue instead
