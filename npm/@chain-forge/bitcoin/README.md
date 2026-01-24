# @chain-forge/bitcoin

TypeScript client for Chain Forge Bitcoin local development.

## Installation

```bash
yarn add @chain-forge/bitcoin bitcoinjs-lib
```

## Prerequisites

- Bitcoin Core must be installed (`bitcoind`)
- The `cf-bitcoin` CLI must be built and available

## Quick Start

```typescript
import { BitcoinClient } from '@chain-forge/bitcoin';

async function main() {
  // Create client with custom configuration
  const client = new BitcoinClient({
    accounts: 5,
    initialBalance: 10, // BTC
    rpcPort: 18443,
  });

  try {
    // Start the Bitcoin regtest node
    await client.start();
    console.log('Bitcoin node started!');

    // Get generated accounts
    const accounts = await client.getAccounts();
    console.log(`Generated ${accounts.length} accounts`);

    for (const account of accounts) {
      console.log(`  ${account.address}: ${account.balance} BTC`);
    }

    // Send BTC between accounts
    const result = await client.sendToAddress(accounts[1].address, 1);
    console.log(`Sent 1 BTC, txid: ${result.txid}`);

    // Mine a block to confirm
    const mineResult = await client.mine(1);
    console.log(`Mined block: ${mineResult.blockHashes[0]}`);

  } finally {
    // Always stop the node
    client.stop();
  }
}

main().catch(console.error);
```

## API

### `BitcoinClient`

#### Constructor Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `accounts` | `number` | `10` | Number of accounts to generate |
| `initialBalance` | `number` | `10` | Initial balance for each account in BTC |
| `rpcPort` | `number` | `18443` | RPC port for the node |
| `p2pPort` | `number` | `18444` | P2P network port |
| `mnemonic` | `string` | - | Optional mnemonic for deterministic accounts |
| `rpcUser` | `string` | `chainforge` | RPC username |
| `rpcPassword` | `string` | `chainforge` | RPC password |

#### Methods

- `start()`: Start the Bitcoin regtest node
- `stop()`: Stop the node
- `isRunning()`: Check if the node is running
- `getAccounts()`: Get all generated accounts
- `setBalance(address, amount)`: Set an account's balance
- `sendToAddress(address, amount)`: Send BTC to an address
- `getBalance(address)`: Get an account's balance
- `mine(blocks, address?)`: Mine blocks
- `getRpcUrl()`: Get the RPC URL
- `getRpcCredentials()`: Get RPC credentials

### Types

- `BitcoinAccount`: Account information (address, publicKey, privateKey, wif, balance, etc.)
- `BitcoinClientConfig`: Client configuration options
- `SendResult`: Result of sending BTC
- `MineResult`: Result of mining blocks

## License

MIT
