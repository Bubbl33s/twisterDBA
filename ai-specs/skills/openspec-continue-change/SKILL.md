---
name: openspec-continue-change
description: Create the next artifact in the dependency chain, one at a time. Use for step-by-step control over artifact creation in twisterDBA.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.4.1"
---

Create the next artifact in the dependency chain for a twisterDBA change.

**Input**: Optionally specify a change name. If omitted, infer from context or prompt.

**Steps**

1. **Select the change**
   - Use provided name or infer from context
   - If ambiguous, run `openspec list --json` and ask user

2. **Check status**
   ```bash
   openspec status --change "<name>" --json
   ```

3. **Show artifact status**
   - ✓ for done, ◆ for ready, ○ for blocked
   - List which are ready vs waiting on dependencies

4. **Create the first ready artifact**
   ```bash
   openspec instructions <artifact-id> --change "<name>" --json
   ```
   - Read `openspec/config.yaml` for context
   - Read completed dependencies for context
   - Create artifact file using template
   - Show what becomes available next

**Guardrails**
- Create one artifact at a time, show progress
- Read `openspec/config.yaml` for project context
- If tasks artifact, include cargo verification steps
