# Interactive CLI Example

A full-featured interactive CLI for exploring Solana development with Chain Forge.

## Overview

The Interactive CLI provides a menu-driven interface for:

- **Deploy Node**: Start a local Solana test validator with customizable settings
- **Deploy Program**: Build and deploy Solana programs interactively
- **Send Funds**: Transfer SOL between accounts
- **Real-time Updates**: See account balances update after transactions

## Getting Started

### Prerequisites

- Node.js 18+
- Yarn 4 (via Corepack: `corepack enable`)
- Solana CLI tools
- Rust and `cargo-build-sbf` for building programs

### Installation

```bash
# From the repository root, build Chain Forge first
cargo build --workspace --release

# Install dependencies
cd examples/interactive-cli
yarn install

# Build TypeScript
yarn build

# Run the CLI
yarn start
```

### Development Mode

```bash
yarn dev
```

## Features

### Deploy Node

Configure and start a local Solana validator:

1. **Port**: Choose the RPC port (default: 8899)
2. **Mnemonic**: Use default test mnemonic, generate random, or enter custom
3. **Accounts**: Number of pre-funded accounts (1-100)
4. **Balance**: Initial SOL balance per account

### Deploy Program

Build and deploy Solana programs from the `programs/` directory:

1. Select a program from discovered programs
2. Choose a payer account
3. Build automatically with `cargo build-sbf` if needed
4. Deploy and see the Program ID

### Send Funds

Transfer SOL between accounts:

1. Select source account
2. Select destination (another account or custom address)
3. Enter amount
4. Confirm and send

## Sample Program

The `programs/hello_chain_forge/` directory contains a ready-to-use sample program.

### Building Manually

```bash
yarn build:program
```

### Program Instructions

| Instruction | Byte | Description |
|------------|------|-------------|
| Initialize | `0` | Sets counter to 0 |
| Increment | `1` | Adds 1 to counter |
| Hello | `2+` | Logs greeting message |

## Adding Your Own Programs

Create a new program in the `programs/` directory:

```bash
mkdir -p programs/my_program/src
```

Create `programs/my_program/Cargo.toml`:

```toml
[package]
name = "my_program"
version = "0.1.0"
edition = "2021"

[workspace]

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "2.0"
```

Create `programs/my_program/src/lib.rs` with your program code.

The CLI will automatically discover programs in the `programs/` directory.

## Architecture

```
src/
├── index.ts          # Main loop and SIGINT handling
├── types.ts          # TypeScript interfaces
├── state.ts          # Application state management
├── ui/
│   ├── header.ts     # Status header display
│   ├── menus.ts      # Menu definitions
│   └── formatters.ts # Output formatting utilities
├── actions/
│   ├── deploy-node.ts    # Node deployment flow
│   ├── deploy-program.ts # Program build and deployment
│   └── send-funds.ts     # SOL transfer flow
└── utils/
    └── programs.ts       # Program discovery utilities
```

## UI Design

The CLI displays a header with current state:

```
╔═══════════════════════════════════════════════════════════════╗
║  Chain Forge Interactive CLI                                  ║
║  Nodes: 1 | Port: 8899 | Accounts: 5                         ║
╠═══════════════════════════════════════════════════════════════╣
║  #  │ Address                                    │ Balance   ║
║   0 │ 7xKX...3mPq                                │    100.00 ║
║   1 │ 9aBC...7nRs                                │    100.00 ║
║  ...                                                          ║
╠═══════════════════════════════════════════════════════════════╣
║  Programs: hello_chain_forge (4xYZ...8kLm)                    ║
╚═══════════════════════════════════════════════════════════════╝
```

## See Also

- [TypeScript Examples](./typescript)
- [Program Deployment](./program-deployment)
- [API Reference](../api/overview)
