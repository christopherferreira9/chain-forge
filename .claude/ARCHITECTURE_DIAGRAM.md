# Chain Forge Architecture Diagram

## High-Level System Overview

```mermaid
flowchart TB
    subgraph "User Interfaces"
        Dashboard["Web Dashboard<br/>(development/dashboard)<br/>React + Vite + TailwindCSS"]
        TSCLI["TypeScript Example<br/>(examples/typescript-basic)"]
    end

    subgraph "API Layer"
        APIServer["REST API Server (cf-api)<br/>(crates/api-server)<br/>Axum + Tokio"]
        TSClient["@chain-forge/solana<br/>TypeScript Client<br/>(npm/@chain-forge/solana)"]
    end

    subgraph "CLI Binaries"
        SolanaCLI["cf-solana CLI<br/>(chains/solana/crates/cli)"]
        BitcoinCLI["cf-bitcoin CLI<br/>(chains/bitcoin/crates/cli)"]
    end

    subgraph "Shared Infrastructure"
        Common["Common Traits & Registry<br/>(crates/common)<br/>ChainProvider, NodeRegistry"]
        Config["Config System<br/>(crates/config)"]
        CLIUtils["CLI Utilities<br/>(crates/cli-utils)"]
    end

    subgraph "Solana Chain"
        SolCore["SolanaProvider<br/>(chain-forge-solana-core)"]
        SolRPC["SolanaRpcClient<br/>(chain-forge-solana-rpc)"]
        SolAccounts["AccountGenerator<br/>(chain-forge-solana-accounts)"]
    end

    subgraph "Bitcoin Chain"
        BtcCore["BitcoinProvider<br/>(chain-forge-bitcoin-core)"]
        BtcRPC["BitcoinRpcClient<br/>(chain-forge-bitcoin-rpc)"]
        BtcAccounts["AccountGenerator<br/>(chain-forge-bitcoin-accounts)"]
    end

    subgraph "External Systems"
        SolValidator["solana-test-validator<br/>(Binary Process)<br/>Port 8899"]
        BtcNode["bitcoind -regtest<br/>(Binary Process)<br/>Port 18443"]
        FS["File System<br/>~/.chain-forge/"]
    end

    %% Dashboard to API
    Dashboard -->|"HTTP REST<br/>localhost:3001"| APIServer

    %% TypeScript flow
    TSCLI -->|"import"| TSClient
    TSClient -->|"spawn child process"| SolanaCLI

    %% API Server to chain crates
    APIServer -->|"Solana operations"| SolRPC
    APIServer -->|"Bitcoin operations"| BtcRPC
    APIServer -->|"Node tracking"| Common

    %% CLIs to Core
    SolanaCLI --> SolCore
    BitcoinCLI --> BtcCore

    %% Core to components
    SolCore --> SolRPC
    SolCore --> SolAccounts
    SolCore --> Common
    SolCore --> Config

    BtcCore --> BtcRPC
    BtcCore --> BtcAccounts
    BtcCore --> Common
    BtcCore --> Config

    %% RPC to external
    SolRPC -->|"HTTP RPC"| SolValidator
    BtcRPC -->|"JSON-RPC"| BtcNode

    %% Storage
    SolAccounts -->|"accounts.json"| FS
    BtcAccounts -->|"accounts.json"| FS
    Common -->|"registry.json"| FS
    Config -->|"config.toml"| FS

    %% Styling
    classDef ui fill:#3178c6,stroke:#235a97,color:#fff
    classDef api fill:#8b5cf6,stroke:#6d28d9,color:#fff
    classDef rust fill:#ce412b,stroke:#9d2e1f,color:#fff
    classDef solana fill:#9945FF,stroke:#7C3AED,color:#fff
    classDef bitcoin fill:#F7931A,stroke:#C77514,color:#fff
    classDef external fill:#6c757d,stroke:#495057,color:#fff

    class Dashboard,TSCLI ui
    class APIServer,TSClient api
    class SolanaCLI,BitcoinCLI,Common,Config,CLIUtils rust
    class SolCore,SolRPC,SolAccounts solana
    class BtcCore,BtcRPC,BtcAccounts bitcoin
    class SolValidator,BtcNode,FS external
```

## Data Flow: Starting a Bitcoin Node with 3 Accounts

```mermaid
sequenceDiagram
    participant User
    participant CLI as cf-bitcoin CLI
    participant Core as BitcoinProvider
    participant Acc as AccountGenerator
    participant RPC as BitcoinRpcClient
    participant Node as bitcoind (regtest)
    participant FS as FileSystem
    participant Reg as NodeRegistry

    User->>CLI: cf-bitcoin start --accounts 3 --balance 10
    CLI->>Core: BitcoinProvider::with_config(config)

    Core->>Acc: Generate 3 accounts with BIP39 mnemonic
    Acc->>Acc: Derive keys using m/84'/1'/0'/0/n (P2WPKH)
    Acc-->>Core: Return [Account0, Account1, Account2]

    Core->>Node: Spawn bitcoind -regtest -rpcport=18443
    Node-->>Node: Start regtest node

    Core->>RPC: wait_for_node(60 attempts)
    RPC->>Node: getblockchaininfo
    Node-->>RPC: 200 OK

    Core->>RPC: create_wallet("chain-forge")
    RPC->>Node: createwallet
    Node-->>RPC: Wallet created

    Core->>RPC: get_new_address("mining")
    RPC->>Node: getnewaddress
    Node-->>RPC: mining address

    Core->>RPC: mine_blocks(101, mining_addr)
    RPC->>Node: generatetoaddress 101
    Node-->>RPC: 101 block hashes (coinbase matures)

    loop For each account
        Core->>RPC: import_address(addr, wif, label)
        RPC->>Node: importdescriptors wpkh(WIF)
        Node-->>RPC: Success
    end

    Core->>RPC: fund_accounts(&accounts)
    loop For each account (3 times)
        RPC->>Node: sendtoaddress(addr, 10 BTC)
        Node-->>RPC: txid
    end

    Core->>RPC: mine_blocks(1, mining_addr)
    RPC->>Node: generatetoaddress 1
    Node-->>RPC: Confirmation block

    Core->>RPC: update_balances(&accounts)
    RPC->>Node: scantxoutset for each address
    Node-->>RPC: 10.0 BTC each

    Core->>Acc: AccountsStorage::save(&accounts)
    Acc->>FS: Write ~/.chain-forge/bitcoin/instances/{id}/accounts.json

    Core->>Reg: Register node in registry
    Reg->>FS: Write ~/.chain-forge/registry.json

    Core-->>CLI: Ok(node started)
    CLI-->>User: Validator started! 3 accounts funded.
```

