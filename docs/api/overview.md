# API Reference

TypeScript API reference for `@chain-forge/solana`.

## SolanaClient

Main class for interacting with a local Solana test validator.

### Constructor

```typescript
new SolanaClient(options?: SolanaClientOptions)
```

#### SolanaClientOptions

```typescript
interface SolanaClientOptions {
  accounts?: number;           // Number of accounts to generate (default: 10)
  initialBalance?: number;     // Initial SOL balance per account (default: 100)
  port?: number;               // RPC port (default: 8899)
  mnemonic?: string;           // BIP39 mnemonic phrase (12 words)
  rpcUrl?: string;             // Custom RPC URL (default: http://localhost:{port})
}
```

#### Example

```typescript
import { SolanaClient } from '@chain-forge/solana';

// Default options
const client = new SolanaClient();

// Custom options
const client = new SolanaClient({
  accounts: 20,
  initialBalance: 500,
  port: 8900,
  mnemonic: 'your twelve word mnemonic phrase here...'
});
```

## Instance Methods

### start()

Start the Solana test validator.

```typescript
async start(): Promise<void>
```

Returns a promise that resolves when the validator is ready.

#### Example

```typescript
await client.start();
console.log('Validator is running');
```

#### Throws

- `Error` if validator fails to start
- `Error` if validator is already running
- `Error` if Solana CLI tools are not installed

### stop()

Stop the Solana test validator.

```typescript
stop(): void
```

Terminates the validator process. Synchronous operation.

#### Example

```typescript
client.stop();
console.log('Validator stopped');
```

### getAccounts()

Get all generated accounts.

```typescript
async getAccounts(): Promise<Account[]>
```

Returns an array of `Account` objects.

#### Account Interface

```typescript
interface Account {
  index: number;           // Account index (0-based)
  publicKey: string;       // Base58-encoded public key
  secretKey: Uint8Array;   // Secret key bytes
  balance: number;         // Current balance in SOL
}
```

#### Example

```typescript
const accounts = await client.getAccounts();

accounts.forEach(account => {
  console.log(`Account ${account.index}: ${account.publicKey}`);
  console.log(`Balance: ${account.balance} SOL`);
});
```

### getBalance()

Get the SOL balance of an account.

```typescript
async getBalance(publicKey: string): Promise<number>
```

#### Parameters

- `publicKey` - Base58-encoded public key

#### Returns

Balance in SOL (not lamports)

#### Example

```typescript
const balance = await client.getBalance('7xJ5k2m8...');
console.log(`Balance: ${balance} SOL`);
```

### fundAccount()

Request an airdrop to fund an account.

```typescript
async fundAccount(publicKey: string, amount: number): Promise<void>
```

#### Parameters

- `publicKey` - Base58-encoded public key
- `amount` - Amount of SOL to airdrop

#### Example

```typescript
await client.fundAccount('7xJ5k2m8...', 50);
console.log('Funded account with 50 SOL');
```

#### Notes

- Subject to rate limiting
- May take 1-2 seconds to complete
- Will throw if airdrop fails

### setBalance()

Set an account's balance to a specific amount.

```typescript
async setBalance(publicKey: string, targetBalance: number): Promise<void>
```

Calculates the difference between current and target balance, then airdrops the difference.

#### Parameters

- `publicKey` - Base58-encoded public key
- `targetBalance` - Target balance in SOL

#### Example

```typescript
// Set balance to exactly 200 SOL
await client.setBalance('7xJ5k2m8...', 200);
```

#### Notes

- Cannot reduce balance (Solana limitation)
- Will throw if target is less than current balance
- Internally uses `fundAccount()`

### getConnection()

Get the Solana Web3.js Connection object.

```typescript
getConnection(): Connection
```

Returns the underlying `Connection` from `@solana/web3.js` for advanced usage.

#### Example

```typescript
import { PublicKey } from '@solana/web3.js';

const connection = client.getConnection();

// Use Connection methods directly
const accountInfo = await connection.getAccountInfo(
  new PublicKey('7xJ5k2m8...')
);
```

### getRpcUrl()

Get the RPC URL of the running validator.

```typescript
getRpcUrl(): string
```

Returns the HTTP URL for the RPC endpoint.

#### Example

```typescript
const url = client.getRpcUrl();
console.log(`Connect to: ${url}`);  // http://localhost:8899
```

### getMnemonic()

Get the mnemonic phrase used to generate accounts.

```typescript
getMnemonic(): string | undefined
```

Returns the 12-word BIP39 mnemonic, or `undefined` if not available.

#### Example

```typescript
const mnemonic = client.getMnemonic();
if (mnemonic) {
  console.log('Save this mnemonic:', mnemonic);
}
```

::: danger
Never log or expose mnemonics in production applications!
:::

## Complete Example

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { PublicKey, Transaction, SystemProgram } from '@solana/web3.js';

async function main() {
  // Create and start validator
  const client = new SolanaClient({
    accounts: 5,
    initialBalance: 100,
  });

  await client.start();
  console.log('Validator started at', client.getRpcUrl());

  // Get accounts
  const accounts = await client.getAccounts();
  console.log(`Generated ${accounts.length} accounts`);

  const sender = accounts[0];
  const receiver = accounts[1];

  // Check initial balances
  console.log(`Sender balance: ${sender.balance} SOL`);
  console.log(`Receiver balance: ${receiver.balance} SOL`);

  // Get Connection for web3.js operations
  const connection = client.getConnection();

  // Create a transfer transaction
  const transaction = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: new PublicKey(sender.publicKey),
      toPubkey: new PublicKey(receiver.publicKey),
      lamports: 1_000_000_000, // 1 SOL
    })
  );

  // Note: Signing and sending requires additional setup
  // This example shows the integration point

  // Fund additional SOL if needed
  await client.fundAccount(sender.publicKey, 50);
  console.log('Added 50 SOL to sender');

  // Check new balance
  const newBalance = await client.getBalance(sender.publicKey);
  console.log(`Sender new balance: ${newBalance} SOL`);

  // Cleanup
  client.stop();
}

main().catch(console.error);
```

## TypeScript Types

All types are exported from the main package:

```typescript
import {
  SolanaClient,
  SolanaClientOptions,
  Account,
} from '@chain-forge/solana';
```

## Error Handling

All async methods can throw errors:

```typescript
try {
  await client.start();
} catch (error) {
  if (error instanceof Error) {
    console.error('Failed to start:', error.message);
  }
}
```

Common errors:

- Validator already running
- Port already in use
- Solana CLI not installed
- RPC connection failed
- Airdrop rate limit exceeded

## Next Steps

- [Basic Usage Guide](../typescript/basic-usage)
- [TypeScript Examples](../examples/typescript)
- [Solana CLI Reference](../solana/cli)
