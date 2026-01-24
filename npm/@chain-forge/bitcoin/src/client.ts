import { spawn, ChildProcess } from 'child_process';
import { promises as fs } from 'fs';
import { homedir } from 'os';
import { join } from 'path';
import { existsSync } from 'fs';
import { EventEmitter } from 'events';
import { BitcoinAccount, BitcoinClientConfig, SendResult, MineResult } from './types';

/**
 * Find the cf-bitcoin binary
 * Looks in multiple locations: workspace root, project root, then PATH
 */
function findBinary(): string {
  // Possible locations to check (both debug and release builds)
  const possiblePaths = [
    // From npm package: npm/@chain-forge/bitcoin/dist/client.js
    join(__dirname, '..', '..', '..', '..', 'target', 'debug', 'cf-bitcoin'),
    join(__dirname, '..', '..', '..', '..', 'target', 'release', 'cf-bitcoin'),
    // From example in node_modules: examples/*/node_modules/@chain-forge/bitcoin/dist/client.js
    join(__dirname, '..', '..', '..', '..', '..', '..', 'target', 'debug', 'cf-bitcoin'),
    join(__dirname, '..', '..', '..', '..', '..', '..', 'target', 'release', 'cf-bitcoin'),
    // From current working directory
    join(process.cwd(), 'target', 'debug', 'cf-bitcoin'),
    join(process.cwd(), 'target', 'release', 'cf-bitcoin'),
    // Check parent directories up to 5 levels
    join(process.cwd(), '..', 'target', 'debug', 'cf-bitcoin'),
    join(process.cwd(), '..', 'target', 'release', 'cf-bitcoin'),
    join(process.cwd(), '..', '..', 'target', 'debug', 'cf-bitcoin'),
    join(process.cwd(), '..', '..', 'target', 'release', 'cf-bitcoin'),
  ];

  for (const path of possiblePaths) {
    if (existsSync(path)) {
      return path;
    }
  }

  // Fall back to PATH (for installed binary)
  return 'cf-bitcoin';
}

/**
 * Events emitted by BitcoinClient
 */
export interface BitcoinClientEvents {
  /** Emitted when the node is fully initialized and all accounts are funded */
  ready: [];
  /** Emitted when an error occurs during startup */
  error: [error: Error];
  /** Emitted when the node process exits */
  exit: [code: number | null];
}

/**
 * Client for managing a local Bitcoin regtest node and accounts
 *
 * @example
 * ```typescript
 * const client = new BitcoinClient({ accounts: 5, initialBalance: 10 });
 *
 * // Listen for ready event
 * client.on('ready', () => {
 *   console.log('Bitcoin node is ready!');
 * });
 *
 * // Or use await
 * await client.start();
 * ```
 */
export class BitcoinClient extends EventEmitter {
  private config: Required<BitcoinClientConfig>;
  private nodeProcess: ChildProcess | null = null;

  constructor(config: BitcoinClientConfig = {}) {
    super();
    this.config = {
      instance: config.instance ?? 'default',
      accounts: config.accounts ?? 10,
      initialBalance: config.initialBalance ?? 10,
      rpcPort: config.rpcPort ?? 18443,
      p2pPort: config.p2pPort ?? 18444,
      mnemonic: config.mnemonic ?? '',
      rpcUrl: config.rpcUrl ?? `http://localhost:${config.rpcPort ?? 18443}`,
      rpcUser: config.rpcUser ?? 'chainforge',
      rpcPassword: config.rpcPassword ?? 'chainforge',
    };
  }

  /**
   * Get the instance ID
   */
  getInstance(): string {
    return this.config.instance;
  }

  /**
   * Type-safe event emitter methods
   */
  on<K extends keyof BitcoinClientEvents>(
    event: K,
    listener: (...args: BitcoinClientEvents[K]) => void
  ): this {
    return super.on(event, listener as (...args: unknown[]) => void);
  }

  once<K extends keyof BitcoinClientEvents>(
    event: K,
    listener: (...args: BitcoinClientEvents[K]) => void
  ): this {
    return super.once(event, listener as (...args: unknown[]) => void);
  }

  emit<K extends keyof BitcoinClientEvents>(
    event: K,
    ...args: BitcoinClientEvents[K]
  ): boolean {
    return super.emit(event, ...args);
  }

