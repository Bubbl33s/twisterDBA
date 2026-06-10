---
name: openspec-bulk-archive-change
description: Archive multiple completed changes at once for twisterDBA. Handles spec conflicts between changes.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.4.1"
---

Archive multiple completed changes at once.

**Input**: Optional list of change names.

**Steps**

1. **List completed changes** via `openspec list`
2. **Validate each change** before archiving
3. **Detect spec conflicts** across changes touching the same domains
4. **Resolve conflicts** by checking what's actually implemented
5. **Archive in chronological order**
6. **Show summary** of archived changes

**Guardrails**
- Warn if tasks are incomplete
- Resolve spec conflicts before archiving
- Archival order: oldest first
