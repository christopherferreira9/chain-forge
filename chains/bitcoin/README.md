# Chain Forge - Bitcoin Support

Local Bitcoin development tools using regtest mode.

## Prerequisites

- Bitcoin Core (`bitcoind`) must be installed
- See [BITCOIN_DEPENDENCIES.md](../../docs/BITCOIN_DEPENDENCIES.md) for installation instructions

## Quick Start

### CLI

```bash
# Build the CLI
cargo build -p chain-forge-bitcoin-cli --release

# Start a local regtest node with 5 pre-funded accounts
cf-bitcoin start --accounts 5 --balance 10

# In another terminal:

# List accounts
cf-bitcoin accounts

# Fund an address
cf-bitcoin fund <address> 5

# Mine blocks
cf-bitcoin mine --blocks 10

# Stop the node (Ctrl+C in the start terminal)
```

### TypeScript

```typescript
import { BitcoinClient } from '@chain-forge/bitcoin';

const client = new BitcoinClient({
  accounts: 5,
  initialBalance: 10,
});

await client.start();

const accounts = await client.getAccounts();
console.log(accounts);

// Send BTC
await client.sendToAddress(accounts[1].address, 1);

// Mine a block
await client.mine(1);

client.stop();
```

## CLI Commands

### `cf-bitcoin start`

Start a local Bitcoin regtest node with pre-funded accounts.

```bash
cf-bitcoin start [OPTIONS]

Options:
  -a, --accounts <N>       Number of accounts to generate [default: 10]
  -b, --balance <BTC>      Initial balance per account [default: 10.0]
      --rpc-port <PORT>    RPC port [default: 18443]
      --p2p-port <PORT>    P2P port [default: 18444]
  -m, --mnemonic <PHRASE>  Mnemonic for deterministic accounts
      --rpc-user <USER>    RPC username [default: chainforge]
      --rpc-password <PW>  RPC password [default: chainforge]
```

### `cf-bitcoin accounts`

List all generated accounts with their balances.

```bash
cf-bitcoin accounts [OPTIONS]

Options:
  -f, --format <FORMAT>  Output format: table, json [default: table]
```

### `cf-bitcoin fund`

Send BTC to an address.

```bash
cf-bitcoin fund <ADDRESS> <AMOUNT>
```

### `cf-bitcoin mine`

Mine blocks (regtest only).

```bash
cf-bitcoin mine [OPTIONS]

Options:
  -b, --blocks <N>       Number of blocks to mine [default: 1]
  -a, --address <ADDR>   Address to receive coinbase [default: account 0]
```

### `cf-bitcoin config`

Show current configuration.

## Architecture

The Bitcoin implementation follows the same structure as Solana:

```
chains/bitcoin/crates/
├── accounts/    # BIP39/BIP44 key derivation (secp256k1)
├── rpc/         # Bitcoin Core JSON-RPC client wrapper
├── core/        # ChainProvider implementation
└── cli/         # cf-bitcoin CLI binary
```

### Key Differences from Solana

1. **Cryptography**: Uses secp256k1 (vs ed25519)
2. **Key derivation**: BIP44 path `m/44'/0'/0'/0/{index}` (coin type 0)
3. **Address format**: P2WPKH bech32 (`bcrt1...` for regtest)
4. **Funding**: Via mining + transactions (vs airdrops)
5. **Block generation**: Must explicitly mine blocks

## Configuration

### Default Configuration

```toml
[bitcoin.default]
rpc_url = "http://localhost:18443"
accounts = 10
initial_balance = 10.0
rpc_port = 18443
p2p_port = 18444
rpc_user = "chainforge"
rpc_password = "chainforge"
```

### Storage

- Accounts: `~/.chain-forge/bitcoin/accounts.json`
- Regtest data: `~/.chain-forge/bitcoin/regtest-data/`

## Development

```bash
# Build
cargo build -p chain-forge-bitcoin-cli

# Test
cargo test -p chain-forge-bitcoin-accounts
cargo test -p chain-forge-bitcoin-rpc
cargo test -p chain-forge-bitcoin-core

# Run with logging
RUST_LOG=debug cargo run -p chain-forge-bitcoin-cli -- start
```
