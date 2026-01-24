# CLI Commands

Complete reference for the `cf-bitcoin` command-line tool.

## cf-bitcoin

```bash
cf-bitcoin [OPTIONS] <COMMAND>
```

### Global Options

- `--help`, `-h` - Display help information
- `--version`, `-V` - Display version information

## Commands

### start

Start a local Bitcoin regtest node with pre-funded accounts.

```bash
cf-bitcoin start [OPTIONS]
```

#### Options

- `--instance <ID>` - Instance ID for isolation (default: "default")
- `--accounts <NUM>` - Number of accounts to generate (default: 10)
- `--balance <BTC>` - Initial balance in BTC for each account (default: 10.0)
- `--rpc-port <PORT>` - RPC port for the node (default: 18443)
- `--p2p-port <PORT>` - P2P network port (default: 18444)
- `--mnemonic <PHRASE>` - Use specific 12-word mnemonic phrase
- `--rpc-user <USER>` - RPC username (default: "chainforge")
- `--rpc-password <PASS>` - RPC password (default: "chainforge")
- `--verbose`, `-v` - Show verbose bitcoind output

#### Examples

```bash
# Basic usage (10 accounts with 10 BTC each)
cf-bitcoin start

# More accounts with higher balance
cf-bitcoin start --accounts 20 --balance 50

# Custom ports
cf-bitcoin start --rpc-port 18445 --p2p-port 18446

# Named instance for isolation
cf-bitcoin start --instance mytest

# Use specific mnemonic for reproducibility
cf-bitcoin start --mnemonic "test test test test test test test test test test test junk"

# Multiple instances (run in separate terminals)
cf-bitcoin start --instance node1 --rpc-port 18443
cf-bitcoin start --instance node2 --rpc-port 18445
```

#### Output

```
🧹 Clearing previous instance data for 'default'...
🔑 Mnemonic: word1 word2 word3 ... word12
   Save this mnemonic to recover your accounts!

🚀 Starting Bitcoin regtest node on port 18443...
⏳ Waiting for Bitcoin node to be ready...
✅ Bitcoin node is ready!

📦 Creating wallet...
📍 Mining address: bcrt1q...
⛏️  Mining 103 initial blocks (this may take a moment)...
   Wallet balance: 150.00 BTC (need 100.00 BTC for 10 accounts)
💰 Funding 10 accounts with 10.00 BTC each...
   Sent 10.00 BTC to account 0 bcrt1q... (txid: abc123...)
   Sent 10.00 BTC to account 1 bcrt1q... (txid: def456...)
   ...
⛏️  Mining 6 blocks to confirm transactions...
📥 Importing accounts into wallet...
✅ All accounts funded!

🎉 Bitcoin regtest node is running!
   RPC URL: http://localhost:18443

💡 Tip: Keep this terminal open to keep the node running
   Run 'cf-bitcoin accounts --instance default' in another terminal to see your accounts
   Run 'cf-bitcoin mine --instance default' to mine new blocks
```

#### Behavior

1. Clears previous instance data (clean slate)
2. Generates or uses provided mnemonic
3. Derives accounts using BIP44 path `m/44'/0'/0'/0/index`
4. Starts `bitcoind` in regtest mode
5. Creates wallet and mines initial blocks
6. Funds accounts from mining rewards
7. Imports accounts into wallet
8. Saves account data to instance directory
9. Runs in foreground until Ctrl+C

### accounts

List all generated accounts and their balances.

```bash
cf-bitcoin accounts [OPTIONS]
```

#### Options

- `--instance <ID>` - Instance ID to query (default: "default")
- `--format <FORMAT>` - Output format: `table` (default) or `json`

#### Examples

```bash
# Display accounts in table format
cf-bitcoin accounts

# Query specific instance
cf-bitcoin accounts --instance mytest

# Export to JSON
cf-bitcoin accounts --format json

# Save to file
cf-bitcoin accounts --format json > accounts.json
```

#### Output

**Table format:**