  /**
   * Start the local Bitcoin regtest node
   *
   * @returns Promise that resolves when the node is fully initialized and all accounts are funded
   */
  async start(): Promise<void> {
    if (this.nodeProcess) {
      throw new Error('Bitcoin node is already running');
    }

    const args = [
      'start',
      '--instance', this.config.instance,
      '--accounts', this.config.accounts.toString(),
      '--balance', this.config.initialBalance.toString(),
      '--rpc-port', this.config.rpcPort.toString(),
      '--p2p-port', this.config.p2pPort.toString(),
      '--rpc-user', this.config.rpcUser,
      '--rpc-password', this.config.rpcPassword,
    ];

    if (this.config.mnemonic) {
      args.push('--mnemonic', this.config.mnemonic);
    }

    return new Promise((resolve, reject) => {
      const binaryPath = findBinary();
      this.nodeProcess = spawn(binaryPath, args, {
        stdio: 'pipe',
      });

      let output = '';
      let resolved = false;

      this.nodeProcess.stdout?.on('data', (data: Buffer) => {
        output += data.toString();
        process.stdout.write(data);

        // Wait for the success message indicating full initialization
        if (!resolved && output.includes('Bitcoin regtest node is running')) {
          resolved = true;
          this.emit('ready');
          resolve();
        }
      });

      this.nodeProcess.stderr?.on('data', (data: Buffer) => {
        process.stderr.write(data);
      });

      this.nodeProcess.on('error', (error: Error) => {
        this.emit('error', error);
        if (!resolved) {
          reject(new Error(`Failed to start Bitcoin node: ${error.message}`));
        }
      });

      this.nodeProcess.on('exit', (code: number | null) => {
        this.emit('exit', code);
        if (code !== 0 && code !== null && !resolved) {
          reject(new Error(`Bitcoin node exited with code ${code}`));
        }
        this.nodeProcess = null;
      });

      // Timeout after 180 seconds (Bitcoin startup takes longer with more accounts)
      setTimeout(() => {
        if (this.nodeProcess && !resolved) {
          this.stop();
          const error = new Error('Bitcoin node startup timeout');
          this.emit('error', error);
          reject(error);
        }
      }, 180000);
    });
  }

  /**
   * Stop the running node
   */
  stop(): void {
    if (this.nodeProcess) {
      this.nodeProcess.kill('SIGINT');
      this.nodeProcess = null;
    }
  }

  /**
   * Check if the node is running
   */
  isRunning(): boolean {
    return this.nodeProcess !== null;
  }

