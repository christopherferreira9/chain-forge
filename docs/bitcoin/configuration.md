# Configuration

Detailed configuration guide for Chain Forge Bitcoin.

::: warning Development Status
Bitcoin support is currently in active development. Configuration options may change.
:::

## Configuration Files

Chain Forge reads configuration from multiple locations with the following priority:

1. **CLI arguments** (highest priority)
2. **Project config**: `./chain-forge.toml`
3. **Global config**: `~/.chain-forge/config.toml`
4. **Built-in defaults** (lowest priority)

## Project Configuration

Create `chain-forge.toml` in your project root:

```toml
[bitcoin.default]
rpc_url = "http://localhost:18443"
accounts = 10
initial_balance = 10.0
rpc_port = 18443
p2p_port = 18444

[bitcoin.ci]
accounts = 3
initial_balance = 5.0

[bitcoin.heavy]
accounts = 50
initial_balance = 100.0
rpc_port = 18445
p2p_port = 18446
```

Use a specific profile:

```bash
cf-bitcoin start --profile ci
```

## Global Configuration

Create `~/.chain-forge/config.toml` for user-wide defaults:

```toml
[bitcoin.default]
accounts = 20
initial_balance = 50.0
```

## Configuration Options

### Required Options

None. All options have sensible defaults.

### Optional Options

#### `rpc_url`

- **Type**: String
- **Default**: `http://localhost:18443`
- **Description**: RPC endpoint URL for connecting to the node

Example:
```toml
rpc_url = "http://localhost:18445"
```

#### `accounts`

- **Type**: Integer
- **Default**: `10`
- **Description**: Number of accounts to generate
- **Range**: 1-100

Example:
```toml
accounts = 20
```

#### `initial_balance`

- **Type**: Float
- **Default**: `10.0`
- **Description**: Initial BTC balance for each account

Example:
```toml
initial_balance = 50.0
```

#### `rpc_port`

- **Type**: Integer
- **Default**: `18443`
- **Description**: RPC port for the Bitcoin node

Example:
```toml
rpc_port = 18445
```

#### `p2p_port`

- **Type**: Integer
- **Default**: `18444`
- **Description**: P2P network port for the Bitcoin node

Example:
```toml
p2p_port = 18446
```

## Instance Configuration

Each instance stores its own configuration in:

```
~/.chain-forge/bitcoin/instances/<instance-id>/instance.json
```

Example:

```json
{
  "instance_id": "default",
  "rpc_url": "http://localhost:18443",
  "rpc_port": 18443,
  "p2p_port": 18444,
  "rpc_user": "chainforge",
  "rpc_password": "chainforge",
  "running": true
}
```

This file is used by CLI commands to discover running instances.

## Examples

### Minimal Configuration

```toml
[bitcoin.default]
# Uses all defaults:
# - 10 accounts
# - 10.0 BTC each
# - RPC port 18443
# - P2P port 18444
```

### Development Configuration

```toml
[bitcoin.development]
accounts = 20
initial_balance = 100.0
rpc_port = 18443
p2p_port = 18444
```

### CI Configuration

```toml
[bitcoin.ci]
accounts = 3
initial_balance = 5.0
```

### Multiple Profiles

```toml
[bitcoin.default]
accounts = 10
initial_balance = 10.0

[bitcoin.light]
accounts = 3
initial_balance = 5.0

[bitcoin.heavy]
accounts = 50
initial_balance = 100.0
rpc_port = 18445
p2p_port = 18446
```

### Running Multiple Instances

To run multiple Bitcoin nodes simultaneously:

```toml
[bitcoin.node1]
rpc_port = 18443
p2p_port = 18444
accounts = 5
initial_balance = 10.0

[bitcoin.node2]
rpc_port = 18445
p2p_port = 18446
accounts = 5
initial_balance = 10.0
```

Start each instance:

```bash
# Terminal 1
cf-bitcoin start --profile node1 --instance node1

# Terminal 2
cf-bitcoin start --profile node2 --instance node2
```

## RPC Authentication

The default RPC credentials are:

- **Username**: `chainforge`
- **Password**: `chainforge`

These can be overridden via CLI:

```bash
cf-bitcoin start --rpc-user myuser --rpc-password mypassword
```

## Data Directory Structure

```
~/.chain-forge/
├── config.toml                 # Global configuration
└── bitcoin/
    └── instances/
        ├── default/
        │   ├── accounts.json   # Account keys and balances
        │   ├── instance.json   # RPC connection info
        │   └── regtest-data/   # Bitcoin blockchain data
        └── mytest/
            ├── accounts.json
            ├── instance.json
            └── regtest-data/
```

## Clean State Behavior

Each `cf-bitcoin start` clears previous instance data:

- Blockchain data is deleted
- Accounts are regenerated (same addresses if same mnemonic)
- Balances start fresh

This ensures reproducible testing environments.

## See Also

- [CLI Commands](./cli)
- [Account Management](./accounts)
- [Overview](./overview)
