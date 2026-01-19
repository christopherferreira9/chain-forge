# Program Deployment Example

This example demonstrates how to deploy Solana programs using Chain Forge.

## Prerequisites

- Solana CLI tools installed
- Chain Forge CLI (`cf-solana`) installed
- A compiled Solana program (.so file)

## Example Project

The full example is available at `examples/program-deployment/` in the repository.

## Included Sample Program

The example includes a ready-to-use sample program called **`hello_chain_forge`** located at `examples/program-deployment/program/`.

### About the Program

This is a simple demonstration program that shows basic Solana program structure:

- Processing different instruction types
- Logging messages to the validator
- Reading and modifying account data (counter)

### Supported Instructions

| Instruction | Byte | Description |
|------------|------|-------------|
| Initialize | `0` | Sets the counter to 0 in the provided account |
| Increment | `1` | Adds 1 to the counter in the provided account |
| Hello | `2` (or any other) | Logs "Hello, Chain Forge!" and account info |

### Building the Sample Program

```bash
cd examples/program-deployment

# Build the program
yarn build:program

# Output: program/target/deploy/hello_chain_forge.so
```

### Deploying and Running

```bash
# Deploy the sample program
yarn deploy:example

# Or with mnemonic for reproducible accounts
yarn deploy:example:mnemonic
```

## Quick Start

### 1. Deploy with Default Accounts

```bash
# Navigate to example
cd examples/program-deployment
yarn install

# Deploy program
yarn deploy ./path/to/your_program.so
```

### 2. Deploy with Mnemonic

For reproducible deployments:

```bash
# Uses default test mnemonic
yarn deploy:mnemonic ./path/to/your_program.so

# Use custom mnemonic
yarn deploy:mnemonic ./program.so "your twelve word mnemonic phrase"
```

## Code Examples

### Basic Deployment

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function deploy() {
  const client = new SolanaClient({
    accounts: 3,
    initialBalance: 500,
  });

  await client.start();

  const result = await client.deployProgram('./my_program.so');

  console.log('Program ID:', result.programId);
  console.log('Size:', result.programSize, 'bytes');

  client.stop();
}

deploy();
```

### Using Different Payer Accounts

```typescript
const client = new SolanaClient({
  accounts: 5,
  initialBalance: 500,
});

await client.start();

// Deploy using account at index 2
const result = await client.deployProgram('./program.so', {
  payerIndex: 2,
});

// Check which account paid
console.log('Payer:', result.payer);
```

### Reproducible Deployments with Mnemonic

```typescript
const TEST_MNEMONIC = 'test test test test test test test test test test test junk';

const client = new SolanaClient({
  accounts: 5,
  initialBalance: 500,
  mnemonic: TEST_MNEMONIC,
});

await client.start();

// Same mnemonic = same accounts = reproducible deployments
const accounts = await client.getAccounts();
console.log('Payer address (always the same):', accounts[0].publicKey);

const result = await client.deployProgram('./program.so');
```

### Interacting with Deployed Program

```typescript
import { SolanaClient } from '@chain-forge/solana';
import {
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} from '@solana/web3.js';

async function deployAndInteract() {
  const client = new SolanaClient({
    accounts: 5,
    initialBalance: 500,
  });

  await client.start();

  // Deploy
  const { programId } = await client.deployProgram('./my_program.so');
  console.log('Deployed:', programId);

  // Create instruction to call program
  const connection = client.getConnection();
  const signer = await client.getKeypair(0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: signer.publicKey, isSigner: true, isWritable: true },
    ],
    programId: new PublicKey(programId),
    data: Buffer.from([/* your instruction data */]),
  });

  // Send transaction
  const tx = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(
    connection,
    tx,
    [signer]
  );

  console.log('Transaction:', signature);

  client.stop();
}
```

## Creating a Test Program

If you don't have a program to deploy:

### Using Solana CLI

```bash
# Create a new Rust library
cargo new --lib hello_solana
cd hello_solana

# Edit Cargo.toml to add:
# [lib]
# crate-type = ["cdylib", "lib"]
#
# [dependencies]
# solana-program = "2.0"

# Build with Solana BPF toolchain
cargo build-sbf

# Output: target/deploy/hello_solana.so
```

### Using Anchor

```bash
anchor init my_project
cd my_project
anchor build

# Output: target/deploy/my_project.so
```

## API Reference

### deployProgram()

```typescript
async deployProgram(
  programPath: string,
  options?: DeployProgramOptions
): Promise<DeployProgramResult>
```

**Options:**
```typescript
interface DeployProgramOptions {
  payerIndex?: number;       // Account index for payer (default: 0)
  programKeypair?: Uint8Array; // Optional deterministic program keypair
}
```

**Result:**
```typescript
interface DeployProgramResult {
  programId: string;    // Deployed program's public key
  signature: string;    // Transaction signature
  payer: string;        // Payer account's public key
  programSize: number;  // Program size in bytes
}
```

### getKeypair()

```typescript
async getKeypair(accountIndex?: number): Promise<Keypair>
```

Returns a `Keypair` for signing transactions.

## Tips

1. **Sufficient Balance**: Use `initialBalance: 500` or higher for program deployment
2. **Mnemonic for CI/CD**: Use a fixed mnemonic for reproducible deployments
3. **Program Size**: Larger programs require more SOL for rent exemption
4. **Multiple Programs**: Deploy multiple programs using different `payerIndex` values

## See Also

- [Program Deployment Guide](../solana/program-deployment)
- [API Reference](../api/overview)
- [TypeScript Examples](./typescript)
