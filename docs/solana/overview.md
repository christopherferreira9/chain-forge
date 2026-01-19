# Solana Support

Chain Forge provides comprehensive Solana support through both CLI and TypeScript interfaces.

## Overview

The Solana implementation wraps `solana-test-validator` with additional features:

- BIP39/BIP44 account generation
- Automatic account funding
- Unified configuration
- Process lifecycle management
- RPC client wrapper

## Architecture

```
chains/solana/crates/
├── accounts/     BIP39/BIP44 key derivation
├── rpc/          Solana RPC client wrapper
├── core/         ChainProvider implementation
└── cli/          cf-solana binary
```

### Key Components

**Accounts Crate** (`chain-forge-solana-accounts`)
- Generates BIP39 mnemonics
- Derives accounts using BIP44 path `m/44'/501'/index'/0'`
- Compatible with popular Solana wallets

**RPC Crate** (`chain-forge-solana-rpc`)
- Wraps `solana-client` for RPC operations
- Provides high-level methods for common operations
- Handles airdrop rate limiting

**Core Crate** (`chain-forge-solana-core`)
- Implements the `ChainProvider` trait
- Manages validator process lifecycle
- Coordinates account generation and funding

**CLI Crate** (`chain-forge-solana-cli`)
- Provides the `cf-solana` command-line tool
- Subcommands: start, accounts, fund, config

## How It Works

### 1. Validator Startup

When you run `cf-solana start`:

1. Generate or load a BIP39 mnemonic
2. Derive accounts using BIP44 paths
3. Spawn `solana-test-validator` process
4. Wait for RPC endpoint to be ready
5. Fund accounts via airdrop
6. Display account information

### 2. Account Derivation

Accounts use standard derivation paths:

```
m/44'/501'/0'/0'  -> Account 0
m/44'/501'/1'/0'  -> Account 1
m/44'/501'/2'/0'  -> Account 2
...
```

This ensures compatibility with wallets like Phantom, Solflare, and Ledger.

### 3. Account Funding

Accounts are funded after validator startup:

- Uses `requestAirdrop` RPC call
- Includes delays between requests (rate limiting)
- Can't reduce balances (Solana limitation)

### 4. Storage

Configuration and accounts are stored in:

```
~/.chain-forge/
├── solana/
│   ├── accounts.json      # Account keys (private!)
│   └── validator.log      # Validator logs
└── config.toml            # Global configuration
```

::: danger Security Warning
`accounts.json` contains private keys. Never commit this file or share it publicly!
:::

## Configuration

See the [Configuration Guide](./configuration) for detailed options.

### Basic Configuration

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899
```

### Advanced Options

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 20
initial_balance = 1000.0
port = 8899

# Optional: custom validator arguments
validator_args = ["--reset", "--quiet"]
```

## CLI Reference

See the [CLI Commands Guide](./cli) for complete reference.

### Quick Reference

```bash
# Start validator
cf-solana start [OPTIONS]

# List accounts
cf-solana accounts [OPTIONS]

# Fund account
cf-solana fund <ADDRESS> <AMOUNT>

# Show config
cf-solana config
```

## TypeScript Package

The `@chain-forge/solana` package provides programmatic access:

```typescript
import { SolanaClient } from '@chain-forge/solana';

const client = new SolanaClient({
  accounts: 10,
  initialBalance: 100,
});

await client.start();
```

See the [TypeScript Guide](../typescript/basic-usage) for details.

## Comparison with solana-test-validator

### Raw solana-test-validator

```bash
# Start validator
solana-test-validator

# Generate account (separate tool)
solana-keygen new -o keypair.json

# Get airdrop
solana airdrop 100 <ADDRESS> --url http://localhost:8899

# Manual management
```

### With Chain Forge

```bash
# Start with 10 pre-funded accounts
cf-solana start --accounts 10 --balance 100

# Everything ready to use immediately
# Accounts saved and managed automatically
```

### Benefits

- **Faster setup**: One command vs multiple
- **Reproducible**: Configuration files and fixed mnemonics
- **Integrated**: Works with TypeScript, CI/CD, testing frameworks
- **Standards**: BIP39/BIP44 wallet compatibility

## Limitations

### Current Limitations

1. **Can't reduce balances**: Solana doesn't support removing SOL from accounts via RPC
2. **Single validator per port**: Can't run multiple validators on same port
3. **Airdrop delays**: Rate limiting requires ~1-2 seconds between airdrops
4. **Local only**: Test validator is for local development, not a full node

### Future Enhancements

- [ ] Support for custom Solana programs/accounts at genesis
- [ ] Snapshot/restore validator state
- [ ] Programmatic log access
- [ ] Network simulation (latency, packet loss)

## Integration with Anchor

Chain Forge works great with Anchor:

```bash
# Start Chain Forge validator
cf-solana start

# Deploy Anchor program (in another terminal)
anchor build
anchor deploy

# Run tests against Chain Forge validator
anchor test --skip-local-validator
```

Configure Anchor to use Chain Forge:

```toml
# Anchor.toml
[provider]
cluster = "localnet"
wallet = "~/.chain-forge/solana/accounts.json"

[test.validator]
url = "http://localhost:8899"
```

## Integration with Solana CLI

Use Solana CLI tools with Chain Forge:

```bash
# Start Chain Forge validator
cf-solana start

# Use Solana CLI (in another terminal)
solana config set --url http://localhost:8899
solana balance <ADDRESS>
solana transfer <TO> <AMOUNT> --from <FROM>
```

## Troubleshooting

### Validator Won't Start

Check if port is already in use:

```bash
lsof -i :8899
```

Try a different port:

```bash
cf-solana start --port 8900
```

### Airdrop Failures

Airdrops can fail due to rate limiting. Chain Forge includes built-in delays, but if you see failures:

- Wait a few seconds and try again
- Reduce the number of accounts
- Use smaller initial balances

### Slow Startup

First startup may be slow due to:

- Account funding (1-2 seconds per account)
- Validator initialization
- Solana CLI downloading updates

Subsequent starts are faster as Solana caches data.

## Next Steps

- [CLI Commands](./cli) - Complete CLI reference
- [Configuration](./configuration) - Configuration options
- [Account Management](./accounts) - Working with accounts
- [TypeScript Usage](../typescript/basic-usage) - Programmatic access
