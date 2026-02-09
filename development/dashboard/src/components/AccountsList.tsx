// Accounts list with fund functionality

import { useState } from 'react';
import type { AccountInfo } from '../api/types';
import { useFundAccount } from '../api/hooks';
import { useQueryClient } from '@tanstack/react-query';

interface AccountsListProps {
  nodeId: string;
  accounts: AccountInfo[];
  chain: 'solana' | 'bitcoin';
}

// Helper to copy text to clipboard with fallback
async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Fallback for browsers that don't support clipboard API
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.left = '-999999px';
    textArea.style.top = '-999999px';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
      document.execCommand('copy');
      document.body.removeChild(textArea);
      return true;
    } catch {
      document.body.removeChild(textArea);
      return false;
    }
  }
}

export function AccountsList({ nodeId, accounts, chain }: AccountsListProps) {
  const [fundingAccount, setFundingAccount] = useState<string | null>(null);
  const [fundAmount, setFundAmount] = useState('');
  const [copiedAddress, setCopiedAddress] = useState<string | null>(null);
  const fundMutation = useFundAccount(nodeId);
  const queryClient = useQueryClient();

  const handleCopy = async (address: string) => {
    const success = await copyToClipboard(address);
    if (success) {
      setCopiedAddress(address);
      setTimeout(() => setCopiedAddress(null), 2000);
    }
  };

  const handleFund = async (address: string) => {
    const amount = parseFloat(fundAmount);
    if (isNaN(amount) || amount <= 0) {
      alert('Please enter a valid amount');
      return;
    }

    const result = await fundMutation.mutateAsync({
      address,
      amount,
    });

    if (result.success) {
      setFundingAccount(null);
      setFundAmount('');
      // Force refresh accounts after a short delay to allow blockchain to update
      setTimeout(() => {
        queryClient.invalidateQueries({ queryKey: ['accounts', nodeId] });
      }, 1000);
    } else if (result.error) {
      alert(`Error: ${result.error}`);
    }
  };

  const unit = chain === 'solana' ? 'SOL' : 'BTC';
  const decimals = chain === 'solana' ? 2 : 8;

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead className="bg-gray-50 dark:bg-gray-800">
          <tr>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              #
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Address
            </th>
            <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Balance ({unit})
            </th>
            <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
          {accounts.map((account) => (
            <tr
              key={account.index}
              className="hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
            >
              <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                {account.index}
              </td>
              <td className="px-4 py-3 whitespace-nowrap">
                <code className="text-sm text-gray-900 dark:text-gray-100 font-mono">
                  {account.address.slice(0, 8)}...{account.address.slice(-8)}
                </code>
                <button
                  onClick={() => handleCopy(account.address)}
                  className="ml-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                  title="Copy address"
                >
                  {copiedAddress === account.address ? (
                    <svg
                      className="w-4 h-4 inline text-green-500"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  ) : (
                    <svg
                      className="w-4 h-4 inline"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                      />
                    </svg>
                  )}
                </button>
              </td>
              <td className="px-4 py-3 whitespace-nowrap text-sm text-right font-mono">
                {account.balance.toFixed(decimals)}
              </td>
              <td className="px-4 py-3 whitespace-nowrap text-right">
                {fundingAccount === account.address ? (
                  <div className="flex items-center justify-end gap-2">
                    <input
                      type="number"
                      value={fundAmount}
                      onChange={(e) => setFundAmount(e.target.value)}
                      placeholder="Amount"
                      step="0.1"
                      min="0"
                      className="w-24 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                      autoFocus
                    />
                    <button
                      onClick={() => handleFund(account.address)}
                      disabled={fundMutation.isPending}
                      className="px-2 py-1 text-xs bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
                    >
                      {fundMutation.isPending ? '...' : 'Fund'}
                    </button>
                    <button
                      onClick={() => {
                        setFundingAccount(null);
                        setFundAmount('');
                      }}
                      className="px-2 py-1 text-xs bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
                    >
                      Cancel
                    </button>
                  </div>
                ) : (
                  <button
                    onClick={() => setFundingAccount(account.address)}
                    className="px-3 py-1 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-200 rounded hover:bg-blue-200 dark:hover:bg-blue-800 transition-colors"
                  >
                    + Fund
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