  /**
   * Get all generated accounts
   */
  async getAccounts(): Promise<BitcoinAccount[]> {
    // Use instance-specific path
    const accountsPath = join(
      homedir(),
      '.chain-forge',
      'bitcoin',
      'instances',
      this.config.instance,
      'accounts.json'
    );

    try {
      const data = await fs.readFile(accountsPath, 'utf-8');
      const accounts = JSON.parse(data) as BitcoinAccount[];
      return accounts;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        return [];
      }
      throw error;
    }
  }

  /**
   * Set an account's balance to a specific amount
   *
   * This sends BTC to the address to achieve the target balance.
   * Note: Requires the wallet to have sufficient funds.
   *
   * @param address - Bitcoin address
   * @param targetAmount - Target balance in BTC
   * @returns Result message
   */
  async setBalance(address: string, targetAmount: number): Promise<string> {
    const currentBalance = await this.getBalance(address);

    if (currentBalance >= targetAmount) {
      return `Balance already at ${currentBalance} BTC (target: ${targetAmount} BTC)`;
    }

    const diff = targetAmount - currentBalance;
    await this.sendToAddress(address, diff);

    // Mine a block to confirm
    await this.mine(1);

    return `Added ${diff} BTC (${currentBalance} -> ${targetAmount} BTC)`;
  }

  /**
   * Send BTC to an address (from wallet funds)
   *
   * This sends from the wallet's available funds, not from a specific account.
   * For sending from a specific account, use `transfer()` instead.
   *
   * @param address - Destination Bitcoin address
   * @param amount - Amount of BTC to send
   * @returns Transaction result
   */
  async sendToAddress(address: string, amount: number): Promise<SendResult> {
    return new Promise((resolve, reject) => {
      const binaryPath = findBinary();
      const args = ['fund', address, amount.toString(), '--instance', this.config.instance];
      const process = spawn(binaryPath, args, {
        stdio: 'pipe',
      });

      let stdout = '';
      let stderr = '';

      process.stdout?.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr?.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('error', (error: Error) => {
        reject(new Error(`Failed to send BTC: ${error.message}`));
      });

      process.on('close', (code: number | null) => {
        if (code === 0) {
          // Try to extract txid from output
          const txidMatch = stdout.match(/TxID: ([a-fA-F0-9]+)/);
          resolve({
            txid: txidMatch ? txidMatch[1] : 'transaction-sent',
          });
        } else {
          reject(new Error(`Send failed (exit code ${code}): ${stderr || stdout}`));
        }
      });
    });
  }

  /**
   * Transfer BTC from one account to another
   *
   * This creates a transaction that specifically spends from the source account's
   * UTXOs. Both accounts must be managed by this node (imported into the wallet).
   *
   * @param fromAddress - Source Bitcoin address
   * @param toAddress - Destination Bitcoin address
   * @param amount - Amount of BTC to send
   * @returns Transaction result
   */
  async transfer(fromAddress: string, toAddress: string, amount: number): Promise<SendResult> {
    return new Promise((resolve, reject) => {
      const binaryPath = findBinary();
      const args = [
        'transfer',
        fromAddress,
        toAddress,
        amount.toString(),
        '--instance',
        this.config.instance,
      ];
      const process = spawn(binaryPath, args, {
        stdio: 'pipe',
      });

      let stdout = '';
      let stderr = '';

      process.stdout?.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr?.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('error', (error: Error) => {
        reject(new Error(`Failed to transfer BTC: ${error.message}`));
      });

      process.on('close', (code: number | null) => {
        if (code === 0) {
          // Try to extract txid from output
          const txidMatch = stdout.match(/TxID: ([a-fA-F0-9]+)/);
          resolve({
            txid: txidMatch ? txidMatch[1] : 'transaction-sent',
          });
        } else {
          reject(new Error(`Transfer failed (exit code ${code}): ${stderr || stdout}`));
        }
      });
    });
  }

  /**
   * Get the balance of an account from the cached accounts file
   * @param address - Bitcoin address
   * @returns Balance in BTC
   */
  async getBalance(address: string): Promise<number> {
    const accounts = await this.getAccounts();
    const account = accounts.find(a => a.address === address);
    return account?.balance ?? 0;
  }

  /**
   * Refresh account balances from the blockchain
   *
   * This queries the Bitcoin node for current balances and returns
   * updated account information. Use this after sending transactions
   * or mining blocks to get accurate balance information.
   *
   * @returns Updated accounts with current blockchain balances
   */
  async refreshBalances(): Promise<BitcoinAccount[]> {
    return new Promise((resolve, reject) => {
      const binaryPath = findBinary();
      const args = ['accounts', '--format', 'json', '--instance', this.config.instance];
      const process = spawn(binaryPath, args, {
        stdio: 'pipe',
      });

      let stdout = '';
      let stderr = '';

      process.stdout?.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr?.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('error', (error: Error) => {
        reject(new Error(`Failed to refresh balances: ${error.message}`));
      });

      process.on('close', (code: number | null) => {
        if (code === 0) {
          try {
            const accounts = JSON.parse(stdout) as BitcoinAccount[];
            resolve(accounts);
          } catch (e) {
            reject(new Error(`Failed to parse accounts: ${(e as Error).message}`));
          }
        } else {
          reject(new Error(`Failed to refresh balances (exit code ${code}): ${stderr || stdout}`));
        }
      });
    });
  }

  /**
   * Mine blocks
   *
   * @param blocks - Number of blocks to mine
   * @param address - Optional address to receive coinbase rewards (uses account 0 if not specified)
   * @returns Mining result
   */
  async mine(blocks: number = 1, address?: string): Promise<MineResult> {
    return new Promise((resolve, reject) => {
      const binaryPath = findBinary();
      const args = ['mine', '--blocks', blocks.toString(), '--instance', this.config.instance];

      if (address) {
        args.push('--address', address);
      }

      const process = spawn(binaryPath, args, {
        stdio: 'pipe',
      });

      let stdout = '';
      let stderr = '';

      process.stdout?.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr?.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('error', (error: Error) => {
        reject(new Error(`Failed to mine blocks: ${error.message}`));
      });

      process.on('close', (code: number | null) => {
        if (code === 0) {
          // Try to extract block hashes from output
          const blockMatches = stdout.matchAll(/Block \d+: ([a-fA-F0-9]+)/g);
          const blockHashes = [...blockMatches].map(m => m[1]);

          // Try to extract height
          const heightMatch = stdout.match(/Current height: (\d+)/);
          const height = heightMatch ? parseInt(heightMatch[1], 10) : undefined;

          resolve({
            blockHashes,
            height,
          });
        } else {
          reject(new Error(`Mining failed (exit code ${code}): ${stderr || stdout}`));
        }
      });
    });
  }

  /**
   * Get the RPC URL
   */
  getRpcUrl(): string {
    return this.config.rpcUrl;
  }

  /**
   * Get the RPC credentials
   */
  getRpcCredentials(): { user: string; password: string } {
    return {
      user: this.config.rpcUser,
      password: this.config.rpcPassword,
    };
  }
}
