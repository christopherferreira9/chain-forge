# Chain Forge Architecture & Core Dependencies

## Overview

Chain Forge is a multi-chain development tool that wraps blockchain test nodes and provides a unified developer experience. This document explains the core technologies, how they work, and architectural decisions.

---

## Core Technologies

### 1. Solana Test Validator

**Source**: Official Solana CLI Tools
**Repository**: https://github.com/solana-labs/solana
**Binary**: `solana-test-validator`

#### What It Is

`solana-test-validator` is a lightweight, single-node Solana cluster designed for local development. It's part of the official Solana CLI tools and provides a fully functional blockchain environment.

#### Installation

```bash
# Official installation
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Verify
solana-test-validator --version
```

#### Key Features

- **Single-node cluster**: Runs a complete Solana validator locally
- **Fast block times**: Optimized for development (400ms slots)
- **Account pre-loading**: Can load accounts with SOL at startup
- **Program deployment**: Supports custom program deployment
- **Ledger persistence**: Optional ledger storage between runs
- **JSON-RPC API**: Full RPC interface on localhost:8899 (default)

#### How Chain Forge Uses It

Chain Forge spawns `solana-test-validator` as a child process:

```rust
// Simplified view
let child = Command::new("solana-test-validator")
    .arg("--rpc-port").arg("8899")
    .arg("--quiet")
    .arg("--reset")
    .spawn()?;
```

**Process Lifecycle:**
1. Spawn validator process
2. Wait for RPC to be ready (health checks)
3. Fund accounts via RPC airdrops
4. Keep process alive
5. Kill process on shutdown

---

### 2. Solana RPC Client

**Crate**: `solana-client`
**Version**: 2.0.x
**Documentation**: https://docs.rs/solana-client

#### What It Is

Official Rust client for interacting with Solana's JSON-RPC API. Provides methods for:
- Querying account data
- Sending transactions
- Getting blockchain state
- Requesting airdrops (test networks only)

#### How Chain Forge Uses It

```rust
use solana_client::rpc_client::RpcClient;

let client = RpcClient::new_with_timeout_and_commitment(
    "http://localhost:8899",
    Duration::from_secs(30),
    CommitmentConfig::confirmed(),
);

// Request airdrop
let signature = client.request_airdrop(&pubkey, lamports)?;

// Get balance
let balance = client.get_balance(&pubkey)?;
```

**Current Approach:**
- Start validator
- Wait for readiness
- Use `request_airdrop()` to fund each account
- Add delays between requests to avoid rate limiting

---

### 3. BIP39/BIP44 Key Derivation

**Crates**:
- `bip39` (v2.0) - Mnemonic generation
- `ed25519-dalek` (v2.1) - Ed25519 signatures

#### What They Are

Standard libraries for deterministic key generation:

- **BIP39**: Mnemonic phrase generation (12-24 words)
- **BIP44**: Hierarchical derivation paths (m/44'/501'/index'/0')

#### How Chain Forge Uses Them

```rust
use bip39::{Mnemonic, Language};

// Generate mnemonic
let mnemonic = Mnemonic::generate(12)?;

// Derive key at path m/44'/501'/0'/0'
let seed = mnemonic.to_seed("");
let key = derive_key_from_path(&seed, "m/44'/501'/0'/0'")?;

// Create Solana keypair
let signing_key = SigningKey::from_bytes(&key);
let keypair = Keypair::from_bytes(&signing_key.to_keypair_bytes())?;
```

**Benefits:**
- Deterministic account generation
- Recovery from mnemonic phrase
- Compatible with Solana wallets (Phantom, Solflare, etc.)
- Industry standard (used by all major wallets)

---

### 4. Solana SDK

**Crate**: `solana-sdk`
**Version**: 2.0.x
**Documentation**: https://docs.rs/solana-sdk

#### What It Is

