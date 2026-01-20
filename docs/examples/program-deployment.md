# Program Deployment

This guide covers deploying Solana programs using Chain Forge.

## Interactive Deployment

For interactive program deployment with a CLI interface, use the **Interactive CLI** example:

```bash
cd examples/interactive-cli
yarn install
yarn build
yarn start
```

The Interactive CLI provides:
- Program discovery from the `programs/` directory
- Automatic building with `cargo build-sbf`
- Payer account selection
- Real-time deployment status

See the [Interactive CLI documentation](./interactive-cli) for full details.

## Programmatic Deployment

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

  // Create instruction
  const connection = client.getConnection();
  const signer = await client.getKeypair(0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: signer.publicKey, isSigner: true, isWritable: true },
    ],
    programId: new PublicKey(programId),
    data: Buffer.from([/* instruction data */]),
  });

  // Send transaction
  const tx = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(connection, tx, [signer]);

  console.log('Transaction:', signature);

  client.stop();
}
```

## Creating a Solana Program

### Using Cargo

```bash
# Create a new Rust library
mkdir -p my_program/src
cd my_program
```

Create `Cargo.toml`:

```toml
[package]
name = "my_program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "2.0"
```

Create `src/lib.rs`:

```rust
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello from my program!");
    msg!("Program ID: {}", program_id);
    Ok(())
}
```

Build with:

```bash
cargo build-sbf
# Output: target/deploy/my_program.so
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
  payerIndex?: number;         // Account index for payer (default: 0)
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

## See Also

- [Interactive CLI](./interactive-cli)
- [TypeScript Examples](./typescript)
- [API Reference](../api/overview)
