# Bitcoin Dependencies for Chain Forge

Chain Forge Bitcoin support requires Bitcoin Core to be installed on your system.

## Required: Bitcoin Core

The `cf-bitcoin` CLI tool uses `bitcoind` to run a local Bitcoin regtest node for development and testing.

### Installation

#### macOS (Homebrew)

```bash
brew install bitcoin
```

#### Ubuntu/Debian

```bash
# Add Bitcoin PPA (for latest version)
sudo add-apt-repository ppa:bitcoin/bitcoin
sudo apt-get update

# Install Bitcoin Core
sudo apt-get install bitcoind bitcoin-cli
```

Alternatively, download from the official website.

#### Windows

Download the installer from: https://bitcoincore.org/en/download/

After installation, ensure `bitcoind.exe` is in your PATH or install it to a standard location.

#### From Source

```bash
# Clone the repository
git clone https://github.com/bitcoin/bitcoin.git
cd bitcoin

# Build (requires dependencies - see Bitcoin Core docs)
./autogen.sh
./configure
make
sudo make install
```

### Verify Installation

After installation, verify that Bitcoin Core is properly installed:

```bash
# Check bitcoind version
bitcoind --version

# Check bitcoin-cli version (optional, not required by Chain Forge)
bitcoin-cli --version
```

You should see output similar to:
```
Bitcoin Core version v27.0.0
```

## Configuration

Chain Forge automatically configures `bitcoind` for regtest mode. No manual Bitcoin configuration is required for local development.

### Default Settings

| Setting | Value | Description |
|---------|-------|-------------|
| Network | regtest | Local development network |
| RPC Port | 18443 | JSON-RPC API port |
| P2P Port | 18444 | Peer-to-peer network port |
| RPC User | chainforge | RPC authentication username |
| RPC Password | chainforge | RPC authentication password |

### Custom Configuration

You can override defaults via CLI arguments:

```bash
cf-bitcoin start \
  --rpc-port 19443 \
  --p2p-port 19444 \
  --rpc-user myuser \
  --rpc-password mypassword
```

Or via `chain-forge.toml`:

```toml
[bitcoin.default]
rpc_url = "http://localhost:18443"
rpc_port = 18443
p2p_port = 18444
accounts = 10
initial_balance = 10.0
rpc_user = "chainforge"
rpc_password = "chainforge"
```

## Storage Locations

Chain Forge stores Bitcoin-related data in the following locations:

| Data | Location |
|------|----------|
| Accounts | `~/.chain-forge/bitcoin/accounts.json` |
| Regtest data | `~/.chain-forge/bitcoin/regtest-data/` |
| Configuration | `~/.chain-forge/config.toml` |

**Security Note:** The accounts file contains private keys. Never commit this file to version control or share it publicly.

## Troubleshooting

### "bitcoind not found"

Ensure Bitcoin Core is installed and `bitcoind` is in your PATH:

```bash
# Check if bitcoind is in PATH
which bitcoind

# If not found, add to PATH (example for Homebrew on macOS)
export PATH="/opt/homebrew/bin:$PATH"
```

### Port Already in Use

If ports 18443 or 18444 are already in use:

```bash
# Check what's using the port
lsof -i :18443

# Use different ports
cf-bitcoin start --rpc-port 19443 --p2p-port 19444
```

### Permission Denied

On Linux, you may need to ensure proper permissions:

```bash
# Make bitcoind executable (if installed from binary)
chmod +x /path/to/bitcoind
```

### Slow Startup

Bitcoin node startup includes mining initial blocks for account funding. This is expected and typically completes within 30-60 seconds. The startup message will indicate progress:

```
⛏️  Mining initial blocks (this may take a moment)...
```

## Network Differences

Chain Forge uses Bitcoin's **regtest** mode, which differs from mainnet and testnet:

| Feature | Regtest | Testnet | Mainnet |
|---------|---------|---------|---------|
| Block generation | On-demand (instant) | ~10 minutes | ~10 minutes |
| Coinbase maturity | 100 blocks | 100 blocks | 100 blocks |
| Difficulty | Minimal | Adjusted | Adjusted |
| Real value | No | No | Yes |
| Address prefix | bcrt1 | tb1 | bc1 |

Regtest is ideal for development because:
- Blocks can be mined instantly
- No need to wait for confirmations
- Full control over the blockchain state
- No real funds at risk

## Further Resources

- [Bitcoin Core Documentation](https://bitcoin.org/en/bitcoin-core/)
- [Bitcoin Developer Guide](https://developer.bitcoin.org/)
- [Regtest Mode Documentation](https://developer.bitcoin.org/examples/testing.html)
