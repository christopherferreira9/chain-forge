/**
 * Configuration options for BitcoinClient
 */
export interface BitcoinClientConfig {
  /**
   * Instance ID for isolation (allows multiple nodes with separate state)
   * @default "default"
   */
  instance?: string;

  /**
   * Number of accounts to generate
   * @default 10
   */
  accounts?: number;

  /**
   * Initial balance for each account in BTC
   * @default 10
   */
  initialBalance?: number;

  /**
   * RPC port for the node
   * @default 18443
   */
  rpcPort?: number;

  /**
   * P2P network port
   * @default 18444
   */
  p2pPort?: number;

  /**
   * Optional mnemonic phrase to use for account generation
   */
  mnemonic?: string;

  /**
   * RPC URL to connect to
   * @default "http://localhost:18443"
   */
  rpcUrl?: string;

  /**
   * RPC username
   * @default "chainforge"
   */
  rpcUser?: string;

  /**
   * RPC password
   * @default "chainforge"
   */
  rpcPassword?: string;
}

/**
 * Bitcoin account information
 */
export interface BitcoinAccount {
  /**
   * Bitcoin address (P2WPKH bech32 format)
   */
  address: string;

  /**
   * Hex-encoded compressed public key
   */
  publicKey: string;

  /**
   * Private key bytes
   */
  privateKey: number[];

  /**
   * WIF-encoded private key for wallet import
   */
  wif: string;

  /**
   * Optional mnemonic phrase used to generate this account
   */
  mnemonic?: string;

  /**
   * Optional derivation path
   */
  derivationPath?: string;

  /**
   * Current balance in BTC
   */
  balance: number;
}

/**
 * Result of sending BTC
 */
export interface SendResult {
  /**
   * Transaction ID
   */
  txid: string;
}

/**
 * Result of mining blocks
 */
export interface MineResult {
  /**
   * Block hashes of the mined blocks
   */
  blockHashes: string[];

  /**
   * Current block height after mining
   */
  height?: number;
}
