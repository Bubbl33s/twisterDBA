---
name: openspec-ff-change
description: Fast-forward through OpenSpec artifact creation for twisterDBA. Use when the user wants to quickly create all artifacts needed for implementation without stepping through each one individually.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.4.1"
---

Fast-forward through artifact creation — generate everything needed to start implementation in one go.

**Input**: The user's request should include:
- A change name (kebab-case) — will use that name directly
- A description of what they want to build — will derive a kebab-case name

**Steps**

1. **Determine input type and get context**

   a. **If input is a change name** (kebab-case format):
      - Use the provided name directly
      - Check if change already exists, if so ask user if they want to continue it

   b. **If input is a description**:
      - Derive a kebab-case name (e.g., "add query history" → `add-query-history`)

   c. **If no input provided**:
      - Use the **AskUserQuestion tool** to ask what they want to build

   **IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

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

   Use TodoWrite to track progress. Loop through ready artifacts in dependency order:

   a. **For each ready artifact**:
      ```bash
      openspec instructions <artifact-id> --change "<name>" --json
      ```
      - Read dependency files for context
      - Create artifact using template, applying context/rules as constraints
      - **CRITICAL for tasks**: Read `openspec/config.yaml` to get project rules
        and `mvp-terminal-db-admin.md` for domain context
      - Verify file exists after writing

   b. **Continue until all `applyRequires` artifacts done**

   c. **Ask user for clarification if context is unclear**

5. **Show final status**
   ```bash
   openspec status --change "<name>"
   ```

**Output**: Summarize change name, artifacts created, and prompt `/opsx-apply` to implement.

**Guardrails**
- Create ALL artifacts from `apply.requires`
- Always read dependency artifacts before creating new ones
- Use `openspec/config.yaml` context and rules — never copy them into artifacts
- If change exists, suggest continuing instead of creating new
