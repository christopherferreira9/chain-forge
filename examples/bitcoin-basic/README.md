# Chain Forge - Bitcoin Basic Example

A minimal example demonstrating Chain Forge with Bitcoin regtest.

## What This Example Shows

1. Starting a local Bitcoin regtest node
2. Getting pre-funded accounts with BIP44 derivation
3. Transferring BTC between accounts
4. Mining blocks to confirm transactions

This example runs to completion without user interaction, making it suitable for testing.

## Prerequisites

1. Install Bitcoin Core (bitcoind)
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

## Learn More

- [Bitcoin Dependencies](../../docs/BITCOIN_DEPENDENCIES.md)
- [Chain Forge Documentation](../../docs/)
- [@chain-forge/bitcoin Package](../../npm/@chain-forge/bitcoin/)
