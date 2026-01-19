# Contributing to Chain Forge

Thank you for your interest in contributing to Chain Forge! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful and inclusive. We want Chain Forge to be a welcoming project for everyone.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Node.js 18 or later
- Solana CLI tools (for testing)
- Git

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/chain-forge
cd chain-forge

# Install development tools (cargo-audit, etc.)
make install-tools

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace

# Install CLI locally
cargo install --path chains/solana/crates/cli
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Write clear, documented code
- Follow Rust conventions and idioms
- Add tests for new functionality
- Update documentation as needed

### 3. Run Tests

```bash
# Run all tests
cargo test --workspace

# Run specific package tests
cargo test -p chain-forge-solana-core

# Run with output
cargo test --workspace -- --nocapture
```

### 4. Format and Lint

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-features -- -D warnings

# Run security audit
cargo audit
```

Or use make for all checks:

```bash
make check-all
```

### 5. Commit Changes

Use clear, descriptive commit messages:

```bash
git commit -m "feat: add Bitcoin support"
git commit -m "fix: resolve account generation bug"
git commit -m "docs: update Solana README"
```

Commit message prefixes:
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Pull Request Guidelines

### PR Title

Use the same format as commit messages:
- `feat: add new feature`
- `fix: resolve issue #123`

### PR Description

Include:
1. **What**: What does this PR do?
2. **Why**: Why is this change needed?
3. **How**: How does it work?
4. **Testing**: How was it tested?
5. **Breaking Changes**: Any breaking changes?

Example:
```markdown
## What
Adds Bitcoin support to Chain Forge

## Why
Extends Chain Forge to support Bitcoin regtest networks

## How
- Implements ChainProvider trait for Bitcoin
- Adds Bitcoin RPC client wrapper
- Creates cf-bitcoin CLI tool

## Testing
- Added unit tests for all new modules
- Tested manually with Bitcoin Core regtest

## Breaking Changes
None
```

### PR Checklist

- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for significant changes)
- [ ] Tests added for new functionality

## Code Style

### Rust

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use descriptive names
- Document public APIs with `///` comments
- Keep functions focused and small

Example:
```rust
/// Generate multiple accounts from a mnemonic
///
/// # Arguments
///
/// * `count` - Number of accounts to generate
///
/// # Returns
///
/// Vector of generated accounts
///
/// # Errors
///
/// Returns `ChainError` if account generation fails
pub fn generate_accounts(&self, count: u32) -> Result<Vec<SolanaAccount>> {
    // Implementation
}
```

### TypeScript

- Use `camelCase` for functions and variables
- Use `PascalCase` for classes and interfaces
- Use TypeScript types, avoid `any`
- Document with JSDoc comments

Example:
```typescript
/**
 * Start the local Solana test validator
 *
 * @throws {Error} If validator is already running
 */
async start(): Promise<void> {
  // Implementation
}
```

## Testing

### Writing Tests

- Test both success and failure cases
- Use descriptive test names
- Keep tests focused and independent
- Mock external dependencies when possible

Example:
```rust
#[test]
fn test_account_generation_with_valid_mnemonic() {
    let mnemonic = "test test test test test test test test test test test junk";
    let generator = AccountGenerator::from_mnemonic(mnemonic).unwrap();
    let accounts = generator.generate_accounts(5).unwrap();
    assert_eq!(accounts.len(), 5);
}

#[test]
fn test_account_generation_with_invalid_mnemonic() {
    let result = AccountGenerator::from_mnemonic("invalid");
    assert!(result.is_err());
}
```

### Integration Tests

For integration tests that require running services:

```rust
#[test]
#[ignore] // Requires running validator
fn test_with_validator() {
    // Test that requires solana-test-validator
}
```

Run ignored tests with:
```bash
cargo test -- --ignored
```

## Adding New Chains

To add support for a new blockchain:

### 1. Create Chain Directory

```bash
mkdir -p chains/your-chain/crates/{cli,core,wallet,rpc}
```

### 2. Implement ChainProvider

```rust
use chain_forge_common::ChainProvider;

pub struct YourChainProvider {
    // Chain-specific state
}

impl ChainProvider for YourChainProvider {
    // Implement required methods
}
```

### 3. Create CLI

```rust
// chains/your-chain/crates/cli/src/main.rs
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Implement commands
```

### 4. Add to Workspace

Update root `Cargo.toml`:
```toml
members = [
    # ...
    "chains/your-chain/crates/cli",
    "chains/your-chain/crates/core",
]
```

### 5. Documentation

- Create `chains/your-chain/README.md`
- Add to main README.md
- Document all public APIs

## Documentation

### Rust Documentation

```bash
# Build docs
cargo doc --workspace --no-deps --open

# Check for broken links
cargo doc --workspace --no-deps
```

### README Files

- Each crate should have a README.md
- Include usage examples
- Document prerequisites
- Explain key concepts

## Release Process

(For maintainers)

1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.1.0`
4. Push tag: `git push origin v0.1.0`
5. GitHub Actions will build and publish

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/yourusername/chain-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/chain-forge/discussions)
- **Discord**: (Add Discord link if available)

## License

By contributing, you agree that your contributions will be dual-licensed under MIT OR Apache-2.0.

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes
- GitHub contributors page

Thank you for contributing to Chain Forge! 🎉
