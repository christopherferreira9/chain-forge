# Development Guide

Guide for contributing to Chain Forge development.

## Prerequisites

- Rust 1.75+
- Node.js 18+
- Solana CLI tools
- Yarn 4 (via Corepack)

## Setup

```bash
git clone https://github.com/christopherferreira9/chain-forge
cd chain-forge
cargo build --workspace
```

## Building

```bash
# Build all crates
cargo build --workspace

# Build in release mode
cargo build --workspace --release

# Build specific crate
cargo build -p chain-forge-solana-cli
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture
```

## Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-features -- -D warnings
```

## See Also

- [Architecture](./architecture)
- [Testing Guide](./testing)
