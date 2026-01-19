# Account Management

Understanding account generation and management in Chain Forge.

## BIP39/BIP44 Standard

Chain Forge uses industry-standard wallet protocols:

- **BIP39**: Mnemonic phrase generation (12 words)
- **BIP44**: Hierarchical deterministic derivation paths

Derivation path format:
```
m/44'/501'/index'/0'
```

Where:
- `44'` = BIP44 standard
- `501'` = Solana coin type
- `index'` = Account index (0, 1, 2, ...)
- `0'` = Change index (always 0 for Solana)

## Account Generation

Accounts are generated when you start the validator:

```bash
cf-solana start --accounts 10
```

This:
1. Generates a 12-word BIP39 mnemonic
2. Derives 10 accounts using BIP44 paths
3. Stores account data in `~/.chain-forge/solana/accounts.json`

## Wallet Compatibility

Accounts are compatible with popular Solana wallets:

- **Phantom**
- **Solflare**
- **Ledger**
- **Any BIP39/BIP44 compatible wallet**

Import using the mnemonic phrase and derivation path `m/44'/501'/index'/0'`.

## Account Storage

Accounts are stored in `~/.chain-forge/solana/accounts.json`:

```json
[
  {
    "index": 0,
    "publicKey": "7xJ5k2m8...",
    "secretKey": [/* Uint8Array */],
    "derivationPath": "m/44'/501'/0'/0'"
  }
]
```

::: danger
This file contains private keys! Never commit or share it.
:::

## Mnemonic Management

### Auto-Generated

By default, a new mnemonic is generated each time:

```bash
cf-solana start
# Displays: Generated mnemonic: word1 word2 ... word12
```

### Fixed Mnemonic

Use a specific mnemonic for reproducibility:

```bash
cf-solana start --mnemonic "your twelve word phrase here..."
```

### Test Mnemonic

Use a well-known test mnemonic:

```bash
cf-solana start --mnemonic "test test test test test test test test test test test junk"
```

::: warning
Never use test mnemonics with real funds!
:::

## Account Funding

Accounts are funded after validator startup via RPC airdrops.

### Initial Funding

Set balance when starting:

```bash
cf-solana start --balance 100
```

### Additional Funding

Add more SOL to an account:

```bash
cf-solana fund <PUBLIC_KEY> 50
```

### Set Exact Balance

Using TypeScript:

```typescript
await client.setBalance(publicKey, 500);
```

## Account Security

### Best Practices

1. **Never commit** `~/.chain-forge/` to version control
2. **Add to .gitignore**: `.chain-forge/`
3. **Use separate mnemonics** for different projects
4. **Never share** accounts with real funds
5. **Use test mnemonics** only for development

### Securing Mnemonics

For production-like testing:

1. Store mnemonics in environment variables
2. Use secret management tools (Vault, AWS Secrets Manager)
3. Encrypt sensitive files
4. Restrict file permissions

Example:

```bash
export MNEMONIC="your twelve word phrase..."
cf-solana start --mnemonic "$MNEMONIC"
```

## See Also

- [CLI Commands](./cli)
- [Configuration](./configuration)
- [Getting Started](../guide/getting-started)
