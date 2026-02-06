# REST API Reference

The Chain Forge REST API provides HTTP endpoints for monitoring and controlling blockchain nodes. This API powers the web dashboard and can be used for custom integrations.

## Getting Started

### Starting the API Server

```bash
# Build the API server
cargo build --release -p chain-forge-api-server

# Start on default port (3001)
cf-api

# Start on custom port
cf-api --port 8080

# Start and open browser
cf-api --open
```

### Base URL

All endpoints are prefixed with `/api/v1`:

```
http://localhost:3001/api/v1
```

### Response Format

All responses follow a consistent JSON structure:

```typescript
interface ApiResponse<T> {
  success: boolean;
  data?: T;        // Present on success
  error?: string;  // Present on failure
}
```

#### Success Response

```json
{
  "success": true,
  "data": { ... }
}
```

#### Error Response

```json
{
  "success": false,
  "error": "Error message describing what went wrong"
}
```

---

## Endpoints

### List All Nodes

Returns all registered blockchain nodes across all chains.

```
GET /api/v1/nodes
```

#### Response

```typescript
interface NodeInfo {
  node_id: string;           // Unique identifier: "{chain}:{instance_id}"
  name: string | null;       // Human-readable name
  chain: "solana" | "bitcoin";
  instance_id: string;       // Instance identifier
  rpc_url: string;           // RPC endpoint URL
  rpc_port: number;          // RPC port number
  accounts_count: number;    // Number of generated accounts
  status: "running" | "stopped" | "unknown";
  started_at: string | null; // ISO 8601 timestamp
}
```

#### Example

```bash
curl http://localhost:3001/api/v1/nodes
```

```json
{
  "success": true,
  "data": [
    {
      "node_id": "solana:dev",
      "name": "Development",
      "chain": "solana",
      "instance_id": "dev",
      "rpc_url": "http://localhost:8899",
      "rpc_port": 8899,
      "accounts_count": 10,
      "status": "running",
      "started_at": "2024-01-15T10:30:00Z"
    },
    {
      "node_id": "bitcoin:btc-test",
      "name": "Bitcoin Test",
      "chain": "bitcoin",
      "instance_id": "btc-test",
      "rpc_url": "http://localhost:18443",
      "rpc_port": 18443,
      "accounts_count": 5,
      "status": "running",
      "started_at": "2024-01-15T11:00:00Z"
    }
  ]
}
```

---

### Get Node Details

Returns detailed information about a specific node.

```
GET /api/v1/nodes/{node_id}
```

#### Parameters

| Parameter | Type   | Description                                    |
|-----------|--------|------------------------------------------------|
| node_id   | string | Node identifier (e.g., `solana:dev`, `bitcoin:test`) |

#### Example

```bash
curl http://localhost:3001/api/v1/nodes/solana:dev
```

```json
{
  "success": true,
  "data": {
    "node_id": "solana:dev",
    "name": "Development",
    "chain": "solana",
    "instance_id": "dev",
    "rpc_url": "http://localhost:8899",
    "rpc_port": 8899,
    "accounts_count": 10,
    "status": "running",
    "started_at": "2024-01-15T10:30:00Z"
  }
}
```

#### Errors

| Status | Error                    | Description           |
|--------|--------------------------|----------------------|
| 404    | "Node not found"         | Node ID doesn't exist |

---

### Get Node Accounts

Returns all accounts associated with a specific node.

```
GET /api/v1/nodes/{node_id}/accounts
```

#### Parameters

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| node_id   | string | Node identifier  |

#### Response

```typescript
interface AccountInfo {
  index: number;     // Account index (0-based)
  address: string;   // Public key / address
  balance: number;   // Current balance (SOL or BTC)
}
```

#### Example

```bash
curl http://localhost:3001/api/v1/nodes/solana:dev/accounts
```

```json
{
  "success": true,
  "data": [
    {
      "index": 0,
      "address": "7xJ5k2m8QJK9xnFhZwkJ...",
      "balance": 100.0
    },
    {
      "index": 1,
      "address": "9aB3c4D5eFgHiJkLmNo...",
      "balance": 100.0
    }
  ]
}
```