## Data Flow: Dashboard Viewing Transactions

```mermaid
sequenceDiagram
    participant Browser as Web Dashboard
    participant API as cf-api (REST API)
    participant Reg as NodeRegistry
    participant RPC as BitcoinRpcClient
    participant Node as bitcoind
    participant FS as FileSystem

    Browser->>API: GET /api/v1/nodes
    API->>Reg: registry.list()
    Reg->>FS: Read registry.json
    FS-->>Reg: Node list
    Reg-->>API: Vec<NodeInfo>
    API-->>Browser: [{ node_id: "bitcoin:default", status: "running", ... }]

    Browser->>API: GET /api/v1/nodes/bitcoin:default/transactions
    API->>Reg: registry.get("bitcoin:default")
    Reg-->>API: NodeInfo { chain: Bitcoin, instance_id: "default" }

    API->>FS: Load instance info
    FS-->>API: { rpc_url, rpc_user, rpc_password }

    API->>RPC: new_with_wallet(url, user, pass, "chain-forge")
    API->>FS: Load accounts.json
    FS-->>API: Known addresses [addr0, addr1, addr2]

    API->>RPC: list_transactions(100)
    RPC->>Node: listtransactions "*" 100 0 true
    Node-->>RPC: Wallet transactions

    API->>API: Filter by known addresses
    API->>API: Deduplicate by txid
    API->>API: Sort by block_time desc
    API-->>Browser: [{ signature: "txid...", slot: 102, account: "bcrt1q...", ... }]

    Browser->>Browser: Render transaction table

    Note over Browser: User clicks a transaction row

    Browser->>API: GET /api/v1/nodes/bitcoin:default/transactions/{txid}
    API->>RPC: get_transaction_detail(txid)
    RPC->>Node: gettransaction txid true
    Node-->>RPC: { amount, fee, details: [...] }
    RPC-->>API: BitcoinTransactionDetail
    API->>API: Map details to BalanceChangeInfo, abs(fee)
    API-->>Browser: { signature, fee, balance_changes: [...] }

    Browser->>Browser: Render detail panel with fee + balance changes
```

## Component Responsibilities

### User Interfaces
- **Web Dashboard** (`development/dashboard/`): React SPA for visual node management. Auto-refreshes via React Query. Chain-aware UI (SOL/BTC units, Signature/TxID labels).
- **TypeScript Examples** (`examples/`): Demonstrate programmatic usage of the NPM package.

### API Layer
- **REST API Server** (`crates/api-server/`): Axum HTTP server. Unified endpoints for both chains. Powers the dashboard. Binary: `cf-api`, default port 3001.
- **@chain-forge/solana** (`npm/@chain-forge/solana/`): TypeScript client that spawns `cf-solana` CLI as a child process.

### CLI Binaries
- **cf-solana** (`chains/solana/crates/cli/`): Solana-specific CLI commands (start, accounts, fund, config, stop).
- **cf-bitcoin** (`chains/bitcoin/crates/cli/`): Bitcoin-specific CLI commands (start, accounts, fund, config, stop).

### Shared Infrastructure
- **Common** (`crates/common/`): `ChainProvider` trait, `ChainError` types, `NodeRegistry` for tracking nodes, `ChainType` enum, input validation.
- **Config** (`crates/config/`): TOML-based config with profiles, data directory management.
- **CLI Utils** (`crates/cli-utils/`): Output formatting (JSON/table).

### Chain Implementations
- **Solana** (`chains/solana/crates/`): accounts (BIP39/BIP44), rpc (solana-client wrapper), core (provider + validator lifecycle), cli.
- **Bitcoin** (`chains/bitcoin/crates/`): accounts (BIP39/BIP84 P2WPKH), rpc (bitcoincore-rpc wrapper), core (provider + bitcoind lifecycle), cli.

### External Systems
- **solana-test-validator**: Local Solana blockchain (port 8899 default).
- **bitcoind (regtest)**: Local Bitcoin blockchain (port 18443 default).
- **File System**: Account storage, node registry, instance info, config files under `~/.chain-forge/`.

## Communication Protocols

1. **Dashboard → API Server**: HTTP REST (port 5173 → port 3001 via Vite proxy)
2. **API Server → Chain RPC**: HTTP JSON-RPC (Solana port 8899, Bitcoin port 18443)
3. **TypeScript → Rust**: Child process spawn + stdio communication
4. **Rust → External Nodes**: HTTP RPC calls
5. **All components → File System**: JSON serialization for accounts, registry, instance info
