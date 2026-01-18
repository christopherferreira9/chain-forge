# chain-forge-solana-rpc

Solana RPC client wrapper for Chain Forge.

## Overview

Provides a simplified wrapper around `solana-client` for common operations like checking balances, requesting airdrops, and interacting with the Solana blockchain.

## Features

- **Balance Queries**: Get account balances in SOL
- **Airdrops**: Request test SOL from faucet
- **Validator Health**: Check if validator is running
- **Batch Operations**: Fund multiple accounts at once
- **Error Handling**: Proper error wrapping and reporting

## Usage

```toml
[dependencies]
chain-forge-solana-rpc = { path = "../rpc" }
```

### Basic Usage

```rust
use chain_forge_solana_rpc::SolanaRpcClient;

#[tokio::main]
async fn main() {
    let client = SolanaRpcClient::new("http://localhost:8899".to_string());

    // Check if validator is running
    if client.is_validator_running() {
        println!("Validator is ready!");
    }

    // Get balance
    let balance = client.get_balance("7xJ5...")?;
    println!("Balance: {} SOL", balance);

    // Request airdrop
    let signature = client.request_airdrop("7xJ5...", 10.0)?;
    println!("Airdrop signature: {}", signature);
}
```

## API Reference

### `SolanaRpcClient`

Wrapper around Solana RPC client with convenience methods.

**Constructor:**

```rust
pub fn new(rpc_url: String) -> Self
```

**Methods:**

#### Connection

```rust
// Get RPC URL
pub fn url(&self) -> &str

// Check if validator is running
pub fn is_validator_running(&self) -> bool

// Wait for validator to be ready
pub async fn wait_for_validator(&self, max_attempts: u32) -> Result<()>
```

#### Balance Operations

```rust
// Get account balance in SOL
pub fn get_balance(&self, address: &str) -> Result<f64>

// Update balances for multiple accounts
pub fn update_balances(&self, accounts: &mut [SolanaAccount]) -> Result<()>
```

#### Funding Operations

```rust
// Request airdrop to an account
pub fn request_airdrop(&self, address: &str, amount_sol: f64) -> Result<String>

// Fund multiple accounts
pub async fn fund_accounts(
    &self,
    accounts: &mut [SolanaAccount],
    amount_sol: f64
) -> Result<()>
```

#### Blockchain Info

```rust
// Get latest blockhash
pub fn get_latest_blockhash(&self) -> Result<Hash>

// Get cluster version
pub fn get_version(&self) -> Result<String>

// Get inner RPC client for advanced ops
pub fn inner(&self) -> &RpcClient
```

## Examples

### Wait for Validator

```rust
use chain_forge_solana_rpc::SolanaRpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = SolanaRpcClient::new("http://localhost:8899".to_string());

    println!("Waiting for validator...");
    client.wait_for_validator(60).await?;
    println!("Validator is ready!");

    Ok(())
}
```

### Fund Multiple Accounts

```rust
use chain_forge_solana_rpc::SolanaRpcClient;
use chain_forge_solana_accounts::SolanaAccount;

#[tokio::main]
async fn main() -> Result<()> {
    let client = SolanaRpcClient::new("http://localhost:8899".to_string());

    let mut accounts = vec![/* ... */];

    println!("Funding {} accounts...", accounts.len());
    client.fund_accounts(&mut accounts, 100.0).await?;

    // Accounts now have updated balances
    for account in accounts {
        println!("{}: {} SOL", account.public_key, account.balance);
    }

    Ok(())
}
```

### Check Validator Health

```rust
use chain_forge_solana_rpc::SolanaRpcClient;

fn check_health() -> Result<()> {
    let client = SolanaRpcClient::new("http://localhost:8899".to_string());

    if !client.is_validator_running() {
        eprintln!("Validator is not running!");
        return Err(ChainError::NotRunning);
    }

    let version = client.get_version()?;
    println!("Validator version: {}", version);

    Ok(())
}
```

### Advanced RPC Operations

```rust
use chain_forge_solana_rpc::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn advanced_operations() -> Result<()> {
    let client = SolanaRpcClient::new("http://localhost:8899".to_string());

    // Get inner RPC client for advanced operations
    let rpc_client = client.inner();

    // Use any solana-client method
    let pubkey = Pubkey::from_str("7xJ5...")?;
    let account_info = rpc_client.get_account(&pubkey)?;

    println!("Lamports: {}", account_info.lamports);
    println!("Owner: {}", account_info.owner);

    Ok(())
}
```

## Error Handling

All methods return `Result<T, ChainError>`. Common errors:

```rust
match client.get_balance(address) {
    Ok(balance) => println!("Balance: {}", balance),
    Err(ChainError::Rpc(e)) => eprintln!("RPC error: {}", e),
    Err(ChainError::NotRunning) => eprintln!("Validator not running"),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Configuration

The RPC client uses these defaults:
- **Timeout**: 30 seconds
- **Commitment**: Confirmed
- **Retry**: Built into `solana-client`

### Custom Configuration

For custom RPC settings, use the inner client:

```rust
let client = SolanaRpcClient::new("http://localhost:8899".to_string());
let rpc_client = client.inner();

// Now you can use any solana-client configuration
```

## Rate Limiting

When funding multiple accounts, the client adds small delays between airdrops to avoid rate limiting:

```rust
// Automatically adds 100ms delay between airdrops
client.fund_accounts(&mut accounts, 100.0).await?;
```

## Testing

```bash
cargo test -p chain-forge-solana-rpc
```

Note: Most tests require a running validator.

## Dependencies

- `solana-client` - Official Solana RPC client
- `solana-sdk` - Solana SDK types
- `tokio` - Async runtime
- `chain-forge-common` - Common error types

## License

MIT OR Apache-2.0
