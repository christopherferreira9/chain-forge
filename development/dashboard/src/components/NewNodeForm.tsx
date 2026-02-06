// Form to start a new node

import { useState } from 'react';
import { useStartNode } from '../api/hooks';

interface NewNodeFormProps {
  onClose: () => void;
}

export function NewNodeForm({ onClose }: NewNodeFormProps) {
  const [chain, setChain] = useState<'solana' | 'bitcoin'>('solana');
  const [instance, setInstance] = useState('default');
  const [name, setName] = useState('');
  const [port, setPort] = useState('8899');
  const [accounts, setAccounts] = useState('10');
  const [balance, setBalance] = useState('100');
  const [generatedCommand, setGeneratedCommand] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const startMutation = useStartNode();

  // Sanitize name/instance input: replace spaces with hyphens, remove invalid chars
  const sanitizeName = (value: string): string => {
    return value
      .replace(/\s+/g, '-')           // Replace spaces with hyphens
      .replace(/[^a-zA-Z0-9-]/g, '')  // Remove any char that's not alphanumeric or hyphen
      .replace(/-+/g, '-')            // Collapse multiple hyphens
      .toLowerCase();
  };

  const handleInstanceChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setInstance(sanitizeName(e.target.value));
  };

  const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setName(sanitizeName(e.target.value));
  };

  // Update balance default when chain changes
  const handleChainChange = (newChain: 'solana' | 'bitcoin') => {
    setChain(newChain);
    setBalance(newChain === 'solana' ? '100' : '10');
    setPort(newChain === 'solana' ? '8899' : '18443');
  };

  const handleCopyCommand = async () => {
    if (!generatedCommand) return;

    try {
      await navigator.clipboard.writeText(generatedCommand);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      // Fallback for browsers that don't support clipboard API
      const textArea = document.createElement('textarea');
      textArea.value = generatedCommand;
      textArea.style.position = 'fixed';
      textArea.style.left = '-999999px';
      textArea.style.top = '-999999px';
      document.body.appendChild(textArea);
      textArea.focus();
      textArea.select();
      try {
        document.execCommand('copy');
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (fallbackErr) {
        console.error('Failed to copy:', fallbackErr);
      }
      document.body.removeChild(textArea);
    }
  };

  const handleCreateAnother = () => {
    setGeneratedCommand(null);
    setChain('solana');
    setInstance('default');
    setName('');
    setPort('8899');
    setAccounts('10');
    setBalance('100');
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const result = await startMutation.mutateAsync({
      chain,
      instance: instance || 'default',
      name: name || undefined,
      port: parseInt(port) || (chain === 'solana' ? 8899 : 18443),
      accounts: parseInt(accounts) || 10,
      balance: parseFloat(balance) || (chain === 'solana' ? 100 : 10),
    });

    if (result.success && result.data) {
      setGeneratedCommand(result.data.command);
    } else if (result.error) {
      setError(result.error);
    }
  };

  // Show command result view
  if (generatedCommand) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Start Your Node
            </h2>
          </div>

          <div className="p-6">
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              Run this command in a terminal to start your node:
            </p>

            <div className="relative">
              <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg text-sm overflow-x-auto font-mono whitespace-pre-wrap break-all min-h-[80px]">
                {generatedCommand}
              </pre>
              <button
                onClick={handleCopyCommand}
                className="absolute top-2 right-2 px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 text-white rounded transition-colors"
              >
                {copied ? 'Copied!' : 'Copy'}
              </button>
            </div>

            <p className="text-xs text-gray-500 dark:text-gray-400 mt-4">
              Keep the terminal open to keep the node running. Press Ctrl+C to stop.
            </p>

            <div className="flex gap-3 mt-6">
              <button
                onClick={handleCreateAnother}
                className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-md transition-colors"
              >
                Create Another
              </button>
              <button
                onClick={onClose}
                className={`flex-1 px-4 py-2 text-sm font-medium text-white rounded-md transition-colors ${
                  chain === 'solana'
                    ? 'bg-purple-600 hover:bg-purple-700'
                    : 'bg-orange-600 hover:bg-orange-700'
                }`}
              >
                Done
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Start New Node
          </h2>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Error Display */}
          {error && (
            <div className="p-3 bg-red-100 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-md">
              <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
            </div>
          )}

          {/* Chain Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Chain
            </label>
            <div className="flex gap-4">
              <label className="flex items-center">
                <input
                  type="radio"
                  name="chain"
                  value="solana"
                  checked={chain === 'solana'}
                  onChange={() => handleChainChange('solana')}
                  className="mr-2"
                />
                <span className="text-purple-600 dark:text-purple-400 font-medium">
                  Solana
                </span>
              </label>
              <label className="flex items-center">
                <input
                  type="radio"
                  name="chain"
                  value="bitcoin"
                  checked={chain === 'bitcoin'}
                  onChange={() => handleChainChange('bitcoin')}
                  className="mr-2"
                />
                <span className="text-orange-600 dark:text-orange-400 font-medium">
                  Bitcoin
                </span>
              </label>
            </div>
          </div>

          {/* Instance ID */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Instance ID (optional)
            </label>
            <input
              type="text"
              value={instance}
              onChange={handleInstanceChange}
              placeholder="default"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Only lowercase letters, numbers, and hyphens
            </p>
          </div>

          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Display Name (optional)
            </label>
            <input
              type="text"
              value={name}
              onChange={handleNameChange}
              placeholder="my-dev-node"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Only lowercase letters, numbers, and hyphens
            </p>
          </div>

          {/* Port */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              RPC Port
            </label>
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(e.target.value)}
              placeholder={chain === 'solana' ? '8899' : '18443'}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Accounts & Balance */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Accounts
              </label>
              <input
                type="number"
                value={accounts}
                onChange={(e) => setAccounts(e.target.value)}
                placeholder="10"
                min="1"
                max="100"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Initial Balance ({chain === 'solana' ? 'SOL' : 'BTC'})
              </label>
              <input
                type="number"
                value={balance}
                onChange={(e) => setBalance(e.target.value)}
                placeholder={chain === 'solana' ? '100' : '10'}
                min="0"
                step="0.1"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-md transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={startMutation.isPending}
              className={`flex-1 px-4 py-2 text-sm font-medium text-white rounded-md transition-colors disabled:opacity-50 ${
                chain === 'solana'
                  ? 'bg-purple-600 hover:bg-purple-700'
                  : 'bg-orange-600 hover:bg-orange-700'
              }`}
            >
              {startMutation.isPending ? 'Creating...' : 'Get Start Command'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
