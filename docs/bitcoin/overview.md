# Bitcoin Support

::: warning Development Status
Bitcoin support is currently in **active development**. While core functionality works, some features may be incomplete or change in future releases. Please report any issues on [GitHub](https://github.com/christopherferreira9/chain-forge/issues).
:::

Chain Forge provides comprehensive Bitcoin support through both CLI and TypeScript interfaces, using Bitcoin Core in regtest mode for local development.

## Overview

The Bitcoin implementation wraps `bitcoind` (Bitcoin Core) with additional features:

- BIP39/BIP44 account generation
- Automatic account funding via mining and transfers
- Instance isolation for multiple concurrent nodes
- UTXO-based balance tracking
- Real transaction creation and signing

## Architecture

```
chains/bitcoin/crates/
├── accounts/     BIP39/BIP44 key derivation
├── rpc/          Bitcoin Core RPC client wrapper
├── core/         ChainProvider implementation
└── cli/          cf-bitcoin binary
```

### Key Components

**Accounts Crate** (`chain-forge-bitcoin-accounts`)
- Generates BIP39 mnemonics
- Derives accounts using BIP44 path `m/44'/0'/0'/0/index`
- Creates P2WPKH (native SegWit bech32) addresses
- Compatible with popular Bitcoin wallets

**RPC Crate** (`chain-forge-bitcoin-rpc`)
- Wraps `bitcoincore-rpc` for RPC operations
- Provides high-level methods for common operations
- Handles UTXO queries via `scantxoutset`
- Creates and signs raw transactions

**Core Crate** (`chain-forge-bitcoin-core`)
- Implements the `ChainProvider` trait
- Manages bitcoind process lifecycle
- Coordinates account generation and funding
- Supports multiple isolated instances

**CLI Crate** (`chain-forge-bitcoin-cli`)
- Provides the `cf-bitcoin` command-line tool
- Subcommands: start, accounts, fund, transfer, mine, config, stop

## How It Works

### 1. Node Startup

When you run `cf-bitcoin start`:

1. Clear previous instance data (clean slate)
2. Generate or load a BIP39 mnemonic
3. Derive accounts using BIP44 paths
4. Spawn `bitcoind` in regtest mode
5. Create a wallet and mine initial blocks
6. Fund accounts from mining rewards
7. Import accounts into wallet for tracking
8. Display account information

### 2. Account Derivation

Accounts use standard Bitcoin derivation paths:

```
m/44'/0'/0'/0/0  -> Account 0
m/44'/0'/0'/0/1  -> Account 1
m/44'/0'/0'/0/2  -> Account 2
...
```

This ensures compatibility with wallets like Electrum, Ledger, and Trezor.

### 3. Account Funding

Accounts are funded after node startup:

- Mines initial blocks to generate spendable coinbase rewards
- Sends BTC from wallet to each account address
- Mines confirmation blocks
- Imports accounts into wallet for balance tracking

### 4. Instance Isolation

Each node instance has isolated state:

```
~/.chain-forge/bitcoin/instances/<instance-id>/
├── accounts.json      # Account keys and balances
├── instance.json      # RPC connection info
└── regtest-data/      # Blockchain data
```

This allows running multiple nodes with different configurations simultaneously.

::: danger Security Warning
`accounts.json` contains private keys. Never commit this file or share it publicly!
:::

## Configuration

See the [Configuration Guide](./configuration) for detailed options.

### Basic Configuration

```toml
[bitcoin.default]
rpc_url = "http://localhost:18443"
accounts = 10
initial_balance = 10.0
rpc_port = 18443
p2p_port = 18444
```

## CLI Reference

See the [CLI Commands Guide](./cli) for complete reference.

### Quick Reference

```bash
# Start node
cf-bitcoin start [OPTIONS]

# List accounts
cf-bitcoin accounts [OPTIONS]

# Fund account (from wallet)
cf-bitcoin fund <ADDRESS> <AMOUNT>

# Transfer between accounts
cf-bitcoin transfer <FROM> <TO> <AMOUNT>

# Mine blocks
cf-bitcoin mine [OPTIONS]

# Show config
cf-bitcoin config
```

## TypeScript Package

The `@chain-forge/bitcoin` package provides programmatic access:

```typescript
import { BitcoinClient } from '@chain-forge/bitcoin';

const client = new BitcoinClient({
  accounts: 10,
  initialBalance: 10,
});

await client.start();

// Get accounts
const accounts = await client.getAccounts();

// Transfer between accounts
await client.transfer(accounts[0].address, accounts[1].address, 1);

// Mine blocks
await client.mine(1);

// Refresh balances from blockchain
const updated = await client.refreshBalances();
```

See the [TypeScript Guide](../typescript/basic-usage) for details.

## Comparison with Raw bitcoind

### Raw bitcoind

```bash
# Start bitcoind
bitcoind -regtest -daemon

# Create wallet
bitcoin-cli -regtest createwallet "test"

# Generate address
bitcoin-cli -regtest getnewaddress

# Mine blocks
bitcoin-cli -regtest generatetoaddress 101 <address>

# Send BTC
bitcoin-cli -regtest sendtoaddress <address> 1.0

# Manual UTXO management, signing, etc.
```

### With Chain Forge

```bash
# Start with 10 pre-funded accounts
cf-bitcoin start --accounts 10 --balance 10

# Everything ready to use immediately
# Accounts saved and managed automatically

# Transfer between accounts
cf-bitcoin transfer <from> <to> 1.0
```

### Benefits

- **Faster setup**: One command vs multiple
- **Reproducible**: Configuration files and fixed mnemonics
- **Instance isolation**: Run multiple nodes simultaneously
- **Clean state**: Each start is a fresh blockchain
- **Integrated**: Works with TypeScript, CI/CD, testing frameworks
- **Standards**: BIP39/BIP44 wallet compatibility

## Key Concepts

### UTXO Model

Bitcoin uses Unspent Transaction Outputs (UTXOs), not account balances:

- Each transaction creates new UTXOs
- Spending requires consuming existing UTXOs
- "Balance" is the sum of all UTXOs for an address
- Chain Forge queries UTXOs via `scantxoutset` for accurate balances

### Transactions

When you transfer BTC:

1. Find UTXOs owned by source address (`listunspent`)
2. Create raw transaction with inputs and outputs
3. Sign with wallet (`signrawtransactionwithwallet`)
4. Broadcast (`sendrawtransaction`)
5. Mine block to confirm

Chain Forge handles all this automatically.

### Regtest Mode

Bitcoin Core's regtest mode provides:

- Instant block mining (no proof-of-work)
- 50 BTC block rewards (original Bitcoin amount)
- Isolated network (no connection to mainnet/testnet)
- Identical transaction format to mainnet

## Limitations

### Current Limitations

1. **Clean state on restart**: Blockchain data is cleared each start
2. **Single instance per port**: Can't run multiple nodes on same RPC port
3. **No persistent state**: Use different instances for different scenarios
4. **Local only**: Regtest is for local development, not a full node

### Future Enhancements

- [ ] Optional persistent blockchain state
- [ ] Multi-signature support
- [ ] Custom transaction types
- [ ] Network simulation (latency, mempool)

## Integration with Bitcoin Tools

Chain Forge works with standard Bitcoin tools:

```bash
# Start Chain Forge node
cf-bitcoin start

# Use bitcoin-cli (in another terminal)
bitcoin-cli -regtest -rpcuser=chainforge -rpcpassword=chainforge getblockchaininfo
bitcoin-cli -regtest -rpcuser=chainforge -rpcpassword=chainforge getbalance
```

## Troubleshooting

### Node Won't Start

Check if bitcoind is installed:

```bash
bitcoind --version
```

Check if port is already in use:

```bash
lsof -i :18443
```

Try a different port:

```bash
cf-bitcoin start --rpc-port 18445 --p2p-port 18446
```

### Accounts Have Zero Balance

If some accounts show 0 balance after startup:

- Check startup logs for funding errors
- Ensure wallet has sufficient mining rewards
- Try reducing number of accounts or balance

### Transfer Failures

If transfers fail with "No UTXOs found":

- Ensure source account has confirmed balance
- Mine a block after receiving funds
- Use `cf-bitcoin accounts` to verify balances

## Next Steps

- [CLI Commands](./cli) - Complete CLI reference
- [Configuration](./configuration) - Configuration options
- [Account Management](./accounts) - Working with accounts
- [TypeScript Usage](../typescript/basic-usage) - Programmatic access
