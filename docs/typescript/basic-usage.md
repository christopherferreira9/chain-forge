# TypeScript Basic Usage

Learn how to use the `@chain-forge/solana` package in your TypeScript projects.

## Installation

```bash
npm install @chain-forge/solana @solana/web3.js
```

The package requires `@solana/web3.js` as a peer dependency.

## Prerequisites

The TypeScript package requires the `cf-solana` CLI to be installed:

```bash
cargo install chain-forge-solana-cli
```

The package spawns `cf-solana` processes to manage the validator.

## Quick Start

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function main() {
  // Create client
  const client = new SolanaClient();

  // Start validator
  await client.start();

  // Get accounts
  const accounts = await client.getAccounts();
  console.log('First account:', accounts[0].publicKey);

  // Stop validator
  client.stop();
}

main();
```

## Creating a Client

### Default Options

```typescript
const client = new SolanaClient();
// Uses defaults: 10 accounts, 100 SOL each, port 8899
```

### Custom Options

```typescript
const client = new SolanaClient({
  accounts: 20,           // Generate 20 accounts
  initialBalance: 500,    // 500 SOL per account
  port: 8900,             // Use port 8900
});
```

### With Fixed Mnemonic

For reproducible tests:

```typescript
const client = new SolanaClient({
  mnemonic: 'test test test test test test test test test test test junk',
  accounts: 5,
  initialBalance: 10,
});
```

## Starting the Validator

```typescript
await client.start();
```

This will:
1. Generate or load accounts
2. Start the validator process
3. Wait for RPC to be ready
4. Fund all accounts
5. Resolve when complete

Typical startup time: 5-10 seconds

## Working with Accounts

### List All Accounts

```typescript
const accounts = await client.getAccounts();

accounts.forEach(account => {
  console.log(`${account.index}: ${account.publicKey} (${account.balance} SOL)`);
});
```

### Get Single Account

```typescript
const accounts = await client.getAccounts();
const firstAccount = accounts[0];

console.log('Public Key:', firstAccount.publicKey);
console.log('Secret Key:', firstAccount.secretKey); // Uint8Array
console.log('Balance:', firstAccount.balance);
```

### Check Balance

```typescript
const balance = await client.getBalance(publicKey);
console.log(`Balance: ${balance} SOL`);
```

### Fund Account

```typescript
// Add 50 SOL
await client.fundAccount(publicKey, 50);

// Check new balance
const newBalance = await client.getBalance(publicKey);
```

### Set Exact Balance

```typescript
// Set balance to exactly 200 SOL
await client.setBalance(publicKey, 200);
```

Note: Can only increase balance, not decrease (Solana limitation).

## Integration with Solana Web3.js

### Get Connection

```typescript
import { Connection } from '@solana/web3.js';

const connection = client.getConnection();

// Use Connection methods
const blockHeight = await connection.getBlockHeight();
const slot = await connection.getSlot();
```

### Send Transactions

```typescript
import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

// Get accounts
const accounts = await client.getAccounts();
const sender = accounts[0];
const receiver = accounts[1];

// Create keypair from secret key
const senderKeypair = Keypair.fromSecretKey(sender.secretKey);

// Get connection
const connection = client.getConnection();

// Create transfer transaction
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

console.log('Transaction signature:', signature);
```

### Query Account Info

```typescript
import { PublicKey } from '@solana/web3.js';

const connection = client.getConnection();

const accountInfo = await connection.getAccountInfo(
  new PublicKey(publicKey)
);

if (accountInfo) {
  console.log('Lamports:', accountInfo.lamports);
  console.log('Owner:', accountInfo.owner.toBase58());
  console.log('Data length:', accountInfo.data.length);
}
```

## Testing Integration

### Vitest Example

```typescript
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { SolanaClient } from '@chain-forge/solana';

describe('Solana Tests', () => {
  let client: SolanaClient;

  beforeAll(async () => {
    client = new SolanaClient({
      accounts: 5,
      initialBalance: 100,
    });
    await client.start();
  }, 30000); // 30s timeout for startup

  afterAll(() => {
    client.stop();
  });

  it('should have funded accounts', async () => {
    const accounts = await client.getAccounts();
    expect(accounts).toHaveLength(5);
    expect(accounts[0].balance).toBe(100);
  });

  it('should fund accounts', async () => {
    const accounts = await client.getAccounts();
    const publicKey = accounts[0].publicKey;

    await client.fundAccount(publicKey, 50);

    const balance = await client.getBalance(publicKey);
    expect(balance).toBeGreaterThan(100);
  });
});
```

### Jest Example

```typescript
import { SolanaClient } from '@chain-forge/solana';

