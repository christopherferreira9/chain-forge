# Chain Forge - Simple Demo

A minimal, easy-to-read example demonstrating Chain Forge's core functionality.

## What This Demo Shows

1. **Starting a validator** with pre-funded accounts (like Foundry/Anvil)
2. **Account generation** with BIP39/BIP44 derivation
3. **setBalance pattern** for predictable balance management
4. **Idempotent operations** - calling setBalance multiple times safely
5. **Integration with @solana/web3.js** for blockchain queries

## Prerequisites

1. **Solana CLI Tools**
   ```bash
   sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
   ```

2. **Chain Forge CLI**
   ```bash
   cd ../..
   cargo install --path chains/solana/crates/cli
   ```

3. **TypeScript Package**
   ```bash
   cd npm/@chain-forge/solana
   yarn install
   yarn build
   ```

## Quick Start

```bash
# Install dependencies
yarn install

# Run the demo
yarn start
```

## Expected Output

```
🎯 Chain Forge - Simple Demo
============================

📦 Configuration:
   - Accounts: 3
   - Initial Balance: 100 SOL
   - Port: 8899

🚀 Starting validator...
✅ Validator started!

📋 Generated Accounts:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Account 0:
  Address: 7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR
  Balance: 100.00 SOL
  Path: m/44'/501'/0'/0'

Account 1:
  Address: 8kL2EwQlX4s5c7Y0Ro9ZnN2xM3yO6dU5bC4tN0iS
  Balance: 100.00 SOL
  Path: m/44'/501'/1'/0'

Account 2:
  Address: 9mN3FyRmY5t6d8A1Sp0aOq3zN4xP7eV6cD5uO1jT
  Balance: 100.00 SOL
  Path: m/44'/501'/2'/0'

🔧 Testing setBalance...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Setting account 0 to 200 SOL...
✅ Result: Added 100 SOL (100 → 200 SOL)

Current balance: 200.00 SOL

Setting same account to 200 SOL again (should be idempotent)...
✅ Result: Balance already at 200 SOL (target: 200 SOL)

📊 Blockchain Info:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Block Height: 42
✅ Slot: 45
✅ Version: 2.0.15

🎉 All tests passed!

Press Ctrl+C to stop the validator and exit...
```

## Code Walkthrough

The demo is intentionally simple and well-commented. Here's what it does:

### 1. Configuration (Lines 13-17)

```javascript
const CONFIG = {
  accounts: 3,           // Generate 3 accounts
  initialBalance: 100,   // Each starts with 100 SOL
  port: 8899            // Default Solana port
};
```

### 2. Create Client (Lines 35-39)

```javascript
const client = new SolanaClient({
  accounts: CONFIG.accounts,
  initialBalance: CONFIG.initialBalance,
  port: CONFIG.port
});
```

This is similar to Foundry/Anvil:
```javascript
const anvil = await Anvil.new({
  accounts: 3,
  balance: 100
});
```

### 3. Start Validator (Lines 44-46)

```javascript
await client.start();
```

This:
- Spawns `solana-test-validator`
- Generates accounts from a mnemonic
- Funds all accounts to target balance
- Waits for validator to be ready

### 4. Get Accounts (Line 53)

```javascript
const accounts = await client.getAccounts();
```

Returns an array of accounts with:
- `publicKey` - Base58 address
- `secretKey` - Private key bytes
- `balance` - Current balance in SOL
- `derivationPath` - BIP44 path (m/44'/501'/n'/0')

### 5. Set Balance (Lines 69-71)

```javascript
await client.setBalance(testAccount.publicKey, 200);
```

This adjusts the account to have exactly 200 SOL:
- If current < 200: Airdrops the difference
- If current >= 200: Does nothing

### 6. Use Web3.js (Lines 91-99)

```javascript
const connection = client.getConnection();
const blockHeight = await connection.getBlockHeight();
const slot = await connection.getSlot();
```

Chain Forge integrates seamlessly with `@solana/web3.js`.

## Key Concepts Demonstrated

### Foundry/Anvil Pattern

Chain Forge follows the same pattern as Foundry:

**Foundry (Ethereum):**
```javascript
const anvil = await Anvil.new({ accounts: 10, balance: 100 });
await anvil.setBalance(address, ethers.parseEther("200"));
```

**Chain Forge (Solana):**
```javascript
const client = new SolanaClient({ accounts: 10, initialBalance: 100 });
await client.setBalance(address, 200);
```

### Idempotent Balance Setting

```javascript
// First call: Adds 100 SOL (100 → 200)
await client.setBalance(address, 200);

// Second call: Does nothing (already 200)
await client.setBalance(address, 200);

// Third call: Still does nothing
await client.setBalance(address, 200);
```

This makes testing predictable - you always know the exact balance.

### Automatic Funding

Unlike manual funding:

```javascript
// Old pattern: Manual funding (unpredictable)
const client = new SolanaClient({ accounts: 10 });
await client.start();
for (const account of await client.getAccounts()) {
  await client.fundAccount(account.publicKey, 100);
}

// New pattern: Automatic funding (predictable)
const client = new SolanaClient({
  accounts: 10,
  initialBalance: 100  // All accounts auto-funded on start
});
await client.start();
// Accounts already have 100 SOL!
```

## Customization

### Change Number of Accounts

```javascript
const CONFIG = {
  accounts: 10,  // Change this
  initialBalance: 100,
  port: 8899
};
```

### Change Initial Balance

```javascript
const CONFIG = {
  accounts: 3,
  initialBalance: 500,  // Start with 500 SOL each
  port: 8899
};
```

### Use Custom Port

```javascript
const CONFIG = {
  accounts: 3,
  initialBalance: 100,
  port: 9000  // Use port 9000 instead
};
```

### Use Specific Mnemonic

```javascript
const client = new SolanaClient({
  accounts: 3,
  initialBalance: 100,
  mnemonic: 'test test test test test test test test test test test junk'
  // Will always generate the same accounts
});
```

## Next Steps

1. **Modify the code** - Try changing the config values
2. **Add your own tests** - Query account data, send transactions
3. **Integrate with your app** - Use this as a template
4. **Read the docs** - See `../../docs/GETTING_STARTED.md`

## Troubleshooting

### Error: `solana-test-validator: command not found`

Install Solana CLI tools:
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### Error: `Cannot find module '@chain-forge/solana'`

Build the TypeScript package:
```bash
cd ../../npm/@chain-forge/solana
yarn install
yarn build
```

### Error: Port 8899 already in use

Kill existing validator or use different port:
```bash
# Kill existing
pkill solana-test-validator

# Or use different port
const CONFIG = { ..., port: 9000 };
```

## Comparison to typescript-basic Example

| Feature | simple-demo | typescript-basic |
|---------|-------------|------------------|
| Language | JavaScript | TypeScript |
| Complexity | Minimal | More examples |
| Use Case | Quick start | Comprehensive demo |
| Comments | Heavily commented | Less verbose |
| Examples | 1 (setBalance) | 4 (various operations) |

## License

MIT OR Apache-2.0