```
┌───────┬──────────────────────────────────────────────┬────────────────┐
│ Index │ Address                                      │ Balance (BTC)  │
├───────┼──────────────────────────────────────────────┼────────────────┤
│ 0     │ bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080 │ 10.00000000    │
│ 1     │ bcrt1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0g │ 10.00000000    │
│ 2     │ bcrt1q0ht9tyks4vh7p5p904t340cr9nvahy7um9zdem │ 10.00000000    │
└───────┴──────────────────────────────────────────────┴────────────────┘
```

**JSON format:**

```json
[
  {
    "address": "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080",
    "publicKey": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    "privateKey": [/* bytes */],
    "wif": "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy",
    "mnemonic": "test test test...",
    "derivationPath": "m/44'/0'/0'/0/0",
    "balance": 10.0
  }
]
```

::: danger
JSON output includes private keys. Never commit or share this data!
:::

### fund

Send BTC to an address from wallet funds (mining rewards).

```bash
cf-bitcoin fund <ADDRESS> <AMOUNT> [OPTIONS]
```

#### Arguments

- `<ADDRESS>` - Bitcoin address (bech32 format for regtest: bcrt1...)
- `<AMOUNT>` - Amount of BTC to send (e.g., 5 or 5.5)

#### Options

- `--instance <ID>` - Instance ID to use (default: "default")

#### Examples

```bash
# Fund account with 5 BTC
cf-bitcoin fund bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080 5

# Fund on specific instance
cf-bitcoin fund bcrt1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0g 10 --instance mytest
```

#### Output

```
💰 Sending 5.00 BTC to bcrt1qw508d6... (from wallet)...
✅ Transaction sent!
   TxID: a1b2c3d4e5f6...
⛏️  Mining block to confirm transaction...
   Block mined: 000000abc123...
   New balance: 15.00 BTC
```

#### Notes

- Sends from wallet's mining rewards, not from a specific account
- Automatically mines a block to confirm
- Use `transfer` to send from a specific account

### transfer

Transfer BTC from one account to another.

```bash
cf-bitcoin transfer <FROM> <TO> <AMOUNT> [OPTIONS]
```

#### Arguments

- `<FROM>` - Source Bitcoin address
- `<TO>` - Destination Bitcoin address
- `<AMOUNT>` - Amount of BTC to send

#### Options

- `--instance <ID>` - Instance ID to use (default: "default")

#### Examples

```bash
# Transfer 1 BTC between accounts
cf-bitcoin transfer bcrt1qw508d6... bcrt1qrp33g0... 1

# Transfer on specific instance
cf-bitcoin transfer bcrt1qw508d6... bcrt1qrp33g0... 5 --instance mytest
```

#### Output

```
💸 Transferring 1.00 BTC
   From: bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7k...
   To:   bcrt1qrp33g0q5c5txsp9arysrx4k6zdkfs4nc...

   Source balance: 10.00 BTC
✅ Transaction sent!
   TxID: a1b2c3d4e5f6...
⛏️  Mining block to confirm transaction...
   Block mined: 000000abc123...

Updated balances:
   From: 8.9999 BTC
   To:   11.00 BTC
```

#### Notes

- Creates a real Bitcoin transaction using source account's UTXOs
- Includes automatic fee estimation (~0.0001 BTC)
- Change is returned to source address
- Automatically mines a block to confirm

### mine

Mine blocks to an address.

```bash
cf-bitcoin mine [OPTIONS]
```

#### Options

- `--blocks <NUM>` - Number of blocks to mine (default: 1)
- `--address <ADDR>` - Address to receive coinbase rewards (default: wallet address)
- `--instance <ID>` - Instance ID to use (default: "default")

#### Examples

```bash
# Mine 1 block (default)
cf-bitcoin mine

# Mine 10 blocks
cf-bitcoin mine --blocks 10

# Mine to specific address
cf-bitcoin mine --blocks 5 --address bcrt1qw508d6...

# Mine on specific instance
cf-bitcoin mine --instance mytest
```

#### Output

```
⛏️  Mining 10 block(s) to bcrt1q...
✅ Mined 10 block(s)!
   Block 1: 000000abc123...
   Block 2: 000000def456...
   ...
   Current height: 215
```

#### Notes

