# Chain Forge - Basic TypeScript Example

This example demonstrates basic usage of Chain Forge with TypeScript and Solana.

## Prerequisites

1. Install Solana CLI tools
2. Install the Chain Forge CLI:
   ```bash
   cd ../..
   cargo install --path chains/solana/crates/cli
   ```

## Setup

```bash
yarn install
```

## Run

```bash
yarn dev
```

Or build and run:

```bash
yarn build
npm start
```

## What This Example Shows

1. **Starting a local validator** - Programmatically start a Solana test validator
2. **Getting accounts** - Retrieve generated accounts with their keys and balances
3. **Funding accounts** - Request airdrops to accounts
4. **Using Solana Web3.js** - Direct integration with the official Solana SDK
5. **Querying balances** - Check account balances
6. **Getting cluster info** - Retrieve blockchain metadata

## Code Structure

- `src/index.ts` - Main example file with 4 different examples
- `package.json` - Dependencies and scripts
- `tsconfig.json` - TypeScript configuration

## Learn More

- [Chain Forge Documentation](../../docs/GETTING_STARTED.md)
- [Solana Web3.js Documentation](https://solana-labs.github.io/solana-web3.js/)
- [@chain-forge/solana Package](../../npm/@chain-forge/solana/README.md)