#### Errors

| Status | Error                      | Description              |
|--------|----------------------------|--------------------------|
| 404    | "Node not found"           | Node ID doesn't exist    |
| 500    | "Failed to load accounts"  | Accounts file not found  |

---

### Health Check

Performs a health check on all registered nodes, updating their status in the registry.

```
POST /api/v1/health
```

#### Request Body

None required.

#### Response

```typescript
interface HealthCheckResponse {
  total: number;    // Total registered nodes
  running: number;  // Currently running nodes
  stopped: number;  // Stopped nodes
  unknown: number;  // Nodes with unknown status
}
```

#### Example

```bash
curl -X POST http://localhost:3001/api/v1/health
```

```json
{
  "success": true,
  "data": {
    "total": 3,
    "running": 2,
    "stopped": 1,
    "unknown": 0
  }
}
```

::: tip
The health check actively probes each node's RPC endpoint to verify it's responding. Node statuses in the registry are updated based on these probes.
:::

---

### Start Node

Returns the CLI command needed to start a new node. Due to the nature of blockchain nodes requiring a persistent process, actual node startup must be done via the CLI.

```
POST /api/v1/nodes
```

#### Request Body

```typescript
interface StartNodeRequest {
  chain: "solana" | "bitcoin";  // Required
  instance?: string;            // Default: "default"
  name?: string;                // Human-readable name
  port?: number;                // RPC port (default: 8899 for Solana, 18443 for Bitcoin)
  accounts?: number;            // Default: 10
  balance?: number;             // Default: 100 SOL or 10 BTC
}
```

#### Response

```typescript
interface StartNodeResponse {
  message: string;    // Instructions
  command: string;    // CLI command to run
  chain: string;
  instance: string;
  port: number;
}
```

#### Example

```bash
curl -X POST http://localhost:3001/api/v1/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "chain": "solana",
    "instance": "my-node",
    "name": "My Development Node",
    "port": 8899,
    "accounts": 5,
    "balance": 200
  }'
```

```json
{
  "success": true,
  "data": {
    "message": "Node start requires running the CLI command in a separate terminal",
    "command": "cf-solana start --instance my-node --port 8899 --accounts 5 --balance 200 --name \"My Development Node\"",
    "chain": "solana",
    "instance": "my-node",
    "port": 8899
  }
}
```

::: warning
This endpoint does **not** start the node directly. Copy the returned `command` and run it in a terminal to start the node.
:::

---

### Stop Node

Marks a node as stopped in the registry. The actual node process must be stopped manually (Ctrl+C in the terminal running it).

```
DELETE /api/v1/nodes/{node_id}
```

#### Parameters

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| node_id   | string | Node identifier  |

#### Response

```typescript
interface StopNodeResponse {
  message: string;      // Status message
  instruction: string;  // How to actually stop the node
  node_id: string;
}
```

#### Example

```bash
curl -X DELETE http://localhost:3001/api/v1/nodes/solana:dev
```

```json
{
  "success": true,
  "data": {
    "message": "Node marked as stopped. To actually stop the node:",
    "instruction": "Press Ctrl+C in the terminal running 'cf-solana start --instance dev'",
    "node_id": "solana:dev"
  }
}
```

::: warning
This endpoint only updates the registry status. To actually stop the node process, use Ctrl+C in the terminal where it's running.
:::

---

### Fund Account

Sends funds to an account on a specific node.

```
POST /api/v1/nodes/{node_id}/fund
```

#### Parameters

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| node_id   | string | Node identifier  |

#### Request Body

```typescript
interface FundAccountRequest {
  address: string;  // Account address to fund
  amount: number;   // Amount to send (SOL or BTC)
}
```

#### Response

```typescript
interface FundResponse {
  success: boolean;
  txid_or_signature: string;  // Transaction ID (Bitcoin) or signature (Solana)
  address: string;
  amount: number;
}
```

