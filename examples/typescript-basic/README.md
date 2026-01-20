# Chain Forge - Basic TypeScript Example

A minimal CI-friendly example demonstrating Chain Forge with TypeScript.

## What This Example Shows

1. Starting a local Solana test validator
2. Getting pre-funded accounts
3. Transferring SOL between accounts

This example runs to completion without user interaction, making it suitable for CI pipelines.

## Prerequisites

1. Install Solana CLI tools
2. Build Chain Forge:
   ```bash
   cd ../..
   cargo build --workspace --release
   ```

## Setup

```bash
yarn install
```

## Run

Build and run:

```bash
yarn build
yarn start
```

Or use development mode:

```bash
yarn dev
```

## For Interactive Exploration

For a full-featured interactive CLI experience with program deployment, fund transfers, and more, see the [interactive-cli](../interactive-cli/) example.

## Learn More

- [Interactive CLI Example](../interactive-cli/)
- [Chain Forge Documentation](../../docs/)
- [@chain-forge/solana Package](../../npm/@chain-forge/solana/)
