---
description: Archive multiple completed changes at once
---

Archive multiple completed changes at once.

**Input**: Optional list of change names.

**Steps**

1. **List completed** via `openspec list`
2. **Validate each** before archiving
3. **Detect spec conflicts** across changes
4. **Resolve conflicts** via codebase inspection
5. **Archive in chronological order**
6. **Show summary**

**Guardrails**
- Warn if tasks incomplete
- Resolve conflicts before archiving
