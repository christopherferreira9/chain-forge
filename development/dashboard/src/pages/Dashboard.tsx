// Main dashboard page

import { useState } from 'react';
import { useNodes, useHealthCheck, useCleanupRegistry } from '../api/hooks';
import { NodeGrid } from '../components/NodeGrid';
import { NewNodeForm } from '../components/NewNodeForm';

export function Dashboard() {
  const [showNewNodeForm, setShowNewNodeForm] = useState(false);
  const { data, isLoading, error } = useNodes();
  const healthCheck = useHealthCheck();
  const cleanup = useCleanupRegistry();

  const nodes = (data?.success ? data.data ?? [] : []).sort((a, b) => {
    // Sort by started_at descending (most recent first), nulls last
    if (!a.started_at && !b.started_at) return 0;
    if (!a.started_at) return 1;
    if (!b.started_at) return -1;
    return new Date(b.started_at).getTime() - new Date(a.started_at).getTime();
  });
  const apiError = data && !data.success ? data.error : null;
  const runningCount = nodes.filter((n) => n.status === 'running').length;
  const stoppedCount = nodes.filter((n) => n.status === 'stopped').length;
  const solanaCount = nodes.filter((n) => n.chain === 'solana').length;
  const bitcoinCount = nodes.filter((n) => n.chain === 'bitcoin').length;

  return (
    <div>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-8">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Node Dashboard
          </h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1">
            Monitor and manage your blockchain nodes
          </p>
        </div>

        <div className="flex gap-3">
          <button
            onClick={() => healthCheck.mutate()}
            disabled={healthCheck.isPending}
            className="px-4 py-2 text-sm font-medium bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50"
          >
            {healthCheck.isPending ? 'Checking...' : 'Refresh Status'}
          </button>
          {nodes.length > 0 && (
            <button
              onClick={() => cleanup.mutate()}
              disabled={cleanup.isPending}
              className="px-4 py-2 text-sm font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 hover:bg-red-200 dark:hover:bg-red-900/50 rounded-lg transition-colors disabled:opacity-50"
            >
              {cleanup.isPending ? 'Cleaning...' : 'Clean Registry'}
            </button>
          )}
          <button
            onClick={() => setShowNewNodeForm(true)}
            className="px-4 py-2 text-sm font-medium bg-blue-600 text-white hover:bg-blue-700 rounded-lg transition-colors"
          >
            + New Node
          </button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            {nodes.length}
          </div>
          <div className="text-sm text-gray-500 dark:text-gray-400">
            Total Nodes
          </div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-3xl font-bold text-green-600">{runningCount}</div>
          <div className="text-sm text-gray-500 dark:text-gray-400">Running</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-3xl font-bold text-red-600">{stoppedCount}</div>
          <div className="text-sm text-gray-500 dark:text-gray-400">Stopped</div>
        </div>
      </div>

      {/* Chain breakdown breadcrumb */}
      {nodes.length > 0 && (
        <div className="flex items-center gap-2 text-sm mb-6">
          <span className="text-gray-500 dark:text-gray-400">Chains:</span>
          {solanaCount > 0 && (
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 font-medium">
              <span className="w-2 h-2 rounded-full bg-purple-500"></span>
              {solanaCount} Solana
            </span>
          )}
          {bitcoinCount > 0 && (
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-300 font-medium">
              <span className="w-2 h-2 rounded-full bg-orange-500"></span>
              {bitcoinCount} Bitcoin
            </span>
          )}
        </div>
      )}

      {/* Content */}
      {isLoading ? (
        <div className="text-center py-12">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-500 dark:text-gray-400">Loading nodes...</p>
        </div>
      ) : error || apiError ? (
        <div className="text-center py-12">
          <div className="text-red-500 text-lg">Failed to load nodes</div>
          <p className="text-gray-500 dark:text-gray-400 mt-2">
            {apiError || 'Make sure the API server is running on port 3001'}
          </p>
          <code className="block mt-4 text-sm text-gray-400">
            cf-api --port 3001
          </code>
        </div>
      ) : (
        <NodeGrid nodes={nodes} />
      )}

      {/* New Node Modal */}
      {showNewNodeForm && (
        <NewNodeForm onClose={() => setShowNewNodeForm(false)} />
      )}
    </div>
  );
}