#### Example - Solana

```bash
curl -X POST http://localhost:3001/api/v1/nodes/solana:dev/fund \
  -H "Content-Type: application/json" \
  -d '{
    "address": "7xJ5k2m8QJK9xnFhZwkJ...",
    "amount": 50
  }'
```

```json
{
  "success": true,
  "data": {
    "success": true,
    "txid_or_signature": "5K8sH7jQ2nM...",
    "address": "7xJ5k2m8QJK9xnFhZwkJ...",
    "amount": 50
  }
}
```

#### Example - Bitcoin

```bash
curl -X POST http://localhost:3001/api/v1/nodes/bitcoin:test/fund \
  -H "Content-Type: application/json" \
  -d '{
    "address": "bcrt1qxyz...",
    "amount": 5
  }'
```

```json
{
  "success": true,
  "data": {
    "success": true,
    "txid_or_signature": "a1b2c3d4e5f6...",
    "address": "bcrt1qxyz...",
    "amount": 5
  }
}
```

#### Errors

| Status | Error                              | Description                    |
|--------|-----------------------------------|--------------------------------|
| 404    | "Node not found"                  | Node ID doesn't exist          |
| 503    | "Solana validator is not running" | Node not running               |
| 503    | "Bitcoin node is not running"     | Node not running               |
| 500    | "Airdrop failed: ..."             | Solana airdrop error           |
| 500    | "Transaction failed: ..."         | Bitcoin transaction error      |

::: tip
For Solana nodes, this uses the airdrop mechanism (free test SOL).
For Bitcoin nodes, this sends from the node's wallet funds, and automatically mines a block to confirm the transaction.
:::

---

### List Node Transactions

Returns recent transactions for all accounts on a specific node. For Solana, fetches the last 10 signatures per account. For Bitcoin, fetches the last 100 wallet transactions filtered to known accounts.

```
GET /api/v1/nodes/{node_id}/transactions
```

#### Parameters

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| node_id   | string | Node identifier  |

#### Response

```typescript
interface TransactionInfo {
  signature: string;               // Transaction signature (Solana) or txid (Bitcoin)
  slot: number;                    // Slot (Solana) or block height (Bitcoin), 0 if unconfirmed
  err: string | null;              // Error message if transaction failed
  memo: string | null;             // Optional memo (Solana only)
  block_time: number | null;       // Unix timestamp of the block
  confirmation_status: string | null; // e.g. "finalized" (Solana) or "6 confirmations" / "unconfirmed" (Bitcoin)
  account: string;                 // Account address involved in the transaction
}
```

#### Example - Solana

```bash
curl http://localhost:3001/api/v1/nodes/solana:dev/transactions
```

```json
{
  "success": true,
  "data": [
    {
      "signature": "5K8sH7jQ2nM...",
      "slot": 42,
      "err": null,
      "memo": null,
      "block_time": 1700000000,
      "confirmation_status": "finalized",
      "account": "7xJ5k2m8QJK9xnFhZwkJ..."
    }
  ]
}
```

#### Example - Bitcoin

```bash
curl http://localhost:3001/api/v1/nodes/bitcoin:test/transactions
```

```json
{
  "success": true,
  "data": [
    {
      "signature": "a1b2c3d4e5f6...",
      "slot": 101,
      "err": null,
      "memo": null,
      "block_time": 1700000000,
      "confirmation_status": "6 confirmations",
      "account": "bcrt1qxyz..."
    }
  ]
}
```

#### Errors

| Status | Error                              | Description                    |
|--------|-----------------------------------|--------------------------------|
| 404    | "Node not found"                  | Node ID doesn't exist          |
| 503    | "Solana validator is not running" | Node not running               |
| 503    | "Bitcoin node is not running"     | Node not running               |
| 500    | "Failed to load accounts"        | Accounts file not found        |
| 500    | "Failed to list transactions"    | RPC call failed                |

