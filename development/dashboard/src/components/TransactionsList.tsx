// Transactions list component with expandable detail rows

import { useState } from 'react';
import { useNodeTransactions, useTransactionDetail } from '../api/hooks';
import type { TransactionInfo, BalanceChange } from '../api/types';

interface TransactionsListProps {
  nodeId: string;
  chain: 'solana' | 'bitcoin';
}

function truncate(str: string, len = 8): string {
  if (str.length <= len * 2 + 3) return str;
  return `${str.slice(0, len)}...${str.slice(-len)}`;
}

function formatTimestamp(blockTime: number | null): string {
  if (!blockTime) return '-';
  return new Date(blockTime * 1000).toLocaleString();
}

function formatAmount(value: number, chain: string): string {
  const decimals = chain === 'bitcoin' ? 8 : 9;
  return value.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: decimals,
  });
}

function currencyUnit(chain: string): string {
  return chain === 'bitcoin' ? 'BTC' : 'SOL';
}

function txLabel(chain: string): string {
  return chain === 'bitcoin' ? 'TxID' : 'Signature';
}

function blockLabel(chain: string): string {
  return chain === 'bitcoin' ? 'Block' : 'Slot';
}

function StatusBadge({ err }: { err: string | null }) {
  if (err) {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300">
        Failed
      </span>
    );
  }
  return (
    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
      Success
    </span>
  );
}

function ChangeIndicator({ change, chain }: { change: number; chain: string }) {
  const unit = currencyUnit(chain);
  if (change > 0) {
    return (
      <span className="text-green-600 dark:text-green-400">
        +{formatAmount(change, chain)} {unit}
      </span>
    );
  }
  if (change < 0) {
    return (
      <span className="text-red-600 dark:text-red-400">
        {formatAmount(change, chain)} {unit}
      </span>
    );
  }
  return <span className="text-gray-400">0 {unit}</span>;
}

