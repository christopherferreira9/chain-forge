# Account Management

Understanding account generation and management in Chain Forge Bitcoin.

::: warning Development Status
Bitcoin support is currently in active development. Some features may change or be incomplete.
:::

## BIP39/BIP44 Standard

Chain Forge uses industry-standard wallet protocols:

- **BIP39**: Mnemonic phrase generation (12 words)
- **BIP44**: Hierarchical deterministic derivation paths

Derivation path format:
```
m/44'/0'/0'/0/index
```

Where:
- `44'` = BIP44 standard
- `0'` = Bitcoin coin type
- `0'` = Account index (always 0)
- `0` = External chain
- `index` = Address index (0, 1, 2, ...)

## Account Generation

Accounts are generated when you start the node:

```bash
cf-bitcoin start --accounts 10
```

This:
1. Generates a 12-word BIP39 mnemonic
2. Derives 10 accounts using BIP44 paths
3. Creates P2WPKH (native SegWit) addresses
4. Stores account data in the instance directory

## Address Format

Chain Forge generates **P2WPKH** (Pay to Witness Public Key Hash) addresses:

- **Regtest prefix**: `bcrt1q...`
- **Mainnet equivalent**: `bc1q...`
- **Testnet equivalent**: `tb1q...`

Example regtest address:
```
bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080
```

## Wallet Compatibility

Accounts are compatible with popular Bitcoin wallets:

- **Electrum**
- **Ledger**
- **Trezor**
- **Any BIP39/BIP44 compatible wallet**

Import using the mnemonic phrase and derivation path `m/44'/0'/0'/0/index`.

## Account Storage

Accounts are stored per-instance:

```
~/.chain-forge/bitcoin/instances/<instance-id>/accounts.json
```

Example structure:

```json
[
  {
    "address": "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080",
    "publicKey": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    "privateKey": [/* 32 bytes */],
    "wif": "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy",
    "mnemonic": "test test test test test test test test test test test junk",
    "derivationPath": "m/44'/0'/0'/0/0",
    "balance": 10.0
  }
]
```

::: danger
This file contains private keys! Never commit or share it.
:::

## Instance Isolation

Each instance has its own accounts and blockchain:

```bash
# Two instances with same mnemonic = same addresses, different balances
cf-bitcoin start --instance node1 --mnemonic "test test..."
cf-bitcoin start --instance node2 --mnemonic "test test..." --rpc-port 18445
```

Instance data is stored separately:

```
~/.chain-forge/bitcoin/instances/
├── node1/
│   ├── accounts.json
│   ├── instance.json
│   └── regtest-data/
└── node2/
    ├── accounts.json
    ├── instance.json
    └── regtest-data/
```

## Mnemonic Management

### Auto-Generated

By default, a new mnemonic is generated each time:

```bash
cf-bitcoin start
# Displays: 🔑 Mnemonic: word1 word2 ... word12
```

### Fixed Mnemonic

Use a specific mnemonic for reproducibility:

```bash
cf-bitcoin start --mnemonic "your twelve word phrase here..."
```

### Test Mnemonic

Use a well-known test mnemonic:

```bash
cf-bitcoin start --mnemonic "test test test test test test test test test test test junk"
```

::: warning
Never use test mnemonics with real funds!
:::

## Account Funding

### Initial Funding

Accounts are funded during node startup:

1. Mine blocks to generate coinbase rewards (50 BTC each)
2. Wait for 100 confirmations (coinbase maturity)
3. Send target balance to each account
4. Mine confirmation blocks

Set balance when starting:

```bash
cf-bitcoin start --balance 100
```

### Additional Funding

Add more BTC to an account from wallet funds:

```bash
cf-bitcoin fund <ADDRESS> 50
```

### Transfer Between Accounts

Transfer BTC from one account to another:

```bash
cf-bitcoin transfer <FROM_ADDRESS> <TO_ADDRESS> 10
```

This creates a real Bitcoin transaction:
- Uses UTXOs from the source address
- Pays transaction fee (~0.0001 BTC)
- Returns change to source address

## Balance Tracking

### UTXO-Based Balances

Bitcoin uses UTXOs (Unspent Transaction Outputs), not account balances:

```
Account "balance" = Sum of all UTXOs owned by that address
```

Chain Forge queries the UTXO set directly:

```bash
# Refresh balances from blockchain
cf-bitcoin accounts
```

### Cached vs Live Balances

- `accounts.json` stores cached balances
- `cf-bitcoin accounts` queries the blockchain and updates cache
- Always refresh after transactions

### TypeScript Balance Refresh

```typescript
// Get cached balances (fast, may be stale)
const accounts = await client.getAccounts();

// Refresh from blockchain (accurate)
const updated = await client.refreshBalances();
```

## Private Key Formats

Accounts include multiple key formats:

### Raw Private Key
32 bytes of entropy:
```json
"privateKey": [1, 2, 3, ..., 32]
```

### WIF (Wallet Import Format)
Base58Check encoded for wallet import:
```json
"wif": "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy"
```

The `c` prefix indicates regtest/testnet compressed key.

## Account Security

### Best Practices

1. **Never commit** `~/.chain-forge/` to version control
2. **Add to .gitignore**: `.chain-forge/`
3. **Use separate instances** for different projects
4. **Never share** accounts with real funds
5. **Use test mnemonics** only for development

### Securing Mnemonics

For production-like testing:

1. Store mnemonics in environment variables
2. Use secret management tools
3. Encrypt sensitive files
4. Restrict file permissions

Example:

```bash
export MNEMONIC="your twelve word phrase..."
cf-bitcoin start --mnemonic "$MNEMONIC"
```

## Importing to External Wallets

To use Chain Forge accounts in external wallets:

1. Get the mnemonic from startup output
2. Import into wallet using BIP44 derivation
3. Set derivation path to `m/44'/0'/0'/0/index`
4. Use regtest network settings

::: warning
External wallets may need regtest configuration to recognize `bcrt1` addresses.
:::

## See Also

- [CLI Commands](./cli)
- [Configuration](./configuration)
- [Overview](./overview)
