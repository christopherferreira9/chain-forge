# Testing Guide

How to test Chain Forge.

## Running Tests

```bash
# All tests
cargo test --workspace

# With features
cargo test --workspace --all-features

# Specific crate
cargo test -p chain-forge-solana-core

# With output
cargo test --workspace -- --nocapture
```

## Test Categories

### Unit Tests
Located alongside code in `src/` files.

### Integration Tests
Located in `tests/` directories.

## CI/CD

Tests run on every PR via GitHub Actions:

- Format check
- Clippy lint
- Test suite
- Security audit

## See Also

- [Development Guide](./development)
- [CI/CD Workflow](/.github/workflows/ci.yml)