function BalanceChangesTable({ changes, chain }: { changes: BalanceChange[]; chain: string }) {
  return (
    <table className="min-w-full text-sm">
      <thead>
        <tr className="text-xs text-gray-500 dark:text-gray-400 uppercase">
          <th className="text-left py-1 pr-4">Account</th>
          <th className="text-right py-1 pr-4">Before</th>
          <th className="text-right py-1 pr-4">After</th>
          <th className="text-right py-1">Change</th>
        </tr>
      </thead>
      <tbody>
        {changes.map((bc) => (
          <tr key={bc.account} className="border-t border-gray-100 dark:border-gray-700">
            <td className="py-1 pr-4">
              <code className="font-mono text-xs text-gray-700 dark:text-gray-300">
                {truncate(bc.account, 6)}
              </code>
            </td>
            <td className="text-right py-1 pr-4 font-mono text-xs text-gray-500 dark:text-gray-400">
              {formatAmount(bc.before, chain)}
            </td>
            <td className="text-right py-1 pr-4 font-mono text-xs text-gray-500 dark:text-gray-400">
              {formatAmount(bc.after, chain)}
            </td>
            <td className="text-right py-1 font-mono text-xs">
              <ChangeIndicator change={bc.change} chain={chain} />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function TransactionDetailRow({
  nodeId,
  signature,
  chain,
}: {
  nodeId: string;
  signature: string;
  chain: string;
}) {
  const { data, isLoading } = useTransactionDetail(nodeId, signature);

  if (isLoading) {
    return (
      <tr>
        <td colSpan={6} className="px-4 py-3">
          <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
            Loading details...
          </div>
        </td>
      </tr>
    );
  }

  if (!data?.success || !data.data) {
    return (
      <tr>
        <td colSpan={6} className="px-4 py-3">
          <p className="text-sm text-red-500">
            {data?.error || 'Failed to load transaction details'}
          </p>
        </td>
      </tr>
    );
  }

  const detail = data.data;

  return (
    <tr>
      <td colSpan={6} className="px-4 py-3 bg-gray-50 dark:bg-gray-800/50">
        <div className="space-y-3">
          {/* Fee and signature/txid */}
          <div className="flex flex-wrap gap-x-6 gap-y-1 text-sm">
            <div>
              <span className="text-gray-500 dark:text-gray-400">Fee: </span>
              <span className="font-mono text-gray-900 dark:text-gray-100">
                {formatAmount(detail.fee, chain)} {currencyUnit(chain)}
              </span>
            </div>
            <div>
              <span className="text-gray-500 dark:text-gray-400">
                {txLabel(chain)}:{' '}
              </span>
              <code className="font-mono text-xs text-gray-700 dark:text-gray-300 break-all">
                {detail.signature}
              </code>
            </div>
          </div>

          {/* Balance changes */}
          {detail.balance_changes.length > 0 ? (
            <div>
              <h4 className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase mb-1">
                Balance Changes
              </h4>
              <BalanceChangesTable changes={detail.balance_changes} chain={chain} />
            </div>
          ) : (
            <p className="text-sm text-gray-400 dark:text-gray-500">
              No balance changes
            </p>
          )}
        </div>
      </td>
    </tr>
  );
}

function TransactionsTable({
  transactions,
  nodeId,
  chain,
}: {
  transactions: TransactionInfo[];
  nodeId: string;
  chain: string;
}) {
  const [expandedSig, setExpandedSig] = useState<string | null>(null);

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead className="bg-gray-50 dark:bg-gray-800">
          <tr>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-8"></th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              {txLabel(chain)}
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              {blockLabel(chain)}
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Time
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Status
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Account
            </th>
          </tr>
        </thead>
        <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
          {transactions.map((tx) => {
            const isExpanded = expandedSig === tx.signature;
            return (
              <>
                <tr
                  key={tx.signature}
                  className="hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer"
                  onClick={() =>
                    setExpandedSig(isExpanded ? null : tx.signature)
                  }
                >
                  <td className="px-4 py-3 whitespace-nowrap text-gray-400">
                    <svg
                      className={`w-4 h-4 transition-transform ${isExpanded ? 'rotate-90' : ''}`}
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M9 5l7 7-7 7"
                      />
                    </svg>
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap">
                    <code className="text-sm text-gray-900 dark:text-gray-100 font-mono">
                      {truncate(tx.signature)}
                    </code>
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400 font-mono">
                    {tx.slot.toLocaleString()}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                    {formatTimestamp(tx.block_time)}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap">
                    <StatusBadge err={tx.err} />
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap">
                    <code className="text-xs text-gray-500 dark:text-gray-400 font-mono">
                      {truncate(tx.account)}
                    </code>
                  </td>
                </tr>
                {isExpanded && (
                  <TransactionDetailRow
                    key={`${tx.signature}-detail`}
                    nodeId={nodeId}
                    signature={tx.signature}
                    chain={chain}
                  />
                )}
              </>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export function TransactionsList({ nodeId, chain }: TransactionsListProps) {
  const { data, isLoading } = useNodeTransactions(nodeId);

  if (isLoading) {
    return (
      <div className="p-8 text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
        <p className="mt-2 text-gray-500 dark:text-gray-400">
          Loading transactions...
        </p>
      </div>
    );
  }

  const transactions = data?.success ? data.data ?? [] : [];

  if (data && !data.success) {
    return (
      <div className="p-8 text-center">
        <p className="text-red-500">
          {data.error || 'Failed to load transactions'}
        </p>
      </div>
    );
  }

  if (transactions.length === 0) {
    return (
      <div className="p-8 text-center">
        <div className="text-gray-400 dark:text-gray-500 mb-4">
          <svg
            className="w-16 h-16 mx-auto"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"
            />
          </svg>
        </div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
          No Transactions Yet
        </h3>
        <p className="text-gray-500 dark:text-gray-400 text-sm max-w-md mx-auto">
          Transactions will appear here as activity occurs on this node's
          accounts. Try funding an account to see the first transaction.
        </p>
      </div>
    );
  }

  return <TransactionsTable transactions={transactions} nodeId={nodeId} chain={chain} />;
}
