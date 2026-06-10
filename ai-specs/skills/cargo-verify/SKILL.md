---
name: cargo-verify
description: Run full Rust verification suite — cargo build, clippy, fmt, and test. Use after any code change in twisterDBA.
author: twisterDBA
version: 1.0.0
---

# cargo-verify Skill

Run the complete Rust verification pipeline for twisterDBA.

## Instructions

Execute these commands in order. Stop on first failure.

1. **Format check**
   ```bash
   cargo fmt --check
   ```
   If files need formatting, run `cargo fmt` and re-check.

2. **Clippy lints**
   ```bash
   cargo clippy -- -D warnings
   ```
   Fix all warnings before proceeding.

3. **Build**
   ```bash
   cargo build
   ```
   Must compile without errors.

4. **Test**
   ```bash
   cargo test
   ```
   All tests must pass.

5. **Report**
   Output a summary: build status, clippy warnings (if any), test count (passed/failed).