Core Solana types and utilities:
- `Keypair` - Ed25519 keypair type
- `Pubkey` - Public key type
- `Transaction` - Transaction builder
- `Instruction` - Instruction types
- Native token constants (`LAMPORTS_PER_SOL`)

#### How Chain Forge Uses It

```rust
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    native_token::LAMPORTS_PER_SOL,
};

// Create keypair
let keypair = Keypair::from_bytes(&secret_key)?;

// Get public key
let pubkey = keypair.pubkey();

// Convert SOL to lamports
let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
```

---

### 5. Bitcoin Core (regtest)

**Source**: Bitcoin Core
**Repository**: https://github.com/bitcoin/bitcoin
**Binary**: `bitcoind`

#### What It Is

Bitcoin Core in regtest mode provides a fully controlled local Bitcoin blockchain for development. Unlike testnet, regtest produces blocks on demand and starts with no pre-existing chain.

#### How Chain Forge Uses It

Chain Forge spawns `bitcoind` with regtest configuration:

```rust
let child = Command::new("bitcoind")
    .arg("-regtest")
    .arg("-rpcport=18443")
    .arg("-rpcuser=...")
    .arg("-rpcpassword=...")
    .arg("-daemon=0")
    .spawn()?;
```

**Process Lifecycle:**
1. Spawn bitcoind in regtest mode
2. Wait for RPC to be ready
3. Create a `chain-forge` wallet (descriptor wallet)
4. Mine initial blocks to a wallet address (generates spendable coinbase)
5. Import account descriptors (`wpkh(WIF)`) into wallet
6. Fund accounts via `sendtoaddress` from wallet balance
7. Mine a confirmation block
8. Keep process alive, kill on shutdown

#### Key Differences from Solana

- **Mining required**: Bitcoin needs blocks mined to confirm transactions and generate funds
- **Wallet-based**: Accounts are imported as descriptors into a wallet, not standalone
- **UTXO model**: No account balances; funds tracked as unspent transaction outputs
- **Descriptor import with `"timestamp": "now"`**: Wallet does not rescan for pre-import transactions

### 6. Bitcoin RPC Client

**Crate**: `bitcoincore-rpc`
**Version**: 0.19

#### How Chain Forge Uses It

```rust
use bitcoincore_rpc::{Auth, Client, RpcApi};

let client = Client::new(&wallet_url, Auth::UserPass(user, password))?;

// Mine blocks
client.generate_to_address(101, &address)?;

// Send BTC
client.send_to_address(&addr, amount, ...)?;

// Raw RPC calls for advanced operations
let txs: Vec<Value> = client.call("listtransactions", &[json!("*"), json!(100)])?;
```

### 7. REST API Server

**Crate**: `chain-forge-api-server`
**Binary**: `cf-api`
**Framework**: Axum + Tokio

#### What It Is

An HTTP REST API that provides unified access to all registered blockchain nodes. It powers the web dashboard and can be used for custom integrations.

#### Key Design Decisions

- **Axum framework**: Lightweight, async, tower-compatible
- **CORS permissive**: Allows all origins so the dashboard (different port) can communicate
- **Node registry**: Shared JSON-based registry tracks all running nodes across chains
- **Chain-agnostic endpoints**: Same URL patterns for Solana and Bitcoin; handlers branch on `ChainType`

#### Endpoints (11 total)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/nodes` | List all registered nodes |
| GET | `/api/v1/nodes/{id}` | Get node details |
| POST | `/api/v1/nodes` | Start node (returns CLI command) |
| DELETE | `/api/v1/nodes/{id}` | Mark node stopped |
| GET | `/api/v1/nodes/{id}/accounts` | Accounts with live balances |
| POST | `/api/v1/nodes/{id}/fund` | Fund an account |
| GET | `/api/v1/nodes/{id}/transactions` | Recent transactions |
| GET | `/api/v1/nodes/{id}/transactions/{sig}` | Transaction detail |
| POST | `/api/v1/health` | Health check all nodes |
| POST | `/api/v1/registry/cleanup` | Remove stopped nodes |

