/**
 * Configuration options for SolanaClient
 */
export interface SolanaClientConfig {
  /**
   * Number of accounts to generate
   * @default 10
   */
  accounts?: number;

  /**
   * Initial balance for each account in SOL
   * @default 100
   */
  initialBalance?: number;

  /**
   * RPC port for the validator
   * @default 8899
   */
  port?: number;

  /**
   * Optional mnemonic phrase to use for account generation
   */
  mnemonic?: string;

  /**
   * RPC URL to connect to
   * @default "http://localhost:8899"
   */
  rpcUrl?: string;
}

/**
 * Solana account information
 */
export interface SolanaAccount {
  /**
   * Public key (address) of the account
   */
  publicKey: string;

  /**
   * Secret key bytes
   */
  secretKey: number[];

  /**
   * Optional mnemonic phrase used to generate this account
   */
  mnemonic?: string;

  /**
   * Optional derivation path
   */
  derivationPath?: string;

  /**
   * Current balance in SOL
   */
  balance: number;
}

/**
 * Options for deploying a Solana program
 */
export interface DeployProgramOptions {
  /**
   * Account index to use as payer (default: 0)
   * Uses the account from getAccounts() at this index
   */
  payerIndex?: number;

  /**
   * Optional keypair to use as the program's keypair
   * If not provided, a new keypair will be generated
   */
  programKeypair?: Uint8Array;
}

/**
 * Result of a program deployment
 */
export interface DeployProgramResult {
  /**
   * The deployed program's public key (program ID)
   */
  programId: string;

  /**
   * Transaction signature of the deployment
   */
  signature: string;

  /**
   * The payer account's public key
   */
  payer: string;

  /**
   * Size of the deployed program in bytes
   */
  programSize: number;
}
