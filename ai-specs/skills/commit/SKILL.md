---
name: commit
description: Stage and commit changes with a conventional commit message in English for twisterDBA.
author: twisterDBA
version: 1.0.0
---

# commit Skill

Stage and commit changes with a proper conventional commit message.

## Instructions

1. **Check current state**
   ```bash
   git status
   git diff --stat
   ```

2. **Stage intended files**
   ```bash
   git add <files>
   ```
   Never stage unintended files or secrets.

3. **Generate conventional commit message**
   Format: `type(scope): description`
   Types: feat, fix, refactor, chore, docs, test, style, perf
   Scope: the src/ module name (e.g., editor, state, events, db, result, explorer)
   All messages in English.

4. **Commit**
   ```bash
   git commit -m "<message>"
   ```
