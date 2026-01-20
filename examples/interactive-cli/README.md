# Chain Forge Interactive CLI

A full-featured interactive CLI for exploring Solana development with Chain Forge.

## Features

- **Deploy Node**: Start a local Solana test validator with customizable settings
- **Deploy Program**: Build and deploy Solana programs from the `programs/` directory
- **Create Program Account**: Create accounts owned by deployed programs for instruction execution
- **Interact with Program**: Execute program instructions using IDL definitions
- **Send Funds**: Transfer SOL between accounts interactively
- **Real-time Balance Updates**: See account balances update after transactions

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

## Sample Program

The `programs/hello_chain_forge/` directory contains a sample Solana program you can deploy.

### Building the Sample Program

```bash
yarn build:program
```

This runs `cargo build-sbf` in the program directory.

## Usage

### Deploy a Node

1. Select "Deploy Node" from the menu
2. Configure port (default: 8899)
3. Choose mnemonic option (default, random, or custom)
4. Set number of accounts (default: 5)
5. Set initial balance per account (default: 100 SOL)

### Deploy a Program

1. With a node running, select "Deploy Program"
2. Choose a program from the discovered list
3. Select a payer account
4. If not built, confirm to build with `cargo build-sbf`
5. Program deploys and appears in the header

### Send Funds

1. Select "Send Funds"
2. Choose source account
3. Choose destination (another account or custom address)
4. Enter amount
5. Confirm and send

### Program Interaction Example

The sample `hello_chain_forge` program includes a counter that can be initialized and incremented. Here's a complete walkthrough:

#### Step 1: Deploy the Node and Program

1. Start the CLI: `yarn start`
2. Select **Deploy Node** with default settings
3. Select **Deploy Program** → `hello_chain_forge`
4. Choose Account 0 as the payer

#### Step 2: Create a Program-Owned Account

Program instructions like Initialize and Increment require an account **owned by the program**, not a regular wallet account.

1. Select **Create Program Account**
2. Select `hello_chain_forge` as the owner program
3. Select Account 0 as the payer (will pay rent)
4. Enter data size: `8` bytes (default, enough for a u64 counter)

The new account appears in the header under "Program Accounts" and will be available when interacting with the program.

#### Step 3: Initialize the Counter

1. Select **Interact with Program**
2. Select `hello_chain_forge`
3. Select Account 0 for the transaction (signer/payer)
4. Select **Initialize** instruction
5. For the `counter` account, select the **Program Account** you just created (appears at the top of the list)

#### Step 4: Increment the Counter

1. Select **Interact with Program**
2. Select `hello_chain_forge`
3. Select Account 0 for the transaction
4. Select **Increment** instruction
5. Select the same **Program Account**

Each successful Increment adds 1 to the counter stored in the program account.

#### Step 5: Read the Counter Value

1. Select **Interact with Program**
2. Select `hello_chain_forge`
3. Select Account 0 for the transaction
4. Select **Read** instruction
5. Select the same **Program Account**

The CLI automatically displays program logs after each transaction, showing the counter value:
```
Transaction Result:
  Signature: 5abc...

Program Logs:
  Program log: Hello Chain Forge program invoked
  Program log: Instruction: Read
  Program log: Current counter value: 1
```

#### Why Program Accounts?

Solana programs can only modify accounts they own. When you create an account with `SystemProgram.createAccount` and set the program as the owner:

- The program can read and write the account's data
- Only that program can modify the account
- The account must be rent-exempt (CLI calculates this automatically)

Regular wallet accounts are owned by the System Program, so the `hello_chain_forge` program cannot modify them.

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
│   ├── deploy-node.ts           # Node deployment flow
│   ├── deploy-program.ts        # Program build and deployment
│   ├── create-program-account.ts # Create program-owned accounts
│   ├── interact-program.ts      # Execute program instructions
│   └── send-funds.ts            # SOL transfer flow
└── utils/
    ├── programs.ts   # Program discovery utilities
    └── idl.ts        # IDL loading utilities
```