describe('Solana Tests', () => {
  let client: SolanaClient;

  beforeAll(async () => {
    client = new SolanaClient();
    await client.start();
  }, 30000);

  afterAll(() => {
    client.stop();
  });

  test('accounts have correct balance', async () => {
    const accounts = await client.getAccounts();
    accounts.forEach(account => {
      expect(account.balance).toBe(100);
    });
  });
});
```

### Mocha Example

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { expect } from 'chai';

describe('Solana Tests', function() {
  this.timeout(30000); // 30s timeout

  let client: SolanaClient;

  before(async () => {
    client = new SolanaClient();
    await client.start();
  });

  after(() => {
    client.stop();
  });

  it('should start validator', async () => {
    const accounts = await client.getAccounts();
    expect(accounts).to.have.lengthOf(10);
  });
});
```

## Error Handling

### Startup Errors

```typescript
try {
  await client.start();
} catch (error) {
  console.error('Failed to start validator:', error);

  // Common issues:
  // - cf-solana not installed
  // - Port already in use
  // - Solana CLI not installed
}
```

### Airdrop Errors

```typescript
try {
  await client.fundAccount(publicKey, 1000);
} catch (error) {
  console.error('Airdrop failed:', error);

  // Retry after delay
  await new Promise(resolve => setTimeout(resolve, 2000));
  await client.fundAccount(publicKey, 1000);
}
```

### General Pattern

```typescript
async function safeOperation() {
  try {
    await client.start();

    const accounts = await client.getAccounts();
    await client.fundAccount(accounts[0].publicKey, 50);

    // ... your code

  } catch (error) {
    if (error instanceof Error) {
      console.error('Error:', error.message);
    }
  } finally {
    client.stop();
  }
}
```

## Best Practices

### 1. Always Stop the Validator

```typescript
try {
  await client.start();
  // ... operations
} finally {
  client.stop();  // Always cleanup
}
```

### 2. Use Appropriate Timeouts

```typescript
// Startup can take 5-10 seconds
beforeAll(async () => {
  await client.start();
}, 30000); // 30s timeout
```

### 3. Fixed Mnemonics for Tests

```typescript
const client = new SolanaClient({
  mnemonic: 'test test test test test test test test test test test junk',
});
```

This ensures:
- Deterministic addresses
- Reproducible tests
- Consistent behavior across runs

### 4. Lightweight Configuration for CI

```typescript
const client = new SolanaClient({
  accounts: 3,        // Fewer accounts
  initialBalance: 10, // Less SOL
});
```

Faster startup, less resource usage.

### 5. Share Client Across Tests

```typescript
// test-setup.ts
export let client: SolanaClient;

export async function setup() {
  client = new SolanaClient();
  await client.start();
}

export function teardown() {
  client?.stop();
}

// tests.ts
import { client } from './test-setup';

test('use shared client', async () => {
  const accounts = await client.getAccounts();
  // ...
});
```

## Common Patterns

### Reusable Test Fixture

```typescript
export class SolanaTestFixture {
  client: SolanaClient;

  async setup() {
    this.client = new SolanaClient({
      accounts: 5,
      initialBalance: 100,
    });
    await this.client.start();
  }

  async teardown() {
    this.client.stop();
  }

  async getTestAccounts() {
    return await this.client.getAccounts();
  }
}

// Usage
let fixture: SolanaTestFixture;

beforeAll(async () => {
  fixture = new SolanaTestFixture();
  await fixture.setup();
});

afterAll(async () => {
  await fixture.teardown();
});
```

### Funding Helper

```typescript
async function ensureBalance(
  client: SolanaClient,
  publicKey: string,
  minBalance: number
) {
  const current = await client.getBalance(publicKey);

  if (current < minBalance) {
    const needed = minBalance - current;
    await client.fundAccount(publicKey, needed);
  }
}

// Usage
await ensureBalance(client, account.publicKey, 500);
```

## TypeScript Configuration

Add to your `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "esModuleInterop": true,
    "skipLibCheck": true,
    "strict": true
  }
}
```

## Next Steps

- [API Reference](../api/overview)
- [Examples](../examples/typescript)
- [CLI Commands](../solana/cli)
