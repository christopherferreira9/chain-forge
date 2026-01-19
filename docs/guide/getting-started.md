# Getting Started

This guide will walk you through your first steps with Chain Forge using Solana.

## Quick Start

### 1. Start a Validator

```bash
cf-solana start
```

This command will:
- Generate a 12-word BIP39 mnemonic phrase
- Create 10 pre-funded accounts (100 SOL each)
- Start a local Solana test validator on port 8899
- Display the mnemonic and account information

Example output:

```
Generated mnemonic: word1 word2 word3 ... word12

Starting Solana test validator...
Validator started on http://localhost:8899

Generated 10 accounts with 100.00 SOL each:

Index  Public Key                                    Balance
0      7xJ5k2m8...                                   100.00 SOL
1      8kL2p9n3...                                   100.00 SOL
...
```

::: warning Save Your Mnemonic
The mnemonic is displayed only once. Save it if you need to recover these accounts later!
:::

### 2. View Your Accounts

In a new terminal window:

```bash
cf-solana accounts
```

This displays all generated accounts with their current balances:

```
┌───────┬──────────────────────────────────────────────┬────────────────┐
│ Index │ Public Key                                   │ Balance (SOL)  │
├───────┼──────────────────────────────────────────────┼────────────────┤
│ 0     │ 7xJ5k2m8...                                  │ 100.00         │
│ 1     │ 8kL2p9n3...                                  │ 100.00         │
└───────┴──────────────────────────────────────────────┴────────────────┘
```

### 3. Fund an Account

Request an airdrop to add more SOL:

```bash
cf-solana fund <PUBLIC_KEY> 50
```

Example:

```bash
cf-solana fund 7xJ5k2m8... 50
# Requested airdrop of 50 SOL to 7xJ5k2m8...
# New balance: 150.00 SOL
```

### 4. Stop the Validator

Press `Ctrl+C` in the terminal running the validator.

## CLI Usage

### Custom Configuration

Start with custom settings:

```bash
# More accounts
cf-solana start --accounts 20 --balance 500

# Different port
cf-solana start --port 8900

# Specific mnemonic
cf-solana start --mnemonic "word1 word2 ... word12"
```

### View Configuration

See your current configuration:

```bash
cf-solana config
```

Output:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899
```

### Export Accounts

Export account data to JSON:

```bash
cf-solana accounts --format json > accounts.json
```

## TypeScript Usage

### Installation

```bash
npm install @chain-forge/solana @solana/web3.js
```

### Basic Example

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function main() {
  // Create client with options
  const client = new SolanaClient({
    accounts: 10,
    initialBalance: 100,
    port: 8899,
  });

  // Start the validator
  await client.start();
  console.log('Validator started!');

  // Get generated accounts
  const accounts = await client.getAccounts();
  console.log(`Generated ${accounts.length} accounts`);
  console.log(`First account: ${accounts[0].publicKey}`);

  // Fund an account
  await client.fundAccount(accounts[0].publicKey, 5);
  console.log('Added 5 SOL to first account');

  // Check balance
  const balance = await client.getBalance(accounts[0].publicKey);
  console.log(`Current balance: ${balance} SOL`);

  // Stop when done
  client.stop();
}

main().catch(console.error);
```

### Using with Solana Web3.js

Chain Forge integrates seamlessly with `@solana/web3.js`:

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { Connection, PublicKey, Transaction, SystemProgram } from '@solana/web3.js';

const client = new SolanaClient();
await client.start();

// Get the Connection object
const connection = client.getConnection();

// Get accounts
const accounts = await client.getAccounts();
const sender = accounts[0];
const receiver = accounts[1];

// Use with standard Solana operations
const balance = await connection.getBalance(
  new PublicKey(sender.publicKey)
);
console.log(`Balance: ${balance} lamports`);

// Create and send a transaction
const transaction = new Transaction().add(
  SystemProgram.transfer({
    fromPubkey: new PublicKey(sender.publicKey),
    toPubkey: new PublicKey(receiver.publicKey),
    lamports: 1_000_000, // 0.001 SOL
  })
);

// Sign and send...
```

### Testing Integration

Use Chain Forge in your test suites:

```typescript
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { SolanaClient } from '@chain-forge/solana';

