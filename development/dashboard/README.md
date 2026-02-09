# Chain Forge Dashboard

A web-based dashboard for monitoring and managing Chain Forge blockchain nodes (Solana and Bitcoin).

## Features

- View all running nodes in a grid layout
- Monitor node status (running/stopped) with auto-refresh
- View account balances for each node
- Start new Solana or Bitcoin nodes
- Stop running nodes
- Fund accounts with additional balance
- Dark/light theme support with system preference detection
- Chain-specific theming (purple for Solana, orange for Bitcoin)

## Prerequisites

- Node.js 18+ or 20+
- Yarn or npm
- Chain Forge API server running (`cf-api`)

## Quick Start

### 1. Start the API Server

First, build and run the Chain Forge dashboard API server:

```bash
# From the project root
cargo build --release -p chain-forge-api-server

# Run the API server (default port: 3001)
./target/release/cf-api

# Or with a custom port
./target/release/cf-api --port 3002
```

### 2. Start the Dashboard

```bash
# Navigate to the dashboard directory
cd development/dashboard

# Install dependencies
yarn install
# or
npm install

# Start the development server
yarn dev
# or
npm run dev
```

The dashboard will be available at http://localhost:5173

## Available Scripts

| Command | Description |
|---------|-------------|
| `yarn dev` | Start development server with hot reload |
| `yarn build` | Build for production |
| `yarn preview` | Preview production build locally |
| `yarn lint` | Run ESLint |

## Configuration

### API Server URL

By default, the dashboard connects to the API server at `http://localhost:3001`. To change this, modify the `API_BASE_URL` in `src/api/client.ts`:

```typescript
const API_BASE_URL = 'http://localhost:3001/api/v1';
```

### Auto-refresh Interval

The dashboard automatically refreshes node data every 5 seconds. This can be configured in `src/api/hooks.ts` by modifying the `refetchInterval` option:

```typescript
refetchInterval: 5000, // milliseconds
```

## Project Structure

```
development/dashboard/
├── src/
│   ├── api/
│   │   ├── client.ts      # API fetch functions
│   │   ├── hooks.ts       # React Query hooks
│   │   └── types.ts       # TypeScript type definitions
│   ├── components/
│   │   ├── AccountsList.tsx   # Account table with fund action
│   │   ├── NewNodeForm.tsx    # Form to start new nodes
│   │   ├── NodeCard.tsx       # Individual node card
│   │   ├── NodeGrid.tsx       # Grid layout for nodes
│   │   └── NodeStatus.tsx     # Status indicator component
│   ├── pages/
│   │   ├── Dashboard.tsx      # Main dashboard view
│   │   └── NodeDetail.tsx     # Individual node details
│   ├── App.tsx                # Main app with routing
│   ├── main.tsx               # Entry point
│   └── index.css              # Tailwind CSS imports
├── index.html
├── package.json
├── tailwind.config.js
├── tsconfig.json
└── vite.config.ts
```

## API Endpoints

The dashboard communicates with the following API endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/nodes` | List all registered nodes |
| GET | `/api/v1/nodes/{node_id}` | Get specific node details |
| GET | `/api/v1/nodes/{node_id}/accounts` | Get accounts for a node |
| POST | `/api/v1/health` | Trigger health check for all nodes |
| POST | `/api/v1/nodes` | Start a new node |
| DELETE | `/api/v1/nodes/{node_id}` | Stop a node |
| POST | `/api/v1/nodes/{node_id}/fund` | Fund an account |

## Running Nodes

Before using the dashboard, you need to start some blockchain nodes:

### Start a Solana Node

```bash
# Install the CLI if not already installed
cargo install --path chains/solana/crates/cli

# Start a Solana node
cf-solana start --instance dev --name "Development" --accounts 5 --balance 100
```

### Start a Bitcoin Node

```bash
# Install the CLI if not already installed
cargo install --path chains/bitcoin/crates/cli

# Start a Bitcoin node (requires bitcoind installed)
cf-bitcoin start --instance btc-dev --name "Bitcoin Dev" --accounts 5 --balance 10
```

## Theme Support

The dashboard supports both dark and light themes:

- Automatically detects system preference on first load
- Toggle between themes using the button in the header
- Theme preference is saved to localStorage

## Technology Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool and dev server
- **TailwindCSS** - Styling
- **React Query** - Data fetching and caching
- **React Router** - Client-side routing

## Troubleshooting

### Dashboard shows "No nodes registered"

Make sure you have:
1. Started the API server (`cf-api`)
2. Started at least one blockchain node (`cf-solana start` or `cf-bitcoin start`)

### API connection errors

Verify that:
1. The API server is running on the expected port
2. CORS is enabled (it is by default)
3. The `API_BASE_URL` in `src/api/client.ts` matches your API server address

### Node shows as "stopped" but is actually running

Click the "Health Check" button to manually refresh node statuses, or wait for the auto-refresh cycle.
