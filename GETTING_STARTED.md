# Getting Started with Chain Forge

This guide will walk you through setting up and using Chain Forge for local blockchain development.

## Table of Contents

1. [Installation](#installation)
2. [Quick Start - Solana](#quick-start---solana)
3. [Using the CLI](#using-the-cli)
4. [Using the TypeScript Package](#using-the-typescript-package)
5. [Configuration](#configuration)
6. [Best Practices](#best-practices)

## Installation

### Prerequisites

- **Rust**: Version 1.75 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Solana CLI Tools**: Required for running the test validator
  ```bash
  sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
  ```

- **Node.js**: Version 18 or later (for TypeScript package)
  ```bash
  # Check version
  node --version
  ```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/christopherferreira9/chain-forge
cd chain-forge

# Build the workspace
cargo build --workspace --release

# Install the Solana CLI
cargo install --path chains/solana/crates/cli

# Verify installation
cf-solana --version
```

## Quick Start - Solana

### 1. Start the Validator

```bash
cf-solana start
```

This command will:
- Generate a 12-word mnemonic phrase
- Create 10 pre-funded accounts (100 SOL each)
- Start a local Solana test validator on port 8899
- Display the mnemonic and account information

**Important**: Save the mnemonic phrase! You'll need it to recover your accounts.

### 2. View Your Accounts

In a new terminal:

```bash
cf-solana accounts
```

Output:
```
┌───────┬──────────────────────────────────────────────┬────────────────┐
│ Index │ Public Key                                   │ Balance (SOL)  │
├───────┼──────────────────────────────────────────────┼────────────────┤
│ 0     │ 7xJ5...                                      │ 100.00         │
│ 1     │ 8kL2...                                      │ 100.00         │
│ ...   │ ...                                          │ ...            │
└───────┴──────────────────────────────────────────────┴────────────────┘
```

### 3. Fund an Account

```bash
cf-solana fund 7xJ5... 50
```

This requests an airdrop of 50 SOL to the specified address.

### 4. Stop the Validator

Press `Ctrl+C` in the terminal running the validator.

## Using the CLI

### Start with Custom Configuration

```bash
# Start with 20 accounts, 500 SOL each, on port 8900
cf-solana start --accounts 20 --balance 500 --port 8900

# Use a specific mnemonic
cf-solana start --mnemonic "your twelve word phrase here..."
```

### Export Accounts to JSON

```bash
cf-solana accounts --format json > accounts.json
```

### View Configuration

```bash
cf-solana config
```

## Using the TypeScript Package

### 1. Install the Package

```bash
npm install @chain-forge/solana @solana/web3.js
```

### 2. Basic Usage

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function main() {
  // Create client
  const client = new SolanaClient({
    accounts: 10,
    initialBalance: 100,
  });

  // Start validator
  await client.start();
  console.log('Validator started!');

  // Get accounts
  const accounts = await client.getAccounts();
  console.log(`Generated ${accounts.length} accounts`);
  console.log(`First account: ${accounts[0].publicKey}`);

  // Fund an account
  await client.fundAccount(accounts[0].publicKey, 5);
  console.log('Funded account with 5 SOL');

  // Check balance
  const balance = await client.getBalance(accounts[0].publicKey);
  console.log(`Balance: ${balance} SOL`);

  // Stop validator
  client.stop();
}

main();
```

### 3. Integration with Solana Web3.js

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { Transaction, SystemProgram, PublicKey } from '@solana/web3.js';

const client = new SolanaClient();
await client.start();

// Get Connection
const connection = client.getConnection();

// Get accounts
const accounts = await client.getAccounts();
const sender = accounts[0];
const receiver = accounts[1];

// Create and send a transaction
const transaction = new Transaction().add(
  SystemProgram.transfer({
    fromPubkey: new PublicKey(sender.publicKey),
    toPubkey: new PublicKey(receiver.publicKey),
    lamports: 1_000_000, // 0.001 SOL
  })
);

// ... sign and send transaction
```

### 4. Testing Framework Integration

```typescript
import { SolanaClient } from '@chain-forge/solana';

describe('My Solana Tests', () => {
  let client: SolanaClient;

  beforeAll(async () => {
    client = new SolanaClient();
    await client.start();
  });

  afterAll(() => {
    client.stop();
  });

  it('should have pre-funded accounts', async () => {
    const accounts = await client.getAccounts();
    expect(accounts).toHaveLength(10);
    expect(accounts[0].balance).toBe(100);
  });

  it('should fund accounts', async () => {
    const accounts = await client.getAccounts();
    await client.fundAccount(accounts[0].publicKey, 5);
    const balance = await client.getBalance(accounts[0].publicKey);
    expect(balance).toBeGreaterThan(100);
  });
});
```

## Configuration

### Project Configuration File

Create `chain-forge.toml` in your project directory:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.ci]
# Lighter configuration for CI environments
accounts = 3
initial_balance = 10.0
```

### Global Configuration

Place `chain-forge.toml` in your home directory for global defaults.

**Priority**: Project config > Global config > Built-in defaults

## Best Practices

### 1. Mnemonic Management

**Development**: Let Chain Forge generate mnemonics automatically.

**Testing**: Use fixed mnemonics for reproducible tests:
```bash
cf-solana start --mnemonic "test test test test test test test test test test test junk"
```

**Production**: Never use test mnemonics or expose private keys.

### 2. Account Security

- Never commit `~/.chain-forge/` to version control
- Add `.chain-forge/` to your `.gitignore`
- Use separate mnemonics for different projects
- Don't share accounts with real funds

### 3. Port Management

If running multiple validators:
```bash
# Validator 1
cf-solana start --port 8899

# Validator 2 (in another terminal)
cf-solana start --port 8900
```

### 4. CI/CD Integration

```yaml
# GitHub Actions example
- name: Install Solana
  run: sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

- name: Install Chain Forge
  run: cargo install --path chains/solana/crates/cli

- name: Start validator
  run: cf-solana start --accounts 3 --balance 10 &

- name: Wait for validator
  run: sleep 5

- name: Run tests
  run: npm test
```

### 5. Cleanup

Always stop validators when done:
- CLI: Press `Ctrl+C`
- TypeScript: Call `client.stop()`
- Testing: Use `afterAll()` hooks

## Next Steps

- Explore the [Solana README](../chains/solana/README.md)
- Read the [TypeScript package docs](../npm/@chain-forge/solana/README.md)
- Check out example projects in `examples/`
- Learn about [advanced configuration](./CONFIGURATION.md)

## Troubleshooting

### Validator Won't Start

1. Check Solana CLI is installed:
   ```bash
   solana --version
   ```

2. Check port availability:
   ```bash
   lsof -i :8899
   ```

3. Try a different port:
   ```bash
   cf-solana start --port 8900
   ```

### TypeScript Package Issues

1. Ensure CLI is installed:
   ```bash
   which cf-solana
   ```

2. Check Node version:
   ```bash
   node --version  # Should be 18+
   ```

3. Reinstall dependencies:
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   ```

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/christopherferreira9/chain-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/youruserchristopherferreira9name/chain-forge/discussions)
- **Documentation**: [docs/](.)
