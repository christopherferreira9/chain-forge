# Chain Forge Development Makefile

.PHONY: help install-tools check test build audit format lint check-all clean release release-dry-run

help: ## Show this help message
	@echo "Chain Forge Development Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

install-tools: ## Install development tools (cargo-audit, git-cliff, etc.)
	@echo "Installing cargo-audit..."
	@cargo install cargo-audit --quiet || echo "cargo-audit already installed"
	@echo "Installing git-cliff..."
	@cargo install git-cliff --quiet || echo "git-cliff already installed"
	@echo "✓ Development tools installed"

check: ## Run format check, clippy, and tests
	@echo "Running format check..."
	@cargo fmt --all -- --check
	@echo "Running clippy..."
	@cargo clippy --workspace --all-features -- -D warnings
	@echo "Running tests..."
	@cargo test --workspace
	@echo "✓ All checks passed"

test: ## Run tests
	@cargo test --workspace

build: ## Build workspace in release mode
	@cargo build --workspace --release

audit: install-tools ## Run security audit with cargo-audit
	@echo "Running cargo audit..."
	@cargo audit
	@echo "✓ Security audit passed"

format: ## Format code
	@cargo fmt --all

lint: ## Run clippy
	@cargo clippy --workspace --all-features -- -D warnings

check-all: format lint test audit ## Run all checks (format, lint, test, audit)
	@echo "✓ All checks passed!"

clean: ## Clean build artifacts
	@cargo clean

# TypeScript commands
.PHONY: ts-build ts-test

ts-build: ## Build TypeScript package
	@cd npm/@chain-forge/solana && yarn install && yarn build

ts-test: ts-build ## Run TypeScript example
	@cd examples/simple-demo && yarn install && yarn start

# Release commands
release: ## Create a release branch with version bump and changelog
	@bash scripts/release.sh

release-dry-run: ## Show current version and release status
	@echo "Current workspace version: $$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')"
	@echo "NPM @chain-forge/solana:  $$(jq -r .version npm/@chain-forge/solana/package.json)"
	@echo "NPM @chain-forge/bitcoin: $$(jq -r .version npm/@chain-forge/bitcoin/package.json)"
	@echo ""
	@if [ -f .release.json ]; then echo "Release manifest:"; cat .release.json; else echo "No .release.json found (not on a release branch)"; fi