### 8. Web Dashboard

**Location**: `development/dashboard/`
**Tech Stack**: React 18, TypeScript, Vite, TailwindCSS, React Query

#### What It Is

A real-time web interface for monitoring and managing blockchain nodes. It communicates with the REST API server.

#### Architecture

- **Dev server**: Vite on port 5173, proxies `/api` requests to port 3001
- **State management**: React Query with auto-refresh (5s for nodes, 10s for transactions)
- **Pages**: Dashboard (node grid) and NodeDetail (accounts + transactions tabs)
- **Components**: NodeGrid, NodeCard, NewNodeForm, AccountsList, TransactionsList, NodeStatus
- **Chain-aware UI**: Displays SOL/BTC units, Signature/TxID labels, Slot/Block headers

---

## Account Funding Mechanisms

### Current Implementation: Post-Startup Airdrops

**How It Works:**

1. Generate accounts from mnemonic
2. Start `solana-test-validator`
3. Wait for validator to be ready
4. Call `request_airdrop()` for each account via RPC
5. Wait for confirmations

**Code Flow:**
```
AccountGenerator → Generate keys
       ↓
SolanaProvider.start() → Spawn validator
       ↓
wait_for_validator() → Poll until ready
       ↓
fund_accounts() → Airdrop to each account
       ↓
Accounts ready with balance
```

**Limitations:**
- Takes time (airdrops are async)
- Rate limiting (need delays between airdrops)
- Requires RPC calls
- Accounts aren't funded immediately

---

### Alternative: Pre-funded Accounts (Solana-Specific)

`solana-test-validator` supports the `--account` flag to pre-load accounts:

```bash
solana-test-validator \
  --account <PUBKEY> <PATH_TO_ACCOUNT_DATA> \
  --account <PUBKEY2> <PATH_TO_ACCOUNT_DATA2>
```

**How It Works:**
- Requires serialized account data files
- Loads accounts into genesis block
- Accounts exist from block 0
- No RPC calls needed

**Challenges:**
- Need to create account data files
- More complex setup
- Files need to persist between runs

---

### Foundry/Anvil Comparison

#### Foundry (Ethereum)

```rust
// Anvil spawns with pre-funded accounts
let anvil = Anvil::new()
    .mnemonic("test test test...")
    .accounts(10)
    .balance(100) // Each account starts with 100 ETH
    .spawn();

// Accounts are IMMEDIATELY available with balance
```

**How Anvil Does It:**
- Genesis block pre-allocates balances
- No funding transactions needed
- Instant availability

#### Current Chain Forge (Solana)

```rust
let mut provider = SolanaProvider::with_config(config);
provider.start(config)?; // Spawns validator + funds accounts

// Accounts available after startup completes
let accounts = provider.get_accounts()?;
```

**Difference:**
- We fund accounts AFTER genesis
- Uses RPC airdrops
- Slight delay for funding

---

## Solana Funding Mechanisms Explained

### 1. Airdrops (What We Use)

**On Test Networks:**
```rust
// Request airdrop
client.request_airdrop(&pubkey, lamports)?;
```

- Available on: localnet, devnet, testnet
- NOT available on: mainnet
- Rate limited on public networks
- Instant on local validator

**Why We Use This:**
- Simple to implement
- Works for all accounts
- No file management
- Standard practice for development

### 2. Genesis Accounts (Alternative)

**During Validator Startup:**
- Accounts can be pre-loaded into genesis
- Requires account data files
- More complex but instant

**Why We Don't Use This (Yet):**
- More complex implementation
- Requires file management
- Non-standard for most developers
- Airdrops work fine for local development

### 3. Account Cloning (Advanced)

**For Testing Against Mainnet State:**
```bash
solana-test-validator --clone <MAINNET_ACCOUNT>
```

- Clones existing mainnet accounts
- Useful for testing with real data
- Future enhancement for Chain Forge

---

## Architecture Decisions

### 1. Why Wrap the Binary Instead of Embedding?

