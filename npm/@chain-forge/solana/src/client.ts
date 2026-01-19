import { spawn, ChildProcess } from 'child_process';
import { promises as fs } from 'fs';
import { homedir } from 'os';
import { join, dirname } from 'path';
import { existsSync } from 'fs';
import { SolanaAccount, SolanaClientConfig, DeployProgramOptions, DeployProgramResult } from './types';

// Import Solana web3.js types
import type { Connection, PublicKey, Keypair } from '@solana/web3.js';

/**
 * Find the cf-solana binary
 * Looks in multiple locations: workspace root, project root, then PATH
 */
function findBinary(): string {
  // Possible locations to check
  const possiblePaths = [
    // From npm package: npm/@chain-forge/solana/dist/client.js
    join(__dirname, '..', '..', '..', '..', 'target', 'release', 'cf-solana'),
    // From example in node_modules: examples/*/node_modules/@chain-forge/solana/dist/client.js
    join(__dirname, '..', '..', '..', '..', '..', '..', 'target', 'release', 'cf-solana'),
    // From current working directory
    join(process.cwd(), 'target', 'release', 'cf-solana'),
    // Check parent directories up to 5 levels
    join(process.cwd(), '..', 'target', 'release', 'cf-solana'),
    join(process.cwd(), '..', '..', 'target', 'release', 'cf-solana'),
  ];

  for (const path of possiblePaths) {
    if (existsSync(path)) {
      return path;
    }
  }

  // Fall back to PATH (for installed binary)
  return 'cf-solana';
}

/**
 * Client for managing a local Solana test validator and accounts
 */
export class SolanaClient {
  private config: Required<SolanaClientConfig>;
  private validatorProcess: ChildProcess | null = null;
  private connection: Connection | null = null;

  constructor(config: SolanaClientConfig = {}) {
    this.config = {
      accounts: config.accounts ?? 10,
      initialBalance: config.initialBalance ?? 100,
      port: config.port ?? 8899,
      mnemonic: config.mnemonic ?? '',
      rpcUrl: config.rpcUrl ?? `http://localhost:${config.port ?? 8899}`,
    };
  }

  /**
   * Start the local Solana test validator
   */
  async start(): Promise<void> {
    if (this.validatorProcess) {
      throw new Error('Validator is already running');
    }

    const args = [
      'start',
      '--accounts', this.config.accounts.toString(),
      '--balance', this.config.initialBalance.toString(),
      '--port', this.config.port.toString(),
    ];

    if (this.config.mnemonic) {
      args.push('--mnemonic', this.config.mnemonic);
    }

    return new Promise((resolve, reject) => {
      const binaryPath = findBinary();
      this.validatorProcess = spawn(binaryPath, args, {
        stdio: 'pipe',
      });

      let output = '';

      this.validatorProcess.stdout?.on('data', (data: Buffer) => {
        output += data.toString();
        process.stdout.write(data);

        // Wait for the success message
        if (output.includes('Solana test validator is running')) {
          resolve();
        }
      });

      this.validatorProcess.stderr?.on('data', (data: Buffer) => {
        process.stderr.write(data);
      });

      this.validatorProcess.on('error', (error: Error) => {
        reject(new Error(`Failed to start validator: ${error.message}`));
      });

      this.validatorProcess.on('exit', (code: number | null) => {
        if (code !== 0 && code !== null) {
          reject(new Error(`Validator exited with code ${code}`));
        }
        this.validatorProcess = null;
      });

      // Timeout after 60 seconds
      setTimeout(() => {
        if (this.validatorProcess && !output.includes('Solana test validator is running')) {
          this.stop();
          reject(new Error('Validator startup timeout'));
        }
      }, 60000);
    });
  }

  /**
   * Stop the running validator
   */
  stop(): void {
    if (this.validatorProcess) {
      this.validatorProcess.kill('SIGINT');
      this.validatorProcess = null;
    }
    this.connection = null;
  }

  /**
   * Check if the validator is running
   */
  isRunning(): boolean {
    return this.validatorProcess !== null;
  }

