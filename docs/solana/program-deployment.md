# Program Deployment

Chain Forge provides native support for deploying Solana programs directly from TypeScript.

## Overview

The `deployProgram` method allows you to:

- Deploy compiled Solana programs (.so files)
- Use any generated account as the deployment payer
- Get the deployed program ID for immediate interaction
- Track deployment costs and transaction signatures

## Basic Deployment

```typescript
import { SolanaClient } from '@chain-forge/solana';

const client = new SolanaClient({
  accounts: 3,
  initialBalance: 500, // Ensure enough SOL for deployment
});

await client.start();

// Deploy program
const result = await client.deployProgram('./target/deploy/my_program.so');

console.log('Program ID:', result.programId);
console.log('Signature:', result.signature);
console.log('Size:', result.programSize, 'bytes');
```

## Deployment Options

### Using Different Payer Accounts

```typescript
// Use account at index 2 as payer
const result = await client.deployProgram('./program.so', {
  payerIndex: 2,
});
```

### Deterministic Program IDs

Provide a program keypair for reproducible deployments:

```typescript
import { Keypair } from '@solana/web3.js';

const programKeypair = Keypair.generate();

const result = await client.deployProgram('./program.so', {
  programKeypair: programKeypair.secretKey,
});

// Program ID will always be programKeypair.publicKey
```

## Interacting with Deployed Programs

After deployment, use the program ID to interact with your program:

```typescript
import { PublicKey, TransactionInstruction, Transaction } from '@solana/web3.js';

// Deploy
const { programId } = await client.deployProgram('./my_program.so');

// Create program instruction
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: userAccount, isSigner: true, isWritable: true },
  ],
  programId: new PublicKey(programId),
  data: Buffer.from([/* instruction data */]),
});

// Get connection and signer
const connection = client.getConnection();
const signer = await client.getKeypair(0);

// Send transaction
const transaction = new Transaction().add(instruction);
const signature = await sendAndConfirmTransaction(
  connection,
  transaction,
  [signer]
);
```

## Cost Considerations

Program deployment costs depend on:

1. **Program Size**: Larger programs require more lamports for rent exemption
2. **Transaction Fees**: Multiple transactions may be needed for large programs

Recommended initial balance:

| Program Size | Recommended Balance |
|-------------|---------------------|
| < 100 KB | 100 SOL |
| 100-500 KB | 250 SOL |
| > 500 KB | 500+ SOL |

## Working with Anchor Programs

Chain Forge works seamlessly with Anchor:

```bash
# Build your Anchor program
anchor build

# The compiled program is at:
# ./target/deploy/your_program.so
```

```typescript
const client = new SolanaClient({ initialBalance: 500 });
await client.start();

// Deploy Anchor program
const result = await client.deployProgram(
  './target/deploy/your_program.so'
);

// Use with Anchor client
import { Program, AnchorProvider } from '@coral-xyz/anchor';

const provider = new AnchorProvider(
  client.getConnection(),
  wallet,
  { commitment: 'confirmed' }
);

const program = new Program(idl, result.programId, provider);
```

## CI/CD Integration

For reproducible deployments in CI/CD:

```typescript
const client = new SolanaClient({
  accounts: 3,
  initialBalance: 500,
  // Use fixed mnemonic for deterministic accounts
  mnemonic: process.env.TEST_MNEMONIC,
});

await client.start();

// Deploy will use the same payer account every time
const result = await client.deployProgram('./program.so');
```

## Error Handling

```typescript
try {
  const result = await client.deployProgram('./program.so');
  console.log('Deployed:', result.programId);
} catch (error) {
  if (error.message.includes('insufficient funds')) {
    console.error('Payer account needs more SOL');
  } else if (error.message.includes('ENOENT')) {
    console.error('Program file not found');
  } else {
    console.error('Deployment failed:', error);
  }
}
```

## Complete Example

```typescript
import { SolanaClient } from '@chain-forge/solana';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';

async function main() {
  // Setup
  const client = new SolanaClient({
    accounts: 5,
    initialBalance: 500,
  });

  await client.start();
  console.log('Validator started');

  // Get accounts
  const accounts = await client.getAccounts();
  console.log(`Using account: ${accounts[0].publicKey}`);

  // Deploy
  const result = await client.deployProgram('./target/deploy/my_program.so');
  console.log(`Program deployed: ${result.programId}`);

  // Check deployment cost
  const balanceAfter = await client.getBalance(accounts[0].publicKey);
  console.log(`Deployment cost: ${500 - balanceAfter} SOL`);

  // Interact with program
  const programId = new PublicKey(result.programId);
  const connection = client.getConnection();
  const signer = await client.getKeypair(0);

  // Your program interaction logic here...

  // Cleanup
  client.stop();
}

main().catch(console.error);
```

## See Also

- [API Reference](../api/overview)
- [TypeScript Examples](../examples/typescript)
- [Account Management](./accounts)