**Decision:** Spawn `solana-test-validator` as subprocess

**Rationale:**
- Official, well-tested validator
- Always up-to-date with Solana changes
- Reduces our maintenance burden
- Avoids version conflicts
- Standard practice (like Foundry wraps `anvil`)

**Trade-offs:**
- Requires Solana CLI installation
- Process management complexity
- Platform-specific considerations

### 2. Why BIP39/BIP44 for Keys?

**Decision:** Use standard mnemonic + derivation paths

**Rationale:**
- Compatible with all Solana wallets
- Deterministic and recoverable
- Industry standard
- Users already familiar with mnemonics

**Trade-offs:**
- More complex than random keys
- Requires crypto dependencies

### 3. Why Post-Startup Airdrops?

**Decision:** Fund accounts after validator starts

**Rationale:**
- Simple and reliable
- Standard practice in Solana ecosystem
- No file management needed
- Works for any number of accounts

**Trade-offs:**
- Slight startup delay
- Requires RPC calls
- Rate limiting considerations

---

## Dependency Overview

### Core Dependencies

| Dependency | Purpose | Why This One |
|------------|---------|--------------|
| `solana-client` | RPC communication | Official Solana client |
| `solana-sdk` | Core types | Official Solana SDK |
| `bip39` | Mnemonic generation | Standard BIP39 implementation |
| `ed25519-dalek` | Ed25519 signatures | Fast, widely-used library |
| `tokio` | Async runtime | Industry standard for Rust async |
| `clap` | CLI parsing | Best Rust CLI framework |
| `serde` | Serialization | Rust standard for ser/de |

### Why These Versions?

- **Solana 2.0.x**: Latest stable release
- **BIP39 2.0**: Modern, maintained
- **Tokio 1.x**: Current stable
- **Clap 4.x**: Latest with derive macros

---

## Comparison to Other Tools

### Foundry/Anvil (Ethereum)

**Similarities:**
- Wraps test node binary
- Mnemonic-based accounts
- Pre-funded accounts
- CLI + programmatic interface

**Differences:**
- Anvil: Genesis pre-funding (instant)
- Chain Forge: Post-startup airdrops (slight delay)
- Anvil: Single binary approach
- Chain Forge: Multi-chain architecture

### Solana Playground

**Similarities:**
- BIP39/BIP44 key derivation
- Solana test validator integration
- Account management

**Differences:**
- Playground: Web-based
- Chain Forge: CLI + local
- Playground: Wallet focus
- Chain Forge: Development tool focus

---

## Future Enhancements

### 1. Genesis Pre-funding

Implement `--account` flag usage for instant funding:

```rust
// Future implementation
fn start_with_prefunded_accounts(&self) -> Result<()> {
    // 1. Generate account data files
    // 2. Pass --account flags to validator
    // 3. Accounts ready immediately
}
```

**Benefits:**
- Instant account availability
- No RPC calls needed
- More like Foundry/Anvil

**Challenges:**
- File management
- Cleanup between runs
- More complex implementation

### 2. Account Cloning

Support cloning mainnet accounts:

```rust
config.clone_accounts = vec![
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token program
    "11111111111111111111111111111111", // System program
];
```

**Benefits:**
- Test against real state
- Integration testing
- Realistic scenarios

### 3. Snapshot/Restore

Save and restore blockchain state:

```rust
provider.save_snapshot("test-state")?;
// ... run tests ...
provider.restore_snapshot("test-state")?;
```

**Benefits:**
- Faster test iterations
- Deterministic testing
- State management

---

## Performance Characteristics

### Startup Time

**Current Implementation:**
1. Generate accounts: ~50ms
2. Start validator: ~2-3 seconds
3. Wait for ready: ~500ms
4. Fund accounts (10): ~1-2 seconds

**Total: ~4-6 seconds for 10 pre-funded accounts**

### With Genesis Pre-funding (Future)

