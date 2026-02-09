// API client for Chain Forge Dashboard

import type {
  ApiResponse,
  NodeInfo,
  AccountInfo,
  HealthCheckResponse,
  StartNodeRequest,
  StartNodeResponse,
  StopNodeResponse,
  FundAccountRequest,
  FundResponse,
  CleanupResponse,
  TransactionInfo,
  TransactionDetail,
} from './types';

const API_BASE = '/api/v1';

async function fetchApi<T>(
  endpoint: string,
  options?: RequestInit
): Promise<ApiResponse<T>> {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    });

    if (!response.ok) {
      // Try to parse error from response
      try {
        const errorData = await response.json();
        return {
          success: false,
          error: errorData.error || `HTTP ${response.status}: ${response.statusText}`,
        };
      } catch {
        return {
          success: false,
          error: `HTTP ${response.status}: ${response.statusText}`,
        };
      }
    }

    const data = await response.json();
    return data;
  } catch (err) {
    // Network error or other fetch failure
    const message = err instanceof Error ? err.message : 'Unknown error';
    return {
      success: false,
      error: `Failed to connect to API: ${message}`,
    };
  }
}

// List all nodes
export async function listNodes(): Promise<ApiResponse<NodeInfo[]>> {
  return fetchApi<NodeInfo[]>('/nodes');
}

// Get a specific node
export async function getNode(nodeId: string): Promise<ApiResponse<NodeInfo>> {
  return fetchApi<NodeInfo>(`/nodes/${encodeURIComponent(nodeId)}`);
}

// Get accounts for a node
export async function getNodeAccounts(
  nodeId: string
): Promise<ApiResponse<AccountInfo[]>> {
  return fetchApi<AccountInfo[]>(
    `/nodes/${encodeURIComponent(nodeId)}/accounts`
  );
}

// Health check all nodes
export async function healthCheck(): Promise<ApiResponse<HealthCheckResponse>> {
  return fetchApi<HealthCheckResponse>('/health', {
    method: 'POST',
  });
}

// Start a new node (returns CLI instructions)
export async function startNode(
  request: StartNodeRequest
): Promise<ApiResponse<StartNodeResponse>> {
  return fetchApi<StartNodeResponse>('/nodes', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// Stop a node (marks as stopped)
export async function stopNode(
  nodeId: string
): Promise<ApiResponse<StopNodeResponse>> {
  return fetchApi<StopNodeResponse>(`/nodes/${encodeURIComponent(nodeId)}`, {
    method: 'DELETE',
  });
}

// Get transactions for a node
export async function getNodeTransactions(
  nodeId: string
): Promise<ApiResponse<TransactionInfo[]>> {
  return fetchApi<TransactionInfo[]>(
    `/nodes/${encodeURIComponent(nodeId)}/transactions`
  );
}

// Get transaction detail
export async function getTransactionDetail(
  nodeId: string,
  signature: string
): Promise<ApiResponse<TransactionDetail>> {
  return fetchApi<TransactionDetail>(
    `/nodes/${encodeURIComponent(nodeId)}/transactions/${encodeURIComponent(signature)}`
  );
}

// Clean up registry (remove non-running nodes)
export async function cleanupRegistry(): Promise<ApiResponse<CleanupResponse>> {
  return fetchApi<CleanupResponse>('/registry/cleanup', {
    method: 'POST',
  });
}

// Fund an account
export async function fundAccount(
  nodeId: string,
  request: FundAccountRequest
): Promise<ApiResponse<FundResponse>> {
  return fetchApi<FundResponse>(
    `/nodes/${encodeURIComponent(nodeId)}/fund`,
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
}
