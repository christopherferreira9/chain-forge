# Chain Forge Development Makefile

.PHONY: help install-tools check test build audit format lint check-all clean

help: ## Show this help message
	@echo "Chain Forge Development Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

install-tools: ## Install development tools (cargo-audit, etc.)
	@echo "Installing cargo-audit..."
	@cargo install cargo-audit --quiet || echo "cargo-audit already installed"
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
