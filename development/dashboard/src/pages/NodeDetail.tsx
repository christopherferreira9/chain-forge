// Node detail page with accounts and transactions

import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useNode, useNodeAccounts, useStopNode } from '../api/hooks';
import { NodeStatus } from '../components/NodeStatus';
import { AccountsList } from '../components/AccountsList';
import { TransactionsList } from '../components/TransactionsList';

type Tab = 'accounts' | 'transactions';

export function NodeDetail() {
  const { nodeId } = useParams<{ nodeId: string }>();
  const [activeTab, setActiveTab] = useState<Tab>('accounts');
  const { data: nodeData, isLoading: nodeLoading } = useNode(nodeId ?? '');
  const { data: accountsData, isLoading: accountsLoading } = useNodeAccounts(
    nodeId ?? ''
  );
  const stopMutation = useStopNode();

  if (!nodeId) {
    return <div>Invalid node ID</div>;
  }

  const node = nodeData?.success ? nodeData.data : null;
  const accounts = accountsData?.success ? accountsData.data ?? [] : [];

  const chainColors = {
    solana: {
      gradient: 'from-purple-500 to-purple-700',
      badge: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
    },
    bitcoin: {
      gradient: 'from-orange-500 to-orange-700',
      badge: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
    },
  };

  if (nodeLoading) {
    return (
      <div className="text-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
        <p className="mt-4 text-gray-500 dark:text-gray-400">Loading node...</p>
      </div>
    );
  }

  if (!node) {
    return (
      <div className="text-center py-12">
        <div className="text-red-500 text-lg">Node not found</div>
        <Link
          to="/"
          className="mt-4 inline-block text-blue-600 hover:underline"
        >
          Back to Dashboard
        </Link>
      </div>
    );
  }

  const colors = chainColors[node.chain];
  const displayName = node.name || node.instance_id;

  const handleStop = () => {
    if (confirm(`Stop node "${displayName}"?`)) {
      stopMutation.mutate(node.node_id);
    }
  };

  return (
    <div>
      {/* Back Link */}
      <Link
        to="/"
        className="inline-flex items-center text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 mb-6"
      >
        <svg
          className="w-4 h-4 mr-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15 19l-7-7 7-7"
          />
        </svg>
        Back to Dashboard
      </Link>

      {/* Node Header */}
      <div
        className={`bg-gradient-to-r ${colors.gradient} rounded-lg p-6 mb-6`}
      >
        <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
          <div>
            <div className="flex items-center gap-3 mb-2">
              <h1 className="text-2xl font-bold text-white">{displayName}</h1>
              <NodeStatus status={node.status} />
              <span className={`text-xs px-2 py-1 rounded-full uppercase font-medium ${colors.badge}`}>
                {node.chain}
              </span>
            </div>
            <div className="text-white/80 font-mono text-sm">{node.node_id}</div>
          </div>
          <div className="flex items-center gap-4">
            {node.status === 'running' && (
              <button
                onClick={handleStop}
                disabled={stopMutation.isPending}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 transition-colors"
              >
                {stopMutation.isPending ? 'Stopping...' : 'Stop Node'}
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Node Info Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
            RPC URL
          </div>
          <div className="font-mono text-sm text-gray-900 dark:text-gray-100 break-all">
            {node.rpc_url}
          </div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
            RPC Port
          </div>
          <div className="text-xl font-bold text-gray-900 dark:text-gray-100">
            {node.rpc_port}
          </div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
            Accounts
          </div>
          <div className="text-xl font-bold text-gray-900 dark:text-gray-100">
            {node.accounts_count}
          </div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">
            Started
          </div>
          <div className="text-sm text-gray-900 dark:text-gray-100">
            {node.started_at
              ? new Date(node.started_at).toLocaleString()
              : 'Unknown'}
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm overflow-hidden">
        <div className="border-b border-gray-200 dark:border-gray-700">
          <nav className="flex -mb-px">
            <button
              onClick={() => setActiveTab('accounts')}
              className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors ${
                activeTab === 'accounts'
                  ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                  : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300'
              }`}
            >
              Accounts
            </button>
            <button
              onClick={() => setActiveTab('transactions')}
              className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors ${
                activeTab === 'transactions'
                  ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                  : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300'
              }`}
            >
              Transactions
            </button>
          </nav>
        </div>

        {/* Tab Content */}
        {activeTab === 'accounts' && (
          <>
            {accountsLoading ? (
              <div className="p-8 text-center">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
                <p className="mt-2 text-gray-500 dark:text-gray-400">
                  Loading accounts...
                </p>
              </div>
            ) : accounts.length > 0 ? (
              <AccountsList
                nodeId={node.node_id}
                accounts={accounts}
                chain={node.chain}
              />
            ) : (
              <div className="p-8 text-center text-gray-500 dark:text-gray-400">
                No accounts found
              </div>
            )}
          </>
        )}

        {activeTab === 'transactions' && (
          <TransactionsList nodeId={node.node_id} chain={node.chain} />
        )}
      </div>
    </div>
  );
}
