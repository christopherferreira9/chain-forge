# CLI Commands

Complete reference for the `cf-solana` command-line tool.

## cf-solana

```bash
cf-solana [OPTIONS] <COMMAND>
```

### Global Options

- `--help`, `-h` - Display help information
- `--version`, `-V` - Display version information

## Commands

### start

Start a local Solana test validator with pre-funded accounts.

```bash
cf-solana start [OPTIONS]
```

#### Options

- `--accounts <NUM>` - Number of accounts to generate (default: 10)
- `--balance <SOL>` - Initial balance in SOL for each account (default: 100.0)
- `--port <PORT>` - RPC port for the validator (default: 8899)
- `--mnemonic <PHRASE>` - Use specific 12-word mnemonic phrase
- `--profile <NAME>` - Use specific configuration profile (default: "default")

#### Examples

```bash
# Basic usage (10 accounts with 100 SOL each)
cf-solana start

# More accounts with higher balance
cf-solana start --accounts 20 --balance 500

# Custom port
cf-solana start --port 8900

# Use specific mnemonic for reproducibility
cf-solana start --mnemonic "test test test test test test test test test test test junk"

# Use CI configuration profile
cf-solana start --profile ci
```

#### Output

```
Generated mnemonic: word1 word2 word3 ... word12

Starting Solana test validator...
Validator started on http://localhost:8899

Generated 10 accounts with 100.00 SOL each:

Index  Public Key                                    Balance
0      7xJ5k2m8...                                   100.00 SOL
1      8kL2p9n3...                                   100.00 SOL
...

Press Ctrl+C to stop the validator
```

#### Behavior

1. Generates or uses provided mnemonic
2. Derives accounts using BIP44 path `m/44'/501'/index'/0'`
3. Starts `solana-test-validator` process
4. Waits for validator to be ready
5. Funds accounts via airdrop
6. Saves account data to `~/.chain-forge/solana/accounts.json`
7. Runs in foreground until Ctrl+C

### accounts

List all generated accounts and their balances.

```bash
cf-solana accounts [OPTIONS]
```

#### Options

- `--format <FORMAT>` - Output format: `table` (default) or `json`
- `--url <URL>` - Custom RPC URL (default: http://localhost:8899)

#### Examples

```bash
# Display accounts in table format
cf-solana accounts

# Export to JSON
cf-solana accounts --format json

# Save to file
cf-solana accounts --format json > accounts.json

# Query custom validator
cf-solana accounts --url http://localhost:8900
```

#### Output

**Table format:**

```
┌───────┬──────────────────────────────────────────────┬────────────────┐
│ Index │ Public Key                                   │ Balance (SOL)  │
├───────┼──────────────────────────────────────────────┼────────────────┤
│ 0     │ 7xJ5k2m8...                                  │ 100.00         │
│ 1     │ 8kL2p9n3...                                  │ 100.00         │
│ 2     │ 9mN3r5s7...                                  │ 100.00         │
└───────┴──────────────────────────────────────────────┴────────────────┘
```

**JSON format:**

```json
[
  {
    "index": 0,
    "publicKey": "7xJ5k2m8...",
    "secretKey": "...",
    "balance": 100.0
  },
  {
    "index": 1,
    "publicKey": "8kL2p9n3...",
    "secretKey": "...",
    "balance": 100.0
  }
]
```

::: danger
JSON output includes private keys. Never commit or share this data!
:::

### fund

Request an airdrop to fund an account.

```bash
cf-solana fund <ADDRESS> <AMOUNT>
```

#### Arguments

- `<ADDRESS>` - Public key (base58 encoded) of the account to fund
- `<AMOUNT>` - Amount of SOL to airdrop (e.g., 50 or 50.5)

#### Options

- `--url <URL>` - Custom RPC URL (default: http://localhost:8899)

#### Examples

```bash
# Fund account with 50 SOL
cf-solana fund 7xJ5k2m8... 50

# Fund with fractional amount
cf-solana fund 8kL2p9n3... 25.5

# Fund on custom validator
cf-solana fund 9mN3r5s7... 100 --url http://localhost:8900
```

#### Output

```
Requesting airdrop of 50.00 SOL to 7xJ5k2m8...
Airdrop successful!
New balance: 150.00 SOL
```

#### Notes

- Subject to Solana airdrop rate limiting
- If airdrop fails, wait a few seconds and try again
- Maximum airdrop amount depends on validator configuration (typically 1-2 SOL per request on devnet, unlimited on local)

### config

Display current configuration.

```bash
cf-solana config [OPTIONS]
```

#### Options

- `--profile <NAME>` - Show specific profile (default: "default")
- `--format <FORMAT>` - Output format: `toml` (default) or `json`

#### Examples

```bash
# Show default profile configuration
cf-solana config

# Show CI profile
cf-solana config --profile ci

# Output as JSON
cf-solana config --format json
```

#### Output

**TOML format:**

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899
```

**JSON format:**

```json
{
  "solana": {
    "default": {
      "rpc_url": "http://localhost:8899",
      "accounts": 10,
      "initial_balance": 100.0,
      "port": 8899
    }
  }
}
```

## Configuration Files

Chain Forge reads configuration from multiple locations:

1. **CLI arguments** (highest priority)
2. **Project config**: `./chain-forge.toml`
3. **Global config**: `~/.chain-forge/config.toml`
4. **Built-in defaults** (lowest priority)

### Project Configuration

Create `chain-forge.toml` in your project root:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.ci]
accounts = 3
initial_balance = 10.0

[solana.development]
accounts = 50
initial_balance = 1000.0
```

### Global Configuration

Create `~/.chain-forge/config.toml` for user-wide defaults:

```toml
[solana.default]
accounts = 20
initial_balance = 500.0
```

## Environment Variables

Currently no environment variables are supported. All configuration is via files or CLI arguments.

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Validator startup error
- `4` - RPC error

## Shell Completion

Generate shell completions (feature planned for future release):

```bash
# Bash
cf-solana completion bash > /usr/local/etc/bash_completion.d/cf-solana

# Zsh
cf-solana completion zsh > ~/.zsh/completion/_cf-solana

# Fish
cf-solana completion fish > ~/.config/fish/completions/cf-solana.fish
```

## Common Workflows

### Development

```bash
# Start validator with generous resources
cf-solana start --accounts 50 --balance 1000

# In another terminal, use the accounts
cf-solana accounts
```

### Testing

```bash
# Use fixed mnemonic for reproducible tests
cf-solana start --mnemonic "test test test test test test test test test test test junk"

# Lightweight setup
cf-solana start --accounts 3 --balance 10
```

### CI/CD

```bash
# Use CI profile from chain-forge.toml
cf-solana start --profile ci

# Or inline configuration
cf-solana start --accounts 3 --balance 10 &
sleep 5  # Wait for startup
npm test
```

## Troubleshooting

### Command Not Found

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Port Already in Use

Check what's using the port:

```bash
lsof -i :8899
```

Kill the process or use a different port:

```bash
cf-solana start --port 8900
```

### Validator Fails to Start

Check Solana CLI is installed:

```bash
solana --version
solana-test-validator --help
```

Check logs:

```bash
cat ~/.chain-forge/solana/validator.log
```

### Airdrop Failures

Rate limiting is common. Wait a few seconds between requests:

```bash
cf-solana fund <ADDRESS> 50
sleep 2
cf-solana fund <ADDRESS2> 50
```

## See Also

- [Configuration Guide](./configuration)
- [Account Management](./accounts)
- [TypeScript Package](../typescript/basic-usage)
