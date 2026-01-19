# What is Chain Forge?

Chain Forge is a Foundry-inspired multi-chain development tool suite designed to simplify local blockchain development and testing. It provides a unified interface for working with different blockchain networks, starting with Solana support.

## The Problem

Blockchain developers often face these challenges:

- **Inconsistent tooling** across different blockchain ecosystems
- **Complex setup** for local development environments
- **Manual account management** and funding
- **Different CLIs and APIs** for each chain
- **Difficulty integrating** into automated testing pipelines

## The Solution

Chain Forge provides:

- **Unified Interface**: One CLI and API for multiple blockchain networks
- **Instant Setup**: Start a local validator with pre-funded accounts in seconds
- **Developer Experience**: Simple commands inspired by Ethereum's Foundry
- **Standard Wallets**: BIP39/BIP44 account generation compatible with popular wallets
- **Programmatic Access**: TypeScript SDK for seamless integration

## Core Features

### Multi-Chain Support

Built with a trait-based architecture that makes adding new blockchain networks straightforward. Currently supports:

- ✅ **Solana** - Full support with CLI and TypeScript package
- 🔜 **Bitcoin** - Planned
- 🔜 **Ethereum** - Planned

### Account Management

- Generate accounts using standard BIP39 mnemonics
- BIP44 derivation paths for wallet compatibility
- Pre-fund accounts automatically on startup
- Fund additional accounts via CLI or API

### CLI Tools

Each supported chain gets its own CLI tool:

```bash
# Solana
cf-solana start --accounts 10 --balance 100
cf-solana accounts
cf-solana fund <address> 50
```

### TypeScript Packages

Programmatic access via NPM packages:

```typescript
import { SolanaClient } from '@chain-forge/solana';

const client = new SolanaClient({ accounts: 10 });
await client.start();
const accounts = await client.getAccounts();
```

### Configuration System

Flexible TOML-based configuration with:

- Project-level configuration
- Global user defaults
- Multiple profiles (default, ci, devnet, etc.)

## Architecture

Chain Forge uses a `ChainProvider` trait as the core abstraction:

```rust
pub trait ChainProvider {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn get_accounts(&self) -> Result<Vec<Account>>;
    async fn set_balance(&self, address: &str, amount: f64) -> Result<()>;
    // ... more methods
}
```

Each blockchain implementation:

1. Implements the `ChainProvider` trait
2. Lives in `chains/<chain-name>/`
3. Provides a CLI binary (`cf-<chain>`)
4. Exposes a TypeScript package (`@chain-forge/<chain>`)

## Use Cases

### Smart Contract Development

Test your programs locally before deploying to testnet or mainnet:

```bash
cf-solana start --accounts 5 --balance 100
# Deploy and test your Solana program
anchor deploy
anchor test
```

### dApp Development

Build frontends against a local blockchain with controlled state:

```typescript
const client = new SolanaClient();
await client.start();

// Connect your dApp to localhost:8899
const connection = client.getConnection();
```

### Automated Testing

Integrate into CI/CD pipelines for reproducible test environments:

```yaml
- name: Start validator
  run: cf-solana start --accounts 3 --balance 10 &

- name: Run integration tests
  run: npm test
```

### Learning & Experimentation

Experiment with blockchain development without spending real tokens:

```bash
# Try different scenarios
cf-solana start --accounts 20 --balance 1000
# Make mistakes, restart fresh anytime
```

## Why "Chain Forge"?

Inspired by Ethereum's [Foundry](https://github.com/foundry-rs/foundry), Chain Forge aims to bring the same level of developer experience and tooling quality to multi-chain development.

The name reflects the project's goal: to be a **forge** for building blockchain applications across any **chain**.

## Design Principles

1. **Developer First**: Optimize for developer productivity and happiness
2. **Simple Defaults**: Works out of the box with sensible defaults
3. **Flexible Configuration**: Easy to customize when needed
4. **Standards Compliant**: Use standard protocols (BIP39/44) for compatibility
5. **Wraps Official Tools**: Leverage official blockchain tooling rather than reimplementing

## Next Steps

- [Installation Guide](./installation)
- [Getting Started](./getting-started)
- [Solana Documentation](../solana/overview)