describe('Solana Integration Tests', () => {
  let client: SolanaClient;

  beforeAll(async () => {
    client = new SolanaClient({
      accounts: 5,
      initialBalance: 100,
    });
    await client.start();
  });

  afterAll(() => {
    client.stop();
  });

  it('should have pre-funded accounts', async () => {
    const accounts = await client.getAccounts();
    expect(accounts).toHaveLength(5);
    expect(accounts[0].balance).toBe(100);
  });

  it('should fund accounts', async () => {
    const accounts = await client.getAccounts();
    const initialBalance = await client.getBalance(accounts[0].publicKey);

    await client.fundAccount(accounts[0].publicKey, 10);

    const newBalance = await client.getBalance(accounts[0].publicKey);
    expect(newBalance).toBeGreaterThan(initialBalance);
  });
});
```

## Configuration

### Project Configuration

Create `chain-forge.toml` in your project root:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.ci]
# Lighter configuration for CI
accounts = 3
initial_balance = 10.0

[solana.development]
# Heavy setup for local development
accounts = 50
initial_balance = 1000.0
port = 8899
```

Use a specific profile:

```bash
cf-solana start --profile ci
```

### Global Configuration

Create `~/.chain-forge/config.toml` for user-wide defaults:

```toml
[solana.default]
accounts = 20
initial_balance = 500.0
```

Configuration priority:
1. CLI arguments (highest)
2. Project `chain-forge.toml`
3. Global `~/.chain-forge/config.toml`
4. Built-in defaults (lowest)

## Best Practices

### Mnemonic Management

**Development**
- Let Chain Forge generate mnemonics automatically
- Don't worry about saving them for throwaway validators

**Testing**
- Use a fixed mnemonic for reproducible tests:

```bash
cf-solana start --mnemonic "test test test test test test test test test test test junk"
```

**Never in Production**
- Don't use test mnemonics with real funds
- Never commit mnemonics to version control

### Account Security

```bash
# Add to .gitignore
echo ".chain-forge/" >> .gitignore
echo "accounts.json" >> .gitignore
```

Never:
- Commit `~/.chain-forge/` to Git
- Share accounts that hold real funds
- Use development mnemonics in production

### Port Management

Running multiple validators:

```bash
# Terminal 1
cf-solana start --port 8899

# Terminal 2
cf-solana start --port 8900

# Terminal 3
cf-solana start --port 8901
```

Check if a port is in use:

```bash
lsof -i :8899
```

### CI/CD Integration

GitHub Actions example:

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Solana
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH

      - name: Install Chain Forge
        run: cargo install --path chains/solana/crates/cli

      - name: Start validator
        run: |
          cf-solana start --accounts 3 --balance 10 &
          sleep 5  # Wait for startup

      - name: Run tests
        run: npm test
```

### Cleanup

Always stop validators when done:

**CLI**: Press `Ctrl+C` in the validator terminal

**TypeScript**: Call `client.stop()`

```typescript
try {
  await client.start();
  // ... your code
} finally {
  client.stop();
}
```

**Testing**: Use proper teardown

```typescript
afterAll(() => {
  client.stop();
});
```

## Common Workflows

### Smart Contract Development

```bash
# Start validator
cf-solana start --accounts 10 --balance 100

# Deploy your program (in another terminal)
anchor build
anchor deploy

# Test your program
anchor test --skip-local-validator
```

### Frontend Development

```bash
# Start validator with many accounts
cf-solana start --accounts 50 --balance 1000

# Run your frontend (in another terminal)
npm run dev
# Connect to http://localhost:8899
```

### Integration Testing

```typescript
// test-setup.ts
import { SolanaClient } from '@chain-forge/solana';

export const client = new SolanaClient({
  accounts: 10,
  initialBalance: 100,
});

// Run once before all tests
export async function setup() {
  await client.start();
}

export function teardown() {
  client.stop();
}
```

## Next Steps

- [Solana CLI Reference](../solana/cli) - All CLI commands
- [Configuration Guide](../solana/configuration) - Advanced configuration
- [TypeScript API](../api/overview) - Full API reference
- [Examples](../examples/typescript) - Real-world examples
