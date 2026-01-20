# TypeScript Examples

Real-world examples of using Chain Forge with TypeScript.

## Basic Example

The `typescript-basic` example demonstrates a minimal CI-friendly workflow:

```typescript
import { SolanaClient } from '@chain-forge/solana';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';

async function main() {
  // Create client with 3 accounts
  const client = new SolanaClient({
    accounts: 3,
    initialBalance: 100,
    port: 8899,
  });

  try {
    await client.start();
    const accounts = await client.getAccounts();

    // Transfer 5 SOL from account 0 to account 1
    const sender = accounts[0];
    const receiver = accounts[1];
    const senderKeypair = Keypair.fromSecretKey(new Uint8Array(sender.secretKey));
    const connection = client.getConnection();

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: new PublicKey(sender.publicKey),
        toPubkey: new PublicKey(receiver.publicKey),
        lamports: 5 * LAMPORTS_PER_SOL,
      })
    );

    await sendAndConfirmTransaction(connection, transaction, [senderKeypair]);
    console.log('Transfer complete!');
  } finally {
    client.stop();
  }
}

main();
```

### Running the Example

```bash
cd examples/typescript-basic
yarn install
yarn build
yarn start
```

## Interactive CLI

For a full-featured interactive experience, see the [Interactive CLI Example](./interactive-cli).

## Test Suite Integration

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

  it('should transfer SOL', async () => {
    const sender = accounts[0];
    const receiver = accounts[1];

    const senderKeypair = Keypair.fromSecretKey(new Uint8Array(sender.secretKey));
    const connection = client.getConnection();

    const initialReceiverBalance = await client.getBalance(receiver.publicKey);

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: new PublicKey(sender.publicKey),
        toPubkey: new PublicKey(receiver.publicKey),
        lamports: 5_000_000_000, // 5 SOL
      })
    );

    await sendAndConfirmTransaction(connection, transaction, [senderKeypair]);

    const finalReceiverBalance = await client.getBalance(receiver.publicKey);
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

- [Interactive CLI Example](./interactive-cli)
- [Program Deployment](./program-deployment)
- [API Reference](../api/overview)
