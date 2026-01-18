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
