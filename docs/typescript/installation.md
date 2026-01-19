# TypeScript Installation

Install and configure the `@chain-forge/solana` TypeScript package.

## Prerequisites

- **Node.js** 18 or later
- **Chain Forge CLI** (`cf-solana`)
- **Solana CLI Tools**

## Installation

```bash
npm install @chain-forge/solana @solana/web3.js
```

## Verify Installation

```typescript
import { SolanaClient } from '@chain-forge/solana';

const client = new SolanaClient();
await client.start();
console.log('Success!');
client.stop();
```

## Next Steps

- [Basic Usage](./basic-usage)
- [API Reference](../api/overview)
