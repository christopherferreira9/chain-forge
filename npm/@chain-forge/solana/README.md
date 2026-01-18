# @chain-forge/solana

TypeScript client for Chain Forge Solana local development.

## Installation

```bash
yarn install @chain-forge/solana @solana/web3.js
```

**Prerequisites**: Install the Solana CLI binary:

```bash
cd chain-forge
cargo install --path chains/solana/crates/cli
```

## Quick Start

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { Connection, PublicKey } from '@solana/web3.js';

// Create a client
const client = new SolanaClient({
  accounts: 10,
  initialBalance: 100,
  port: 8899,
});

// Start the local validator
await client.start();

// Get generated accounts
const accounts = await client.getAccounts();
console.log(`First account: ${accounts[0].publicKey}`);

// Fund an account
await client.fundAccount(accounts[0].publicKey, 5);

// Check balance
const balance = await client.getBalance(accounts[0].publicKey);
console.log(`Balance: ${balance} SOL`);

// Use with @solana/web3.js
const connection = client.getConnection();
const version = await connection.getVersion();
console.log('Solana version:', version);

// Stop when done
client.stop();
```

## API Reference

### `SolanaClient`

Main client class for managing the local Solana validator.

#### Constructor

```typescript
new SolanaClient(config?: SolanaClientConfig)
```

**Options:**
- `accounts?: number` - Number of accounts to generate (default: 10)
- `initialBalance?: number` - Initial balance for each account in SOL (default: 100)
- `port?: number` - RPC port for the validator (default: 8899)
- `mnemonic?: string` - Optional mnemonic phrase for account generation
- `rpcUrl?: string` - RPC URL (default: `http://localhost:8899`)

#### Methods

##### `start(): Promise<void>`

Start the local Solana test validator with pre-funded accounts.

```typescript
await client.start();
```

##### `stop(): void`

Stop the running validator.

```typescript
client.stop();
```

##### `isRunning(): boolean`

Check if the validator is currently running.

```typescript
if (client.isRunning()) {
  console.log('Validator is running');
}
```

##### `getAccounts(): Promise<SolanaAccount[]>`

Get all generated accounts with their public keys and balances.

```typescript
const accounts = await client.getAccounts();
accounts.forEach((acc, i) => {
  console.log(`Account ${i}: ${acc.publicKey} (${acc.balance} SOL)`);
});
```

##### `fundAccount(address: string | PublicKey, amount: number): Promise<string>`

Request an airdrop to fund an account.

```typescript
const signature = await client.fundAccount(address, 5.0);
console.log(`Funded account: ${signature}`);
```

##### `getBalance(address: string | PublicKey): Promise<number>`

Get the current balance of an account in SOL.

```typescript
const balance = await client.getBalance(address);
console.log(`Balance: ${balance} SOL`);
```

##### `getConnection(): Connection`

Get a `Connection` instance from `@solana/web3.js` for direct blockchain interaction.

```typescript
const connection = client.getConnection();
const blockHeight = await connection.getBlockHeight();
```

##### `getRpcUrl(): string`

Get the RPC URL for the validator.

```typescript
const url = client.getRpcUrl();
console.log(`Connect to: ${url}`);
```

## Types

### `SolanaAccount`

```typescript
interface SolanaAccount {
  publicKey: string;
  secretKey: number[];
  mnemonic?: string;
  derivationPath?: string;
  balance: number;
}
```

### `SolanaClientConfig`

```typescript
interface SolanaClientConfig {
  accounts?: number;
  initialBalance?: number;
  port?: number;
  mnemonic?: string;
  rpcUrl?: string;
}
```

## Integration with Solana Web3.js

The client seamlessly integrates with `@solana/web3.js`:

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { Transaction, SystemProgram, sendAndConfirmTransaction } from '@solana/web3.js';

const client = new SolanaClient();
await client.start();

const accounts = await client.getAccounts();
const connection = client.getConnection();

// Use web3.js directly
const transaction = new Transaction().add(
  SystemProgram.transfer({
    fromPubkey: new PublicKey(accounts[0].publicKey),
    toPubkey: new PublicKey(accounts[1].publicKey),
    lamports: 1000000,
  })
);

// ... sign and send transaction
```

## Examples

### Testing Framework Integration

```typescript
import { SolanaClient } from '@chain-forge/solana';

describe('My Solana Program', () => {
  let client: SolanaClient;

  beforeAll(async () => {
    client = new SolanaClient({ accounts: 5, initialBalance: 50 });
    await client.start();
  });

  afterAll(() => {
    client.stop();
  });

  it('should transfer SOL', async () => {
    const accounts = await client.getAccounts();
    // ... test code
  });
});
```

### Custom Mnemonic

```typescript
const client = new SolanaClient({
  mnemonic: 'your twelve word mnemonic phrase goes here like this example',
  accounts: 5,
});

await client.start();

// Will generate the same accounts every time
const accounts = await client.getAccounts();
```

## License

MIT OR Apache-2.0