1. Generate accounts: ~50ms
2. Create account files: ~100ms
3. Start validator: ~2-3 seconds
4. Wait for ready: ~500ms

**Total: ~3-4 seconds for 10 pre-funded accounts**

---

## Security Considerations

### Private Key Management

**Current Approach:**
- Keys stored in `~/.chain-forge/solana/accounts.json`
- File contains secret keys
- User responsible for securing file

**Recommendations:**
- ⚠️ Never commit this file
- ⚠️ Use only for local development
- ⚠️ Set restrictive permissions (0600)
- ⚠️ Different mnemonics per project

### Mnemonic Handling

**Display Once:**
```
🔑 Mnemonic: word1 word2 ... word12
   Save this mnemonic to recover your accounts!
```

**Security:**
- Displayed on first run
- Not stored (only derived keys stored)
- User responsible for backup
- Optional: can provide own mnemonic

---

## Platform Support

### Current Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS (x86_64) | ✅ Full | Primary development |
| macOS (ARM64) | ✅ Full | Apple Silicon |
| Linux (x86_64) | ✅ Full | CI tested |
| Windows (x86_64) | ⚠️ Limited | Path handling differences |

### Requirements

- **Rust**: 1.75+
- **Node.js**: 18+ (for TypeScript package)
- **Solana CLI**: Latest stable

---

## Summary

### What Chain Forge Actually Does

1. **Account Generation**
   - Uses BIP39 to create mnemonic
   - Derives keys via BIP44 (Solana: `m/44'/501'/n'/0'`, Bitcoin: `m/84'/1'/0'/0/n`)
   - Creates chain-specific keypairs
   - Saves to `~/.chain-forge/{chain}/instances/{id}/accounts.json`

2. **Node Management**
   - Spawns `solana-test-validator` or `bitcoind` as subprocess
   - Registers nodes in shared registry (`~/.chain-forge/registry.json`)
   - Monitors process health via RPC
   - Cleans up on exit

3. **Account Funding**
   - Solana: Airdrops via RPC after validator is ready
   - Bitcoin: Mines blocks for wallet funds, sends to each account, mines confirmation block

4. **User Interfaces**
   - **CLI**: `cf-solana` and `cf-bitcoin` binaries for direct usage
   - **TypeScript**: `@chain-forge/solana` NPM package for programmatic access
   - **REST API**: `cf-api` server exposing all operations over HTTP
   - **Web Dashboard**: React app for visual monitoring and management

### Key Technologies

1. **`solana-test-validator`** - Local Solana blockchain
2. **`bitcoind` (regtest)** - Local Bitcoin blockchain
3. **`solana-client`** / **`bitcoincore-rpc`** - RPC communication
4. **`bip39`** - Mnemonic generation
5. **`ed25519-dalek`** - Cryptographic operations (Solana + Bitcoin)
6. **`axum`** - REST API framework
7. **React + Vite** - Web dashboard
8. **`tokio`** - Async runtime

### Design Philosophy

> **Wrap, Don't Reimplement**

Chain Forge doesn't re-implement blockchain logic. Instead:
- Use official binaries (`solana-test-validator`, `bitcoind`)
- Use official SDKs (`solana-client`, `bitcoincore-rpc`)
- Use standard protocols (BIP39/44)
- Focus on developer experience layer (CLI, API, Dashboard)

This keeps the tool maintainable, reliable, and always compatible with the latest blockchain versions.

---

## References

- [Solana Documentation](https://docs.solana.com/)
- [Solana Test Validator Guide](https://docs.solana.com/developing/test-validator)
- [Bitcoin Core RPC Documentation](https://developer.bitcoin.org/reference/rpc/)
- [BIP39 Specification](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [BIP44 Specification](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
- [BIP84 Specification](https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki) (Native SegWit derivation)
- [Foundry Book](https://book.getfoundry.sh/)
- [Axum Web Framework](https://docs.rs/axum)

---

**Last Updated**: February 2026
**Version**: 0.1.0
