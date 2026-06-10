---
description: Validate implementation matches change artifacts
---

Verify that implementation matches change artifacts.

**Input**: Optionally a change name.

**Steps**

1. **Select the change**
2. **Check completeness, correctness, coherence**
3. **Search codebase** for implementation evidence
4. **Run cargo build + clippy + test**
5. **Report issues** as CRITICAL, WARNING, SUGGESTION
6. **Show readiness** for archive

**Guardrails**
- Verify against specs/ and design.md
- Cargo build, clippy, test must pass
- Does not block archive but surfaces issues
