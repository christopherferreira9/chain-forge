// API Response types

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface NodeInfo {
  node_id: string;
  name: string | null;
  chain: 'solana' | 'bitcoin';
  instance_id: string;
  rpc_url: string;
  rpc_port: number;
  accounts_count: number;
  status: 'running' | 'stopped' | 'unknown';
  started_at: string | null;
}

export interface AccountInfo {
  index: number;
  address: string;
  balance: number;
}

export interface TransactionInfo {
  signature: string;
  slot: number;
  err: string | null;
  memo: string | null;
  block_time: number | null;
  confirmation_status: string | null;
  account: string;
}

export interface BalanceChange {
  account: string;
  before: number;
  after: number;
  change: number;
}

export interface TransactionDetail {
  signature: string;
  slot: number;
  block_time: number | null;
  fee: number;
  err: string | null;
  balance_changes: BalanceChange[];
}

export interface HealthCheckResponse {
  total: number;
  running: number;
  stopped: number;
  unknown: number;
}

export interface StartNodeRequest {
  chain: 'solana' | 'bitcoin';
  instance?: string;
  name?: string;
  port?: number;
  accounts?: number;
  balance?: number;
}

export interface StartNodeResponse {
  message: string;
  command: string;
  chain: string;
  instance: string;
  port: number;
}

export interface StopNodeResponse {
  message: string;
  instruction: string;
  node_id: string;
}

export interface FundAccountRequest {
  address: string;
  amount: number;
}

export interface FundResponse {
  success: boolean;
  txid_or_signature: string;
  address: string;
  amount: number;
}

export interface CleanupResponse {
  removed: number;
  remaining: number;
  removed_nodes: string[];
}
