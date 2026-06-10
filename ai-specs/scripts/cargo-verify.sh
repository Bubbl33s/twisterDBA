#!/bin/bash
# cargo-verify.sh — Full Rust verification pipeline for twisterDBA
# Usage: bash ai-specs/scripts/cargo-verify.sh
set -euo pipefail

echo "=== twisterDBA Verification Pipeline ==="

echo ""
echo "--- cargo fmt --check ---"
if cargo fmt --check; then
    echo "✓ formatting OK"
else
    echo "✗ formatting issues — run: cargo fmt"
    exit 1
fi

echo ""
echo "--- cargo clippy ---"
if cargo clippy -- -D warnings 2>&1; then
    echo "✓ clippy OK"
else
    echo "✗ clippy warnings — fix before proceeding"
    exit 1
fi

echo ""
echo "--- cargo build ---"
if cargo build 2>&1; then
    echo "✓ build OK"
else
    echo "✗ build failed"
    exit 1
fi

echo ""
echo "--- cargo test ---"
if cargo test 2>&1; then
    echo "✓ all tests pass"
else
    echo "✗ test failures"
    exit 1
fi

echo ""
echo "=== All checks passed ==="
