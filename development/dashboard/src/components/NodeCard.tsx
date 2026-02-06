// Node card component for the dashboard grid

import { useNavigate } from 'react-router-dom';
import type { NodeInfo } from '../api/types';
import { NodeStatus } from './NodeStatus';
import { useStopNode } from '../api/hooks';

interface NodeCardProps {
  node: NodeInfo;
}

export function NodeCard({ node }: NodeCardProps) {
  const navigate = useNavigate();
  const stopMutation = useStopNode();

  const chainColors = {
    solana: {
      gradient: 'from-purple-500 to-purple-700',
      accent: 'border-purple-500',
      badge: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
    },
    bitcoin: {
      gradient: 'from-orange-500 to-orange-700',
      accent: 'border-orange-500',
      badge: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
    },
  };

  const colors = chainColors[node.chain];
  const displayName = node.name || node.instance_id;

  const handleCardClick = () => {
    navigate(`/nodes/${encodeURIComponent(node.node_id)}`);
  };

  const handleStop = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent card click
    if (confirm(`Stop node "${displayName}"?`)) {
      stopMutation.mutate(node.node_id);
    }
  };

  return (
    <div
      onClick={handleCardClick}
      className={`bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden border-t-4 ${colors.accent} transition-shadow hover:shadow-lg cursor-pointer`}
    >
      {/* Header */}
      <div className={`bg-gradient-to-r ${colors.gradient} px-4 py-3`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h3 className="text-white font-semibold truncate">{displayName}</h3>
            <NodeStatus status={node.status} />
          </div>
          <span
            className={`text-xs px-2 py-1 rounded-full uppercase font-medium ${colors.badge}`}
          >
            {node.chain}
          </span>
        </div>
      </div>

      {/* Body */}
      <div className="p-4">
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <span className="text-gray-500 dark:text-gray-400 text-sm">
              Port
            </span>
            <span className="font-mono text-sm">{node.rpc_port}</span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-gray-500 dark:text-gray-400 text-sm">
              Accounts
            </span>
            <span className="font-mono text-sm">{node.accounts_count}</span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-gray-500 dark:text-gray-400 text-sm">
              Instance
            </span>
            <span className="font-mono text-xs text-gray-600 dark:text-gray-300 truncate max-w-[120px]">
              {node.instance_id}
            </span>
          </div>
        </div>

        {/* Actions */}
        {node.status === 'running' && (
          <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
            <button
              onClick={handleStop}
              disabled={stopMutation.isPending}
              className="w-full px-3 py-2 text-sm bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 hover:bg-red-200 dark:hover:bg-red-800 rounded transition-colors disabled:opacity-50"
            >
              {stopMutation.isPending ? 'Stopping...' : 'Stop Node'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
