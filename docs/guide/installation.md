# Installation

This guide covers installing Chain Forge on your system.

## Prerequisites

### Required

#### Rust Toolchain

Chain Forge is built with Rust. Install version 1.75 or later:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:

```bash
rustc --version  # Should show 1.75 or later
cargo --version
```

#### Solana CLI Tools

Required for running the Solana test validator:

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

Add to your PATH (the installer will show you the command):

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

Verify installation:

```bash
solana --version
solana-test-validator --help
```

### Optional

#### Node.js (for TypeScript package)

Version 18 or later required for using `@chain-forge/solana`:

```bash
# Check your version
node --version

# Install via nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

#### Yarn 4 (for development)

If building from source:

```bash
corepack enable
```

## Installation Methods

### Method 1: Install from Crates.io (Recommended)

Once published, you'll be able to install directly:

```bash
cargo install chain-forge-solana-cli
```

Verify installation:

```bash
cf-solana --version
```

### Method 2: Build from Source

Clone the repository and build:

```bash
# Clone
git clone https://github.com/christopherferreira9/chain-forge
cd chain-forge

# Build the entire workspace
cargo build --workspace --release

# Install the Solana CLI
cargo install --path chains/solana/crates/cli

# Verify
cf-solana --version
```

### Method 3: Install TypeScript Package

For programmatic access in Node.js projects:

```bash
# Using npm
npm install @chain-forge/solana @solana/web3.js

# Using yarn
yarn add @chain-forge/solana @solana/web3.js

# Using pnpm
pnpm add @chain-forge/solana @solana/web3.js
```

Note: The TypeScript package requires the CLI to be installed separately.

## Verify Installation

Test that everything works:

```bash
# Check CLI is available
cf-solana --version

# Check Solana tools are available
solana --version
solana-test-validator --help

# Start a validator (will stop automatically)
cf-solana start --accounts 1 --balance 10
# Press Ctrl+C to stop
```

## Platform-Specific Notes

### macOS

No additional steps required. OpenSSL is vendored for compatibility.

### Linux

OpenSSL is vendored. On some distributions you may need to install pkg-config:

```bash
# Debian/Ubuntu
sudo apt-get install pkg-config

# Fedora
sudo dnf install pkg-config

# Arch
sudo pacman -S pkg-config
```

### Windows

Windows support is experimental. You may encounter:

- Path separator issues
- Process spawning differences
- Longer build times

Consider using WSL2 (Windows Subsystem for Linux) for the best experience:

```bash
# In WSL2
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
cargo install chain-forge-solana-cli
```

## Updating

### Update CLI

```bash
# From crates.io
cargo install chain-forge-solana-cli --force

# From source
cd chain-forge
git pull
cargo install --path chains/solana/crates/cli --force
```

### Update TypeScript Package

```bash
npm update @chain-forge/solana
# or
yarn upgrade @chain-forge/solana
```

### Update Solana Tools

```bash
solana-install update
```

## Troubleshooting

### "cf-solana: command not found"

The Cargo bin directory is not in your PATH. Add this to your shell profile:

```bash
# ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload your shell:

```bash
source ~/.bashrc  # or source ~/.zshrc
```

### "solana-test-validator: command not found"

The Solana CLI tools are not in your PATH. Add this to your shell profile:

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### Build Errors

**OpenSSL issues:**

The project uses vendored OpenSSL, but if you encounter issues:

```bash
# macOS
brew install openssl

# Linux
sudo apt-get install libssl-dev pkg-config
```

**Linker errors:**

Ensure you have a C compiler installed:

```bash
# macOS
xcode-select --install

# Linux
sudo apt-get install build-essential
```

### Permission Errors

If you get permission errors during installation:

```bash
# Don't use sudo with cargo install
# Instead, ensure ~/.cargo/bin is writable
chmod +x ~/.cargo/bin
```

## Uninstallation

### Remove CLI

```bash
cargo uninstall chain-forge-solana-cli
```

### Remove Configuration and Data

```bash
rm -rf ~/.chain-forge
```

### Remove Solana Tools (optional)

```bash
rm -rf ~/.local/share/solana
```

## Next Steps

- [Getting Started Guide](./getting-started)
- [Solana CLI Reference](../solana/cli)
- [TypeScript Usage Guide](../typescript/basic-usage)
