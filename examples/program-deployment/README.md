# Program Deployment Example

This example demonstrates how to deploy Solana programs using Chain Forge's `SolanaClient`.

## Included Example Program

This example includes a simple Solana program (`hello_chain_forge`) in the `program/` directory that you can build and deploy.

### Program Features

The included program supports three instructions:

- **Initialize (0)**: Set a counter to 0
- **Increment (1)**: Add 1 to the counter
- **Hello (2/default)**: Log "Hello, Chain Forge!" and account info

## Prerequisites

1. **Solana CLI Tools** with BPF toolchain:
   ```bash
   sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
   ```

2. **Chain Forge CLI** (`cf-solana`):
   ```bash
   cargo install --path ../../chains/solana/crates/cli
   ```

## Quick Start

### 1. Install Dependencies

```bash
yarn install
```

### 2. Build the Example Program

```bash
yarn build:program
```

This compiles the Solana program to `./program/target/deploy/hello_chain_forge.so`.

### 3. Deploy the Program

**Using default accounts:**
```bash
yarn deploy:example
```

**Using mnemonic for reproducible accounts:**
```bash
yarn deploy:example:mnemonic
```

## Manual Deployment

You can also deploy any Solana program:

```bash
# Deploy any .so file with default accounts
yarn deploy ./path/to/your_program.so

# Deploy with mnemonic
yarn deploy:mnemonic ./path/to/your_program.so

# Deploy with custom mnemonic
yarn deploy:mnemonic ./program.so "your twelve word mnemonic phrase"
```

## Scripts

| Script | Description |
|--------|-------------|
| `yarn build:program` | Build the example Solana program |
| `yarn deploy:example` | Deploy example program with default accounts |
| `yarn deploy:example:mnemonic` | Deploy example program with test mnemonic |
| `yarn deploy <path>` | Deploy any program with default accounts |
| `yarn deploy:mnemonic <path>` | Deploy any program with mnemonic |
| `yarn dev [path]` | Interactive deployment demo |

## Code Example

```typescript
import { SolanaClient } from '@chain-forge/solana';

async function main() {
  const client = new SolanaClient({
    accounts: 3,
    initialBalance: 500,
  });

  await client.start();

  // Deploy the example program
  const result = await client.deployProgram(
    './program/target/deploy/hello_chain_forge.so'
  );

  console.log('Program ID:', result.programId);
  console.log('Deployment cost:', result.programSize, 'bytes');

  // Get keypair for interactions
  const signer = await client.getKeypair(0);
  console.log('Signer:', signer.publicKey.toBase58());

  client.stop();
}

main();
```

## Interacting with the Deployed Program

After deployment, you can call the program:

```typescript
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';

// Call the "hello" instruction (instruction byte = 2)
const instruction = new TransactionInstruction({
  keys: [],
  programId: new PublicKey(result.programId),
  data: Buffer.from([2]), // Hello instruction
});

const connection = client.getConnection();
const signer = await client.getKeypair(0);

const tx = new Transaction().add(instruction);
const signature = await sendAndConfirmTransaction(connection, tx, [signer]);
```

## Program Structure

```
program/
├── Cargo.toml          # Rust dependencies
└── src/
    └── lib.rs          # Program code
```

The program is a standard Solana BPF program using `solana-program` crate.

## Troubleshooting

### "cargo build-sbf: command not found"

Install the Solana BPF toolchain:
```bash
solana-install update
```

### "insufficient funds"

Increase the initial balance:
```typescript
const client = new SolanaClient({
  initialBalance: 1000, // More SOL
});
```

### Build errors

Ensure you have the latest Solana tools:
```bash
solana-install update
rustup update
```