::: tip
For Bitcoin, transactions are fetched from the wallet and filtered to only those involving known accounts. Due to `"timestamp": "now"` on descriptor import, initial funding transactions (sent before account import) may only appear as "send" entries rather than "receive" entries per account.
:::

---

### Get Transaction Detail

Returns detailed information about a specific transaction, including fee and per-account balance changes.

```
GET /api/v1/nodes/{node_id}/transactions/{signature}
```

#### Parameters

| Parameter | Type   | Description                                    |
|-----------|--------|------------------------------------------------|
| node_id   | string | Node identifier                                |
| signature | string | Transaction signature (Solana) or txid (Bitcoin) |

#### Response

```typescript
interface TransactionDetailInfo {
  signature: string;           // Transaction signature or txid
  slot: number;                // Slot or block height
  block_time: number | null;   // Unix timestamp
  fee: number;                 // Transaction fee (SOL or BTC, always positive)
  err: string | null;          // Error message if failed
  balance_changes: BalanceChange[];
}

interface BalanceChange {
  account: string;   // Account address
  before: number;    // Balance before transaction
  after: number;     // Balance after transaction
  change: number;    // Balance change (positive = received, negative = sent)
}
```

#### Example - Solana

```bash
curl http://localhost:3001/api/v1/nodes/solana:dev/transactions/5K8sH7jQ2nM...
```

```json
{
  "success": true,
  "data": {
    "signature": "5K8sH7jQ2nM...",
    "slot": 42,
    "block_time": 1700000000,
    "fee": 0.000005,
    "err": null,
    "balance_changes": [
      {
        "account": "7xJ5k2m8...",
        "before": 0.0,
        "after": 100.0,
        "change": 100.0
      }
    ]
  }
}
```

#### Example - Bitcoin

```bash
curl http://localhost:3001/api/v1/nodes/bitcoin:test/transactions/a1b2c3d4e5f6...
```

```json
{
  "success": true,
  "data": {
    "signature": "a1b2c3d4e5f6...",
    "slot": 101,
    "block_time": 1700000000,
    "fee": 0.00005,
    "err": null,
    "balance_changes": [
      {
        "account": "bcrt1qsender...",
        "before": 1.0,
        "after": 0.0,
        "change": -1.0
      },
      {
        "account": "bcrt1qreceiver...",
        "before": 0.0,
        "after": 1.0,
        "change": 1.0
      }
    ]
  }
}
```

#### Errors

| Status | Error                              | Description                    |
|--------|-----------------------------------|--------------------------------|
| 404    | "Node not found"                  | Node ID doesn't exist          |
| 503    | "Solana validator is not running" | Node not running               |
| 503    | "Bitcoin node is not running"     | Node not running               |
| 500    | "Failed to get transaction"       | Transaction not found or RPC error |

::: tip
For Bitcoin, the `fee` field is always returned as a positive number (Bitcoin Core reports fees as negative values internally). The `slot` field maps to Bitcoin's block height.
:::

---

## CORS

The API server has CORS enabled by default, allowing requests from any origin. This enables the web dashboard (running on a different port) to communicate with the API.

---

## Error Handling

### HTTP Status Codes

| Code | Meaning               | Description                          |
|------|-----------------------|--------------------------------------|
| 200  | OK                    | Request successful                   |
| 400  | Bad Request           | Invalid request parameters           |
| 404  | Not Found             | Resource not found                   |
| 500  | Internal Server Error | Server-side error                    |
| 503  | Service Unavailable   | Node not running                     |

### Error Response Format

All errors return a JSON response:

```json
{
  "success": false,
  "error": "Descriptive error message"
}
```

---

## Integration Examples

### JavaScript/TypeScript

