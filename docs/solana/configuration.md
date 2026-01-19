# Configuration

Detailed configuration guide for Chain Forge Solana.

## Configuration Files

Chain Forge reads configuration from multiple locations with the following priority:

1. **CLI arguments** (highest priority)
2. **Project config**: `./chain-forge.toml`
3. **Global config**: `~/.chain-forge/config.toml`
4. **Built-in defaults** (lowest priority)

## Project Configuration

Create `chain-forge.toml` in your project root:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.ci]
accounts = 3
initial_balance = 10.0

[solana.development]
accounts = 50
initial_balance = 1000.0
port = 8899
```

Use a specific profile:

```bash
cf-solana start --profile ci
```

## Global Configuration

Create `~/.chain-forge/config.toml` for user-wide defaults:

```toml
[solana.default]
accounts = 20
initial_balance = 500.0
```

## Configuration Options

### Required Options

None. All options have sensible defaults.

### Optional Options

#### `rpc_url`

- **Type**: String
- **Default**: `http://localhost:8899`
- **Description**: RPC endpoint URL

Example:
```toml
rpc_url = "http://localhost:8900"
```

#### `accounts`

- **Type**: Integer
- **Default**: `10`
- **Description**: Number of accounts to generate
- **Range**: 1-1000

Example:
```toml
accounts = 20
```

#### `initial_balance`

- **Type**: Float
- **Default**: `100.0`
- **Description**: Initial SOL balance for each account

Example:
```toml
initial_balance = 500.0
```

#### `port`

- **Type**: Integer
- **Default**: `8899`
- **Description**: RPC port for the validator

Example:
```toml
port = 8900
```

## Examples

### Minimal Configuration

```toml
[solana.default]
# Uses all defaults
```

### Development Configuration

```toml
[solana.development]
accounts = 50
initial_balance = 1000.0
port = 8899
```

### CI Configuration

```toml
[solana.ci]
accounts = 3
initial_balance = 10.0
```

### Multiple Profiles

```toml
[solana.default]
accounts = 10
initial_balance = 100.0

[solana.light]
accounts = 3
initial_balance = 10.0

[solana.heavy]
accounts = 100
initial_balance = 5000.0
```

## See Also

- [CLI Commands](./cli)
- [Getting Started](../guide/getting-started)
