# chain-forge-solana-accounts

BIP39/BIP44 account generation for Solana.

## Overview

Provides deterministic account generation using BIP39 mnemonics and BIP44 derivation paths. Follows Solana's standard derivation path: `m/44'/501'/index'/0'`.

## Features

- **BIP39 Mnemonics**: 12-word mnemonic phrase generation
- **BIP44 Derivation**: Solana-standard key derivation
- **Account Recovery**: Regenerate accounts from mnemonic
- **Storage**: Save/load accounts to disk
- **Ed25519 Keys**: Native Solana keypair format

## Usage

```toml
[dependencies]
chain-forge-solana-accounts = { path = "../accounts" }
```

### Generate Accounts

```rust
use chain_forge_solana_accounts::AccountGenerator;

// Generate new mnemonic and accounts
let generator = AccountGenerator::new()?;
let mnemonic = generator.mnemonic_phrase();
println!("Mnemonic: {}", mnemonic);

let accounts = generator.generate_accounts(10)?;
for (i, account) in accounts.iter().enumerate() {
    println!("Account {}: {}", i, account.public_key);
}
```

### Recover from Mnemonic

```rust
use chain_forge_solana_accounts::AccountGenerator;

let mnemonic = "your twelve word mnemonic phrase goes here like this example";
let generator = AccountGenerator::from_mnemonic(mnemonic)?;

// Will generate the same accounts every time
let accounts = generator.generate_accounts(10)?;
```

### Save/Load Accounts

```rust
use chain_forge_solana_accounts::AccountsStorage;
use std::path::Path;

let storage = AccountsStorage::new(Path::new("~/.chain-forge"));

// Save accounts
storage.save(&accounts)?;

// Load accounts
let loaded_accounts = storage.load()?;

// Check if file exists
if storage.exists() {
    println!("Accounts file exists");
}

// Delete accounts file
storage.delete()?;
```

## API Reference

### `AccountGenerator`

Generate accounts from BIP39 mnemonics.

**Methods:**

```rust
// Create with new random mnemonic
pub fn new() -> Result<Self>

// Create from existing mnemonic
pub fn from_mnemonic(phrase: &str) -> Result<Self>

// Get the mnemonic phrase
pub fn mnemonic_phrase(&self) -> String

// Generate multiple accounts
pub fn generate_accounts(&self, count: u32) -> Result<Vec<SolanaAccount>>

// Derive single account at index
pub fn derive_account(&self, index: u32) -> Result<SolanaAccount>
```

### `SolanaAccount`

Represents a Solana account with keypair and metadata.

**Fields:**

```rust
pub struct SolanaAccount {
    pub public_key: String,           // Base58 public key
    pub secret_key: Vec<u8>,          // 64-byte secret key
    pub mnemonic: Option<String>,     // Source mnemonic
    pub derivation_path: Option<String>, // BIP44 path
    pub balance: f64,                 // Balance in SOL
}
```

**Methods:**

```rust
// Get keypair from account
pub fn keypair(&self) -> Result<Keypair>

// Get address as string
pub fn address(&self) -> String
```

### `AccountsStorage`

Manage account persistence.

**Methods:**

```rust
// Create storage manager
pub fn new(data_dir: &Path) -> Self

// Save accounts to file
pub fn save(&self, accounts: &[SolanaAccount]) -> Result<()>

// Load accounts from file
pub fn load(&self) -> Result<Vec<SolanaAccount>>

// Check if file exists
pub fn exists(&self) -> bool

// Delete accounts file
pub fn delete(&self) -> Result<()>
```

## Key Derivation

Uses Solana's standard BIP44 derivation path:

```
m/44'/501'/index'/0'
```

Where:
- `44'` - BIP44 purpose
- `501'` - Solana coin type
- `index'` - Account index (0, 1, 2, ...)
- `0'` - Change index (always 0 for Solana)

All indices use hardened derivation (indicated by `'`).

## Security Considerations

⚠️ **Important Security Notes:**

1. **Mnemonic Protection**: Never commit mnemonics to version control
2. **Private Keys**: `secret_key` contains private key material
3. **File Permissions**: Accounts file should be readable only by owner
4. **Production Use**: Only use test mnemonics for development
5. **Backup**: Always backup mnemonic phrases securely

### Example Security Setup

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

// Set restrictive permissions on accounts file
let accounts_path = storage.accounts_file();
let mut perms = fs::metadata(accounts_path)?.permissions();
perms.set_mode(0o600); // Owner read/write only
fs::set_permissions(accounts_path, perms)?;
```

## Storage Format

Accounts are stored as JSON:

```json
[
  {
    "public_key": "7xJ5...",
    "secret_key": [1, 2, 3, ...],
    "mnemonic": "word1 word2 ...",
    "derivation_path": "m/44'/501'/0'/0'",
    "balance": 100.0
  }
]
```

Location: `~/.chain-forge/solana/accounts.json`

## Examples

### Generate and Display Accounts

```rust
use chain_forge_solana_accounts::{AccountGenerator, AccountsStorage};

let generator = AccountGenerator::new()?;
println!("Save this mnemonic: {}", generator.mnemonic_phrase());

let accounts = generator.generate_accounts(5)?;
for (i, account) in accounts.iter().enumerate() {
    println!("Account {}:", i);
    println!("  Address: {}", account.address());
    println!("  Path: {}", account.derivation_path.as_ref().unwrap());
}

// Save to disk
let storage = AccountsStorage::new(Path::new("./data"));
storage.save(&accounts)?;
```

### Recover and Use Account

```rust
use chain_forge_solana_accounts::AccountGenerator;

let mnemonic = "test test test test test test test test test test test junk";
let generator = AccountGenerator::from_mnemonic(mnemonic)?;

let account = generator.derive_account(0)?;
let keypair = account.keypair()?;

// Use keypair with Solana SDK
// let signature = keypair.sign_message(&message);
```

## Testing

```bash
cargo test -p chain-forge-solana-accounts
```

## License

MIT OR Apache-2.0
