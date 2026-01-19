# TypeScript Examples

Real-world examples of using Chain Forge with TypeScript.

## Basic Example

Simple validator setup and account usage:

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function main() {
  const client = new SolanaClient({
    accounts: 10,
    initialBalance: 100,
  });

  await client.start();
  console.log('Validator started!');

  const accounts = await client.getAccounts();
  console.log(`Generated ${accounts.length} accounts`);

  client.stop();
}

main().catch(console.error);
```

## Transfer SOL

Transfer SOL between accounts:

```typescript
import { SolanaClient } from '@chain-forge/solana';
import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

async function transferExample() {
  const client = new SolanaClient();
  await client.start();

  // Get accounts
  const accounts = await client.getAccounts();
  const sender = accounts[0];
  const receiver = accounts[1];

  console.log(`Sender: ${sender.publicKey} (${sender.balance} SOL)`);
  console.log(`Receiver: ${receiver.publicKey} (${receiver.balance} SOL)`);

  // Create keypair
  const senderKeypair = Keypair.fromSecretKey(sender.secretKey);
  const connection = client.getConnection();

  // Create transfer
  const transaction = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: new PublicKey(sender.publicKey),
      toPubkey: new PublicKey(receiver.publicKey),
      lamports: 1_000_000_000, // 1 SOL
    })
  );

  // Send and confirm
  const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [senderKeypair]
  );

  console.log(`Transfer successful! Signature: ${signature}`);

  // Check balances
  const senderBalance = await client.getBalance(sender.publicKey);
  const receiverBalance = await client.getBalance(receiver.publicKey);

  console.log(`Sender new balance: ${senderBalance} SOL`);
  console.log(`Receiver new balance: ${receiverBalance} SOL`);

  client.stop();
}

transferExample().catch(console.error);
```

## Test Suite Integration

Complete test suite example:

```typescript
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { SolanaClient } from '@chain-forge/solana';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

describe('Solana Integration Tests', () => {
  let client: SolanaClient;
  let accounts: any[];

  beforeAll(async () => {
    client = new SolanaClient({
      accounts: 5,
      initialBalance: 100,
      mnemonic: 'test test test test test test test test test test test junk',
    });
    await client.start();
    accounts = await client.getAccounts();
  }, 30000);

  afterAll(() => {
    client.stop();
  });

  it('should have correct number of accounts', () => {
    expect(accounts).toHaveLength(5);
  });

  it('should have initial balance', () => {
    accounts.forEach(account => {
      expect(account.balance).toBe(100);
    });
  });

  it('should fund account', async () => {
    const account = accounts[0];
    const initialBalance = await client.getBalance(account.publicKey);

    await client.fundAccount(account.publicKey, 50);

    const newBalance = await client.getBalance(account.publicKey);
    expect(newBalance).toBeGreaterThan(initialBalance);
  });

  it('should transfer SOL', async () => {
    const sender = accounts[0];
    const receiver = accounts[1];

    const senderKeypair = Keypair.fromSecretKey(sender.secretKey);
    const connection = client.getConnection();

    const initialSenderBalance = await client.getBalance(sender.publicKey);
    const initialReceiverBalance = await client.getBalance(receiver.publicKey);

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: new PublicKey(sender.publicKey),
        toPubkey: new PublicKey(receiver.publicKey),
        lamports: 5_000_000_000, // 5 SOL
      })
    );

    await sendAndConfirmTransaction(connection, transaction, [senderKeypair]);

    const finalSenderBalance = await client.getBalance(sender.publicKey);
    const finalReceiverBalance = await client.getBalance(receiver.publicKey);

    // Sender should have less (5 SOL + fees)
    expect(finalSenderBalance).toBeLessThan(initialSenderBalance - 5);

    // Receiver should have exactly 5 SOL more
    expect(finalReceiverBalance).toBe(initialReceiverBalance + 5);
  });
});
```

## CI/CD Example

GitHub Actions workflow:

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install Rust
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          echo "$HOME/.cargo/bin" >> $GITHUB_PATH

      - name: Install Solana
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH

      - name: Install Chain Forge
        run: cargo install --path chains/solana/crates/cli

      - name: Install dependencies
        run: npm install

      - name: Run tests
        run: npm test
```

## Express API Server

Run a local validator with an Express API:

```typescript
import express from 'express';
import { SolanaClient } from '@chain-forge/solana';

const app = express();
const client = new SolanaClient();

app.use(express.json());

// Start validator on server startup
async function startServer() {
  await client.start();
  console.log('Validator started');

  app.get('/accounts', async (req, res) => {
    try {
      const accounts = await client.getAccounts();
      res.json(accounts);
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  app.get('/balance/:publicKey', async (req, res) => {
    try {
      const balance = await client.getBalance(req.params.publicKey);
      res.json({ balance });
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  app.post('/fund', async (req, res) => {
    try {
      const { publicKey, amount } = req.body;
      await client.fundAccount(publicKey, amount);
      const balance = await client.getBalance(publicKey);
      res.json({ success: true, balance });
    } catch (error) {
      res.status(500).json({ error: (error as Error).message });
    }
  });

  const PORT = 3000;
  app.listen(PORT, () => {
    console.log(`API server running on http://localhost:${PORT}`);
  });
}

// Cleanup on exit
process.on('SIGINT', () => {
  console.log('Stopping validator...');
  client.stop();
  process.exit(0);
});

startServer().catch(console.error);
```

## Multiple Validators

Run multiple validators on different ports:

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function multiValidatorExample() {
  const client1 = new SolanaClient({ port: 8899 });
  const client2 = new SolanaClient({ port: 8900 });

  await Promise.all([
    client1.start(),
    client2.start(),
  ]);

  console.log('Both validators running');
  console.log('Validator 1:', client1.getRpcUrl());
  console.log('Validator 2:', client2.getRpcUrl());

  // Use both validators...

  // Cleanup
  client1.stop();
  client2.stop();
}

multiValidatorExample().catch(console.error);
```

## See Also

- [API Reference](../api/overview)
- [Basic Usage Guide](../typescript/basic-usage)
- [Example Projects Repository](https://github.com/yourusername/chain-forge/tree/main/examples)