```typescript
const API_BASE = 'http://localhost:3001/api/v1';

// List all nodes
async function listNodes() {
  const response = await fetch(`${API_BASE}/nodes`);
  const data = await response.json();

  if (data.success) {
    return data.data;
  }
  throw new Error(data.error);
}

// Get accounts for a node
async function getAccounts(nodeId: string) {
  const response = await fetch(`${API_BASE}/nodes/${encodeURIComponent(nodeId)}/accounts`);
  const data = await response.json();

  if (data.success) {
    return data.data;
  }
  throw new Error(data.error);
}

// Fund an account
async function fundAccount(nodeId: string, address: string, amount: number) {
  const response = await fetch(`${API_BASE}/nodes/${encodeURIComponent(nodeId)}/fund`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ address, amount }),
  });
  const data = await response.json();

  if (data.success) {
    return data.data;
  }
  throw new Error(data.error);
}

// Get recent transactions for a node
async function getTransactions(nodeId: string) {
  const response = await fetch(`${API_BASE}/nodes/${encodeURIComponent(nodeId)}/transactions`);
  const data = await response.json();

  if (data.success) {
    return data.data;
  }
  throw new Error(data.error);
}

// Get transaction detail
async function getTransactionDetail(nodeId: string, signature: string) {
  const response = await fetch(
    `${API_BASE}/nodes/${encodeURIComponent(nodeId)}/transactions/${encodeURIComponent(signature)}`
  );
  const data = await response.json();

  if (data.success) {
    return data.data;
  }
  throw new Error(data.error);
}
```

### Python

```python
import requests

API_BASE = 'http://localhost:3001/api/v1'

def list_nodes():
    response = requests.get(f'{API_BASE}/nodes')
    data = response.json()
    if data['success']:
        return data['data']
    raise Exception(data['error'])

def get_accounts(node_id):
    response = requests.get(f'{API_BASE}/nodes/{node_id}/accounts')
    data = response.json()
    if data['success']:
        return data['data']
    raise Exception(data['error'])

def fund_account(node_id, address, amount):
    response = requests.post(
        f'{API_BASE}/nodes/{node_id}/fund',
        json={'address': address, 'amount': amount}
    )
    data = response.json()
    if data['success']:
        return data['data']
    raise Exception(data['error'])

def get_transactions(node_id):
    response = requests.get(f'{API_BASE}/nodes/{node_id}/transactions')
    data = response.json()
    if data['success']:
        return data['data']
    raise Exception(data['error'])

def get_transaction_detail(node_id, signature):
    response = requests.get(f'{API_BASE}/nodes/{node_id}/transactions/{signature}')
    data = response.json()
    if data['success']:
        return data['data']
    raise Exception(data['error'])

# Usage
nodes = list_nodes()
for node in nodes:
    print(f"{node['name'] or node['instance_id']}: {node['status']}")

    if node['status'] == 'running':
        accounts = get_accounts(node['node_id'])
        for acc in accounts:
            print(f"  Account {acc['index']}: {acc['balance']}")
```

### cURL

```bash
# List all nodes
curl http://localhost:3001/api/v1/nodes | jq

# Get specific node
curl http://localhost:3001/api/v1/nodes/solana:dev | jq

# Get accounts
curl http://localhost:3001/api/v1/nodes/solana:dev/accounts | jq

# Health check
curl -X POST http://localhost:3001/api/v1/health | jq

# Fund account
curl -X POST http://localhost:3001/api/v1/nodes/solana:dev/fund \
  -H "Content-Type: application/json" \
  -d '{"address": "...", "amount": 50}' | jq

# List transactions
curl http://localhost:3001/api/v1/nodes/solana:dev/transactions | jq

# Get transaction detail
curl http://localhost:3001/api/v1/nodes/solana:dev/transactions/SIGNATURE_HERE | jq
```

---

## Node ID Format

Node IDs follow the format: `{chain}:{instance_id}`

| Chain   | Example Node ID     |
|---------|---------------------|
| Solana  | `solana:default`    |
| Solana  | `solana:dev`        |
| Bitcoin | `bitcoin:default`   |
| Bitcoin | `bitcoin:btc-test`  |

---

## Next Steps

- [Solana CLI Reference](../solana/cli) - Start Solana nodes
- [Bitcoin CLI Reference](../bitcoin/cli) - Start Bitcoin nodes
- [TypeScript API](./overview) - TypeScript client library
