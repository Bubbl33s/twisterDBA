# twisterDBA Makefile
# Rust DX task runner

.PHONY: help check fix build test release audit deny coverage clean githooks install-tools update-tools

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
	awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-18s\033[0m %s\n", $$1, $$2}'

check: ## Full pipeline: fmt + clippy + build + test
	@echo "=== twisterDBA DX Pipeline ==="
	@echo ""
	@echo "--- cargo fmt --check ---"
	@cargo fmt --check
	@echo ""
	@echo "--- cargo clippy ---"
	@cargo clippy -- -D warnings
	@echo ""
	@echo "--- cargo build ---"
	@cargo build
	@echo ""
	@echo "--- cargo test ---"
	@cargo nextest run 2>/dev/null || cargo test
	@echo ""
	@echo "All checks passed."

fix: ## Auto-fix formatting and clippy issues
	@echo "--- cargo fmt ---"
	@cargo fmt
	@echo "--- cargo clippy --fix ---"
	@cargo clippy --fix --allow-dirty --allow-staged
	@echo ""
	@echo "Fixes applied. Run 'make check' to verify."

build: ## Debug build
	@cargo build

release: ## Release build (optimized)
	@cargo build --release

test: ## Run tests with nextest (or fallback to cargo test)
	@cargo nextest run 2>/dev/null || cargo test

audit: ## Run cargo-audit vulnerability scanner
	@cargo audit --version >/dev/null 2>&1 || { echo "cargo-audit not installed. Run: make install-tools"; exit 1; }
	@cargo audit

deny: ## Run cargo-deny license and dependency checks
	@cargo deny --version >/dev/null 2>&1 || { echo "cargo-deny not installed. Run: make install-tools"; exit 1; }
	@cargo deny check

coverage: ## Generate HTML coverage report with cargo-llvm-cov
	@cargo llvm-cov --version >/dev/null 2>&1 || { echo "cargo-llvm-cov not installed. Run: make install-tools"; exit 1; }
	@cargo llvm-cov --html --open 2>/dev/null || cargo llvm-cov --html
	@echo "Coverage report generated."

clean: ## Clean build artifacts
	@cargo clean
	@echo "Build artifacts cleaned."

githooks: ## Activate .githooks/ directory for git
	@git config core.hooksPath .githooks
	@echo "Git hooks activated: .githooks/pre-commit, .githooks/commit-msg"

install-tools: ## Install all DX tools via cargo
	@echo "Installing DX tools..."
	@cargo install cargo-audit 2>/dev/null || echo "cargo-audit already installed"
	@cargo install cargo-deny 2>/dev/null || echo "cargo-deny already installed"
	@cargo install cargo-nextest 2>/dev/null || echo "cargo-nextest already installed"
	@cargo install cargo-llvm-cov 2>/dev/null || echo "cargo-llvm-cov already installed"
	@cargo install bacon 2>/dev/null || echo "bacon already installed"
	@cargo install git-cliff 2>/dev/null || echo "git-cliff already installed"
	@echo ""
	@echo "All tools installed."
	@echo "Run 'make githooks' to activate git hooks."

update-tools: ## Update all DX tools to latest
	@echo "Updating DX tools..."
	@cargo install --force cargo-audit cargo-deny cargo-nextest cargo-llvm-cov bacon git-cliff
	@echo "Done."
