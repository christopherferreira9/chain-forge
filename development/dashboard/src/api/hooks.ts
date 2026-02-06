// React Query hooks for Chain Forge API

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as api from './client';
import type { StartNodeRequest, FundAccountRequest } from './types';

// List all nodes with auto-refresh
export function useNodes() {
  return useQuery({
    queryKey: ['nodes'],
    queryFn: api.listNodes,
    refetchInterval: 5000, // Auto-refresh every 5 seconds
  });
}

// Get a specific node
export function useNode(nodeId: string) {
  return useQuery({
    queryKey: ['node', nodeId],
    queryFn: () => api.getNode(nodeId),
    enabled: !!nodeId,
  });
}

// Get accounts for a node
export function useNodeAccounts(nodeId: string) {
  return useQuery({
    queryKey: ['accounts', nodeId],
    queryFn: () => api.getNodeAccounts(nodeId),
    enabled: !!nodeId,
  });
}

// Health check mutation
export function useHealthCheck() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: api.healthCheck,
    onSuccess: () => {
      // Refresh nodes list after health check
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

// Start node mutation
export function useStartNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: StartNodeRequest) => api.startNode(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

// Stop node mutation
export function useStopNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (nodeId: string) => api.stopNode(nodeId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

// Get transactions for a node with auto-refresh
export function useNodeTransactions(nodeId: string) {
  return useQuery({
    queryKey: ['transactions', nodeId],
    queryFn: () => api.getNodeTransactions(nodeId),
    enabled: !!nodeId,
    refetchInterval: 10000,
  });
}

// Get transaction detail (fetched on demand)
export function useTransactionDetail(nodeId: string, signature: string | null) {
  return useQuery({
    queryKey: ['transaction-detail', nodeId, signature],
    queryFn: () => api.getTransactionDetail(nodeId, signature!),
    enabled: !!nodeId && !!signature,
  });
}

// Cleanup registry mutation
export function useCleanupRegistry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: api.cleanupRegistry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
    },
  });
}

// Fund account mutation
export function useFundAccount(nodeId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: FundAccountRequest) =>
      api.fundAccount(nodeId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts', nodeId] });
    },
  });
}
