// Grid layout for node cards

import type { NodeInfo } from '../api/types';
import { NodeCard } from './NodeCard';

interface NodeGridProps {
  nodes: NodeInfo[];
}

export function NodeGrid({ nodes }: NodeGridProps) {
  if (nodes.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="text-gray-400 dark:text-gray-500 text-6xl mb-4">
          <span role="img" aria-label="No nodes">
            <svg
              className="mx-auto h-16 w-16"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
              />
            </svg>
          </span>
        </div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
          No nodes running
        </h3>
        <p className="mt-2 text-gray-500 dark:text-gray-400">
          Start a Solana or Bitcoin node using the CLI or the "New Node" button above.
        </p>
        <div className="mt-4 text-sm text-gray-400 dark:text-gray-500 font-mono">
          <code>cf-solana start --name "My-Node"</code>
        </div>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {nodes.map((node) => (
        <NodeCard key={node.node_id} node={node} />
      ))}
    </div>
  );
}