- By default, rewards go to a wallet-generated address (not user accounts)
- Use `--address` to direct rewards to a specific account
- Each block reward is 50 BTC (Bitcoin's original reward)
- Coinbase rewards require 100 confirmations to be spendable

### config

Display current configuration for an instance.

```bash
cf-bitcoin config [OPTIONS]
```

#### Options

- `--instance <ID>` - Instance ID to show (default: "default")

#### Examples

```bash
# Show default instance config
cf-bitcoin config

# Show specific instance config
cf-bitcoin config --instance mytest
```

#### Output

```
Chain Forge Bitcoin Configuration
==================================

Instance: default
  Status: Running (may be stale)
  RPC URL: http://localhost:18443
  RPC Port: 18443
  P2P Port: 18444

Instance Directory: "/Users/you/.chain-forge/bitcoin/instances/default"

Global Config (chain-forge.toml):
  Default Profile:
    Accounts: 10
    Initial Balance: 10.0 BTC
```

### stop

Mark an instance as stopped (informational only).

```bash
cf-bitcoin stop [OPTIONS]
```

#### Options

- `--instance <ID>` - Instance ID to stop (default: "default")

#### Notes

- The actual node is stopped by pressing Ctrl+C in the `start` terminal
- This command updates the instance status file

## Configuration Files

Chain Forge reads configuration from multiple locations:

1. **CLI arguments** (highest priority)
2. **Project config**: `./chain-forge.toml`
3. **Global config**: `~/.chain-forge/config.toml`
4. **Built-in defaults** (lowest priority)

### Project Configuration

Create `chain-forge.toml` in your project root:

```toml
[bitcoin.default]
rpc_url = "http://localhost:18443"
accounts = 10
initial_balance = 10.0
rpc_port = 18443
p2p_port = 18444

[bitcoin.ci]
accounts = 3
initial_balance = 5.0

[bitcoin.heavy]
accounts = 50
initial_balance = 100.0
```

## Exit Codes

- `0` - Success
- `1` - General error

## Common Workflows

### Development

```bash
# Start node with generous resources
cf-bitcoin start --accounts 20 --balance 100

# In another terminal, check accounts
cf-bitcoin accounts

# Transfer between accounts
cf-bitcoin transfer <from> <to> 10

# Mine to confirm and check balances
cf-bitcoin accounts
```

### Testing

```bash
# Use fixed mnemonic for reproducible tests
cf-bitcoin start --mnemonic "test test test test test test test test test test test junk"

# Lightweight setup
cf-bitcoin start --accounts 3 --balance 5
```

### CI/CD

```bash
# Start in background
cf-bitcoin start --instance ci --accounts 3 &
sleep 30  # Wait for startup

# Run tests
npm test

# Cleanup (Ctrl+C or kill)
```

### Multiple Instances

```bash
# Terminal 1: First node
cf-bitcoin start --instance node1 --rpc-port 18443

# Terminal 2: Second node (same mnemonic, different state)
cf-bitcoin start --instance node2 --rpc-port 18445 --mnemonic "same mnemonic..."

# Terminal 3: Query either node
cf-bitcoin accounts --instance node1
cf-bitcoin accounts --instance node2
```

## Troubleshooting

### Command Not Found

Ensure the binary is built and in PATH:

```bash
cargo build -p chain-forge-bitcoin-cli
export PATH="$PWD/target/debug:$PATH"
```

Or install globally:

```bash
cargo install --path chains/bitcoin/crates/cli
```

### bitcoind Not Found

Install Bitcoin Core:

```bash
# macOS
brew install bitcoin

# Ubuntu
sudo apt install bitcoind

# Or download from https://bitcoin.org/en/download
```

### Port Already in Use

Check what's using the port:

```bash
lsof -i :18443
```

Use a different port:

```bash
cf-bitcoin start --rpc-port 18445 --p2p-port 18446
```

### Instance Not Found

Ensure the instance was started:

```bash
cf-bitcoin start --instance mytest
```

Then query it:

```bash
cf-bitcoin accounts --instance mytest
```

## See Also

- [Configuration Guide](./configuration)
- [Account Management](./accounts)
- [TypeScript Package](../typescript/basic-usage)
