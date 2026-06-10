---
description: Fast-forward — create all artifacts for a change in one go
---

Fast-forward through artifact creation — generate everything needed to start implementation in one go.

**Input**: The argument after `/opsx-ff` is the change name (kebab-case), OR a description of what to build.

**Steps**

1. **If no input provided, ask what they want to build**
   Use AskUserQuestion to ask: "What change do you want to work on? Describe what you want to build or fix."
   Derive a kebab-case name from their description.

2. **Create the change directory**
   ```bash
   openspec new change "<name>"
   ```

3. **Get the artifact build order**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse: `applyRequires`, `artifacts`, `planningHome`, `changeRoot`.

4. **Create artifacts in sequence until apply-ready**
   Use TodoWrite to track. Loop through ready artifacts in dependency order:
   - Get instructions: `openspec instructions <artifact-id> --change "<name>" --json`
   - Read `openspec/config.yaml` for context
   - Read dependency files
   - Create artifact using template, write to resolved output path
   - **For tasks artifact**: Include cargo verification steps at end

5. **Show final status**: `openspec status --change "<name>"`
   Summary: change name, artifacts, "Ready for implementation. Run /opsx-apply."

**Guardrails**
- Create ALL artifacts from `apply.requires`
- Context/rules are constraints — never copy into artifact files
- If change exists, ask to continue instead
