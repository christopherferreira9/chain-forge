# Chain Forge Architecture Diagram

## Component Interaction Flow

```mermaid
flowchart TB
    subgraph "TypeScript Example Layer"
        Example["TypeScript Example<br/>(examples/typescript-basic/src/*.ts)"]
    end

    subgraph "NPM Package Layer"
        TSClient["@chain-forge/solana<br/>TypeScript Client<br/>(npm/@chain-forge/solana/src/client.ts)"]
    end

    subgraph "Rust CLI Binary"
        CLI["cf-solana CLI<br/>(chains/solana/crates/cli)"]
    end

    subgraph "Rust Core Libraries"
        Core["SolanaProvider<br/>(chain-forge-solana-core)"]
        RPC["SolanaRpcClient<br/>(chain-forge-solana-rpc)"]
        Accounts["AccountGenerator<br/>AccountsStorage<br/>(chain-forge-solana-accounts)"]
        Config["Config System<br/>(chain-forge-config)"]
        Common["Common Traits<br/>(chain-forge-common)"]
    end

    subgraph "External Systems"
        Validator["solana-test-validator<br/>(Binary Process)"]
        SolanaSDK["Solana SDK<br/>(solana-client crate)"]
        FileSystem["File System<br/>~/.chain-forge/solana/accounts.json"]
    end

    %% Example to Client
    Example -->|"import { SolanaClient }"| TSClient
    Example -->|"await client.start()"| TSClient
    Example -->|"await client.getAccounts()"| TSClient
    Example -->|"await client.fundAccount()"| TSClient

    %% Client to CLI
    TSClient -->|"spawn('cf-solana', ['start', ...])"| CLI
    TSClient -->|"Child Process Communication"| CLI

    %% CLI to Core
    CLI -->|"SolanaProvider::with_config()"| Core
    CLI -->|"provider.start(config)"| Core
    CLI -->|"AccountsStorage::load()"| Accounts

    %% Core to Components
    Core -->|"Create accounts"| Accounts
    Core -->|"Load config"| Config
    Core -->|"Implements ChainProvider"| Common
    Core -->|"Fund accounts"| RPC
    Core -->|"Start process"| Validator
    Core -->|"Save accounts"| Accounts

    %% RPC interactions
    RPC -->|"RpcClient::new()"| SolanaSDK
    RPC -->|"request_airdrop()"| SolanaSDK
    RPC -->|"get_balance()"| SolanaSDK

    %% SDK to Validator
    SolanaSDK -->|"HTTP RPC Calls<br/>(localhost:8899)"| Validator

    %% Accounts to FileSystem
    Accounts -->|"Generate BIP39/BIP44 keys"| Accounts
    Accounts -->|"Save/Load JSON"| FileSystem

    %% Config
    Config -->|"Load chain-forge.toml"| Config

    %% Client reads filesystem
    TSClient -->|"Read accounts.json"| FileSystem

    %% Styling
    classDef typescript fill:#3178c6,stroke:#235a97,color:#fff
    classDef rust fill:#ce412b,stroke:#9d2e1f,color:#fff
    classDef external fill:#6c757d,stroke:#495057,color:#fff

    class Example,TSClient typescript
    class CLI,Core,RPC,Accounts,Config,Common rust
    class Validator,SolanaSDK,FileSystem external
```

## Data Flow Example: Starting a Validator with 3 Accounts

```mermaid
sequenceDiagram
    participant Ex as TypeScript Example
    participant Client as @chain-forge/solana
    participant CLI as cf-solana CLI
    participant Core as SolanaProvider
    participant Acc as AccountGenerator
    participant RPC as SolanaRpcClient
    participant Val as solana-test-validator
    participant FS as FileSystem

    Ex->>Client: new SolanaClient({ accounts: 3, initialBalance: 10 })
    Ex->>Client: await client.start()

    Client->>CLI: spawn('cf-solana', ['start', '--accounts', '3', '--balance', '10'])
    CLI->>Core: SolanaProvider::with_config(config)

    Core->>Acc: Generate 3 accounts with BIP39 mnemonic
    Acc->>Acc: Derive keys using m/44'/501'/n'/0'
    Acc-->>Core: Return [Account0, Account1, Account2]

    Core->>Val: Command::new("solana-test-validator")<br/>--rpc-port 8899 --faucet-port 9901
    Val-->>Val: Start validator process

    Core->>RPC: wait_for_validator(60 attempts)
    RPC->>Val: HTTP GET /health
    Val-->>RPC: 200 OK

    loop For each account
        Core->>Core: account.balance = initial_balance (10 SOL)
    end

    Core->>RPC: set_balances(&mut accounts)

    loop For each account (3 times)
        RPC->>Val: request_airdrop(address, 10 SOL)
        Val-->>RPC: Signature
        RPC->>Val: confirm_transaction(signature)
        Val-->>RPC: Confirmed
    end

    Core->>RPC: update_balances(&mut accounts)
    RPC->>Val: get_balance(account.address)
    Val-->>RPC: 10000000000 lamports
    RPC-->>Core: accounts[].balance = 10.0 SOL

    Core->>Acc: AccountsStorage::save(&accounts)
    Acc->>FS: Write ~/.chain-forge/solana/accounts.json

    Core-->>CLI: Ok(validator started)
    CLI-->>Client: stdout: "✅ Validator started!"

    Client->>FS: Read ~/.chain-forge/solana/accounts.json
    FS-->>Client: [{ publicKey, balance, mnemonic, ... }]

    Client-->>Ex: return accounts
    Ex->>Ex: console.log(accounts[0].publicKey)
```

## Component Responsibilities

### TypeScript Layer
- **Example Applications**: Demonstrate usage patterns
- **@chain-forge/solana**: User-friendly API, process management, TypeScript types

### Rust Binary Layer
- **cf-solana CLI**: Command-line interface, argument parsing
- **SolanaProvider**: Orchestrates validator lifecycle, account management
- **SolanaRpcClient**: RPC communication with validator
- **AccountGenerator**: BIP39/BIP44 key derivation, cryptographic operations
- **AccountsStorage**: Persistence to/from JSON
- **Config**: TOML configuration loading

### External Systems
- **solana-test-validator**: Local Solana blockchain
- **Solana SDK**: Official RPC client library
- **File System**: Persistent account storage

## Communication Protocols

1. **TypeScript → Rust**: Child process spawn, stdio communication
2. **Rust → Validator**: HTTP RPC calls (localhost:8899)
3. **Rust → FileSystem**: JSON serialization/deserialization
4. **TypeScript → FileSystem**: Read accounts.json for client-side operations
