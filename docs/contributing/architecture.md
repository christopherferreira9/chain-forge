# Architecture

Chain Forge's technical architecture and design decisions.

## Core Abstraction

The `ChainProvider` trait defines the interface for all blockchain implementations:

```rust
pub trait ChainProvider {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn get_accounts(&self) -> Result<Vec<Account>>;
    async fn set_balance(&self, address: &str, amount: f64) -> Result<()>;
    // ...
}
```

## Project Structure

```
chain-forge/
├── chains/              # Chain implementations
│   └── solana/
│       └── crates/
│           ├── accounts/  # BIP39/BIP44 derivation
│           ├── rpc/       # RPC client wrapper
│           ├── core/      # ChainProvider impl
│           └── cli/       # cf-solana binary
├── crates/              # Shared utilities
│   ├── common/          # Shared traits
│   ├── config/          # Configuration
│   └── cli-utils/       # CLI helpers
└── npm/                 # TypeScript packages
```

## Solana Implementation

The Solana implementation wraps `solana-test-validator`:

1. **Process Management**: Spawns validator as subprocess
2. **Account Generation**: BIP39/BIP44 key derivation
3. **Funding**: Post-startup airdrops via RPC
4. **Lifecycle**: Clean process termination

## Adding New Chains

1. Create `chains/<chain>/crates/{cli,core,accounts,rpc}`
2. Implement `ChainProvider` trait
3. Add TypeScript package at `npm/@chain-forge/<chain>`

## See Also

- [Development Guide](./development)
- [CLAUDE.md](https://github.com/christopherferreira9/chain-forge/blob/main/CLAUDE.md) for detailed technical docs