  /**
   * Get all generated accounts
   */
  async getAccounts(): Promise<SolanaAccount[]> {
    const accountsPath = join(homedir(), '.chain-forge', 'solana', 'accounts.json');

    try {
      const data = await fs.readFile(accountsPath, 'utf-8');
      const accounts = JSON.parse(data) as SolanaAccount[];
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
   * This is the preferred method for managing account balances.
   * It adjusts the account to have exactly the target amount.
   *
   * Note: On Solana, we can only add funds (not reduce them).
   * If the account already has >= target balance, no action is taken.
   *
   * @param address - Public key string or PublicKey object
   * @param targetAmount - Target balance in SOL
   * @returns Result message
   *
   * @example
   * ```typescript
   * // Ensure account has exactly 100 SOL
   * await client.setBalance(address, 100);
   * ```
   */
  async setBalance(address: string | PublicKey, targetAmount: number): Promise<string> {
    const connection = this.getConnection();
    const web3 = require('@solana/web3.js');
    const { PublicKey: SolanaPublicKey, LAMPORTS_PER_SOL } = web3;

    const pubkey = typeof address === 'string'
      ? new SolanaPublicKey(address)
      : address;

    // Get current balance
    const currentLamports = await connection.getBalance(pubkey);
    const currentSol = currentLamports / LAMPORTS_PER_SOL;
    const targetLamports = targetAmount * LAMPORTS_PER_SOL;

    // If already at or above target, nothing to do
    if (currentLamports >= targetLamports) {
      return `Balance already at ${currentSol} SOL (target: ${targetAmount} SOL)`;
    }

    // Request airdrop for the difference
    const diffLamports = targetLamports - currentLamports;
    const diffSol = diffLamports / LAMPORTS_PER_SOL;

    const signature = await connection.requestAirdrop(pubkey, diffLamports);
    await connection.confirmTransaction(signature);

    return `Added ${diffSol} SOL (${currentSol} → ${targetAmount} SOL)`;
  }

  /**
   * Fund an account with SOL (adds to existing balance)
   *
   * @deprecated Use setBalance instead for more predictable behavior
   * @param address - Public key string or PublicKey object
   * @param amount - Amount of SOL to add
   * @returns Transaction signature
   */
  async fundAccount(address: string | PublicKey, amount: number): Promise<string> {
    const connection = this.getConnection();
    const web3 = require('@solana/web3.js');
    const { PublicKey: SolanaPublicKey, LAMPORTS_PER_SOL } = web3;

    const pubkey = typeof address === 'string'
      ? new SolanaPublicKey(address)
      : address;

    const lamports = amount * LAMPORTS_PER_SOL;
    const signature = await connection.requestAirdrop(pubkey, lamports);
    await connection.confirmTransaction(signature);

    return signature;
  }

  /**
   * Get the balance of an account
   * @param address - Public key string or PublicKey object
   * @returns Balance in SOL
   */
  async getBalance(address: string | PublicKey): Promise<number> {
    const connection = this.getConnection();
    const web3 = require('@solana/web3.js');
    const { PublicKey: SolanaPublicKey, LAMPORTS_PER_SOL } = web3;

    const pubkey = typeof address === 'string'
      ? new SolanaPublicKey(address)
      : address;

    const lamports = await connection.getBalance(pubkey);
    return lamports / LAMPORTS_PER_SOL;
  }

  /**
   * Get a Connection instance for direct Solana web3.js usage
   */
  getConnection(): Connection {
    if (!this.connection) {
      const web3 = require('@solana/web3.js');
      this.connection = new web3.Connection(this.config.rpcUrl, 'confirmed');
    }
    return this.connection!;
  }

  /**
   * Get the RPC URL
   */
  getRpcUrl(): string {
    return this.config.rpcUrl;
  }

  /**
   * Get a Keypair from a generated account
   * @param accountIndex - Index of the account (default: 0)
   * @returns Keypair instance
   */
  async getKeypair(accountIndex: number = 0): Promise<Keypair> {
    const accounts = await this.getAccounts();

    if (accountIndex < 0 || accountIndex >= accounts.length) {
      throw new Error(`Invalid account index ${accountIndex}. Available accounts: 0-${accounts.length - 1}`);
    }

    const account = accounts[accountIndex];
    const web3 = require('@solana/web3.js');
    const { Keypair: SolanaKeypair } = web3;

    return SolanaKeypair.fromSecretKey(new Uint8Array(account.secretKey));
  }

  /**
   * Deploy a Solana program from a compiled .so file
   *
   * Uses the `solana program deploy` CLI command which handles the modern
   * Upgradeable BPF Loader automatically.
   *
   * @param programPath - Path to the compiled program (.so file)
   * @param options - Deployment options
   * @returns Deployment result with program ID and transaction signature
   *
   * @example
   * ```typescript
   * const client = new SolanaClient();
   * await client.start();
   *
   * // Deploy using the first account as payer
   * const result = await client.deployProgram('./target/deploy/my_program.so');
   * console.log('Program ID:', result.programId);
   *
   * // Deploy using a specific account
   * const result2 = await client.deployProgram('./program.so', { payerIndex: 1 });
   * ```
   */
  async deployProgram(
    programPath: string,
    options: DeployProgramOptions = {}
  ): Promise<DeployProgramResult> {
    const { payerIndex = 0, programKeypair } = options;

    // Read the program binary to get size
    const programData = await fs.readFile(programPath);
    const programSize = programData.length;

    // Get payer account
    const accounts = await this.getAccounts();
    if (payerIndex < 0 || payerIndex >= accounts.length) {
      throw new Error(`Invalid payer index ${payerIndex}. Available accounts: 0-${accounts.length - 1}`);
    }
    const payerAccount = accounts[payerIndex];

    // Create temporary keypair file for the payer
    const tmpDir = join(homedir(), '.chain-forge', 'tmp');
    await fs.mkdir(tmpDir, { recursive: true });
    const keypairPath = join(tmpDir, `keypair-${Date.now()}.json`);

    try {
      // Write payer keypair to temp file (Solana CLI format)
      await fs.writeFile(keypairPath, JSON.stringify(Array.from(payerAccount.secretKey)));

      // Build solana program deploy command
      const args = [
        'program',
        'deploy',
        programPath,
        '--keypair', keypairPath,
        '--url', this.config.rpcUrl,
        '--output', 'json',
      ];

      // If a program keypair is provided, save it and use it
      let programKeypairPath: string | null = null;
      if (programKeypair) {
        programKeypairPath = join(tmpDir, `program-keypair-${Date.now()}.json`);
        await fs.writeFile(programKeypairPath, JSON.stringify(Array.from(programKeypair)));
        args.push('--program-id', programKeypairPath);
      }

      // Execute solana program deploy
      const result = await this.executeSolanaCommand(args);

      // Clean up program keypair if created
      if (programKeypairPath) {
        await fs.unlink(programKeypairPath).catch(() => {});
      }

      // Parse the JSON output
      let deployResult: { programId?: string; signature?: string };
      try {
        deployResult = JSON.parse(result);
      } catch {
        // Try to extract program ID from text output
        const programIdMatch = result.match(/Program Id: ([A-Za-z0-9]+)/);
        if (programIdMatch) {
          deployResult = { programId: programIdMatch[1] };
        } else {
          throw new Error(`Failed to parse deployment result: ${result}`);
        }
      }

      if (!deployResult.programId) {
        throw new Error(`Deployment failed: ${result}`);
      }

      return {
        programId: deployResult.programId,
        signature: deployResult.signature || 'deployment-complete',
        payer: payerAccount.publicKey,
        programSize,
      };
    } finally {
      // Clean up keypair file
      await fs.unlink(keypairPath).catch(() => {});
    }
  }

  /**
   * Execute a solana CLI command and return the output
   */
  private executeSolanaCommand(args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
      const process = spawn('solana', args, { stdio: 'pipe' });

      let stdout = '';
      let stderr = '';

      process.stdout?.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      process.stderr?.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      process.on('error', (error: Error) => {
        reject(new Error(`Failed to execute solana command: ${error.message}`));
      });

      process.on('close', (code: number | null) => {
        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new Error(`Solana command failed (exit code ${code}): ${stderr || stdout}`));
        }
      });
    });
  }
}
