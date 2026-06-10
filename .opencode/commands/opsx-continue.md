---
description: Create the next artifact in the dependency chain, one at a time
---

Step-by-step artifact creation — create one artifact at a time.

**Input**: Optionally a change name. If omitted, infer from context or prompt.

**Steps**

1. **Select the change**
   - Use provided name or infer from context
   - If ambiguous, run `openspec list --json` and ask

2. **Check status**
   ```bash
   openspec status --change "<name>" --json
   ```

3. **Show artifact status**
   - ✓ done, ◆ ready, ○ blocked
   - List which are ready vs waiting on dependencies

4. **Create the first ready artifact**
   ```bash
   openspec instructions <artifact-id> --change "<name>" --json
   ```
   - Read `openspec/config.yaml` for context
   - Read completed dependencies
   - Create artifact using template
   - Show what becomes available next

**Guardrails**
- One artifact at a time
- Read config.yaml for project context
- If tasks, include cargo verification steps
