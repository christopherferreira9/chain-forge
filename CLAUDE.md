# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Chain Forge is a Foundry-inspired multi-chain development tool suite for local blockchain development. Currently supports Solana with a unified interface for managing test validators, accounts, and development workflows. Built with Rust (backend/CLI) and TypeScript (NPM package).

## Key Commands

### Building

```bash
# Build entire workspace
cargo build --workspace

# Build in release mode
cargo build --workspace --release

# Build specific crate
cargo build -p chain-forge-solana-cli
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with all features
cargo test --workspace --all-features

# Run tests without default features
cargo test --workspace --no-default-features

# Run specific crate tests
cargo test -p chain-forge-solana-core

# Run tests with output visible
cargo test --workspace -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy (must pass with no warnings)
cargo clippy --workspace --all-features -- -D warnings
```

### Documentation

```bash
# Build and open docs
cargo doc --workspace --no-deps --open

# Build docs (CI mode - fails on warnings)
cargo doc --workspace --all-features --no-deps
# With RUSTDOCFLAGS=-D warnings
```

### Installing CLI Locally

```bash
# Install from local path
cargo install --path chains/solana/crates/cli

# Verify installation
cf-solana --version
```

### TypeScript Package

```bash
# Build TypeScript package
cd npm/@chain-forge/solana
yarn install
yarn build

# Type check only
yarn tsc --noEmit
```

### Examples

```bash
# Run the simple demo
cd examples/simple-demo
yarn install
yarn start

# Run the TypeScript basic example
cd examples/typescript-basic
yarn install
yarn build
node dist/index.js
```

## Architecture

### Multi-Chain Abstraction

The codebase is designed for multi-chain support through a trait-based architecture:

1. **`ChainProvider` trait** (`crates/common/src/chain.rs`): Core abstraction that all blockchain implementations must implement. Defines methods for:
   - Starting/stopping validators
   - Account management
   - Balance operations (`set_balance` is preferred over deprecated `fund_account`)
   - RPC access

2. **Chain-specific implementations**: Each chain lives in `chains/<chain-name>/` with its own crates for accounts, RPC, core logic, and CLI.

3. **Shared utilities**: `crates/common`, `crates/config`, and `crates/cli-utils` provide shared infrastructure.

### Solana Implementation Structure

```
chains/solana/crates/
├── accounts/    # BIP39/BIP44 key derivation and account generation
├── rpc/         # Wrapper around solana-client for RPC operations
├── core/        # ChainProvider implementation, validator lifecycle
└── cli/         # cf-solana CLI binary
```

**Key architectural decisions:**

- **Wraps `solana-test-validator`**: Spawns official Solana binary as subprocess rather than reimplementing. Reduces maintenance and ensures compatibility.
- **BIP39/BIP44 derivation**: Uses standard mnemonic + derivation paths (`m/44'/501'/index'/0'`) for wallet compatibility.
- **Post-startup funding**: Accounts are funded via RPC airdrops after validator starts (not genesis pre-funding). Simple but adds slight startup delay.

### Account Funding: `set_balance` vs `fund_account`

- **`set_balance(address, amount)`**: Preferred method. Adjusts balance to match target amount. Idempotent behavior where possible.
- **`fund_account(address, amount)`**: Deprecated. Adds funds to existing balance (non-idempotent).

For Solana: `set_balance` calculates difference between current and target, then airdrops the difference. Cannot reduce balances (Solana limitation).

### Configuration System

TOML-based configuration with multiple profiles:

- Project config: `chain-forge.toml` in project root
- Global config: `~/.chain-forge/config.toml`
- Example: `chain-forge.toml.example`

Profiles support different environments (e.g., `[solana.default]`, `[solana.devnet]`).

### Storage Locations

- Accounts: `~/.chain-forge/solana/accounts.json` (contains secret keys - never commit)
- Config: `~/.chain-forge/config.toml`
- Logs: `~/.chain-forge/solana/validator.log`

## Development Workflow

### Adding New Solana Features

1. Identify which crate:
   - Account/key operations → `chains/solana/crates/accounts`
   - RPC/network calls → `chains/solana/crates/rpc`
   - Provider/lifecycle logic → `chains/solana/crates/core`
   - CLI commands → `chains/solana/crates/cli`

2. Update `ChainProvider` trait if adding cross-chain capability

3. Add tests in the same crate

4. Update TypeScript wrapper if exposing to NPM package

### Adding a New Chain (e.g., Bitcoin)

1. Create directory structure:
   ```
   chains/bitcoin/crates/{cli,core,accounts,rpc}
   ```

2. Implement `ChainProvider` trait in core crate

3. Add to workspace in root `Cargo.toml`

4. Create CLI binary (e.g., `cf-bitcoin`)

5. Add TypeScript package at `npm/@chain-forge/bitcoin`

### Code Style

- Follow Rust API Guidelines
- Use `snake_case` for functions/variables, `PascalCase` for types
- Document public APIs with `///` doc comments
- Keep functions focused and small
- TypeScript: `camelCase` for functions, `PascalCase` for classes

### Error Handling

- Use `ChainError` from `crates/common/src/error.rs`
- Wrap chain-specific errors with context
- Return `Result<T>` for fallible operations

## Common Gotchas

1. **`solana-test-validator` must be installed**: CLI requires Solana CLI tools in PATH. Installation: `sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`

2. **Port 8899 conflicts**: Default Solana RPC port. Check for existing processes with `lsof -i :8899`

3. **Account funding delays**: Post-startup airdrops take 1-2 seconds per account. Rate limiting requires delays between requests.

4. **Windows path handling**: Be cautious with path separators and process spawning on Windows.

5. **Yarn 4 required**: TypeScript package uses Yarn 4 (via Corepack). Enable with `corepack enable`.

6. **OpenSSL vendored**: Uses `features = ["vendored"]` for Windows compatibility.

## CI/CD

GitHub Actions workflows in `.github/workflows/`:

- **ci.yml**: Runs tests, formatting, clippy, builds on Ubuntu/macOS/Windows
- **typescript.yml**: Builds TypeScript package and examples
- **security.yml**: Security audits
- **release.yml**: Release automation

All CI checks must pass. Clippy runs with `-D warnings` (treats warnings as errors).

## Dependencies

### Critical Rust Dependencies

- `solana-sdk`, `solana-client` (2.0): Official Solana libraries
- `bip39` (2.0): Mnemonic generation
- `ed25519-dalek` (2.1): Cryptographic operations
- `tokio` (1.x): Async runtime
- `clap` (4.x): CLI parsing

### TypeScript

- Peer dependency: `@solana/web3.js` (^1.87.0)
- Uses TypeScript 5.3+

## Testing Solana Locally

```bash
# Terminal 1: Start validator
cargo run -p chain-forge-solana-cli -- start --accounts 5 --balance 100

# Terminal 2: Query accounts
cargo run -p chain-forge-solana-cli -- accounts

# Terminal 2: Fund account
cargo run -p chain-forge-solana-cli -- fund <ADDRESS> 50

# Terminal 1: Stop with Ctrl+C
```

## Project History

This is a ground-up implementation inspired by Foundry (Ethereum), designed for multi-chain local development. V0.1.0 provides Solana MVP with full CLI and TypeScript support. Future chains (Bitcoin, Ethereum) will follow the same `ChainProvider` abstraction.

## Documentation

- Architecture details: `.claude/ARCHITECTURE.md`
- Testing guide: `docs/TESTING.md`
- Contributing: `docs/CONTRIBUTING.md`
- Implementation notes: `.claude/IMPLEMENTATION_SUMMARY.md`
