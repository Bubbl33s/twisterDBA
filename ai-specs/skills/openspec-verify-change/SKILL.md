---
name: openspec-verify-change
description: Validate implementation against twisterDBA change artifacts. Checks completeness, correctness, and coherence.
license: MIT
compatibility: Requires openspec CLI.
metadata:
  author: openspec
  version: "1.0"
  generatedBy: "1.4.1"
---

Verify that implementation matches change artifacts for twisterDBA.

**Input**: Optionally a change name.

**Steps**

1. **Select the change**
2. **Check dimensions**:
   - **Completeness**: All tasks done, requirements implemented, scenarios covered
   - **Correctness**: Implementation matches spec intent, edge cases handled
   - **Coherence**: Design decisions reflected in Rust code, patterns consistent
3. **Search codebase** for implementation evidence
4. **Report issues** categorized as CRITICAL, WARNING, or SUGGESTION
5. **Run cargo build + clippy + test** to verify
6. **Show summary** with readiness for archive

**Guardrails**
- Verify against specs/ and design.md in the change
- cargo build, clippy, and test must pass
- Does not block archive but surfaces issues
