import { SolanaClient, SolanaAccount } from '@chain-forge/solana';
import { BitcoinClient, BitcoinAccount } from '@chain-forge/bitcoin';

// Chain type discriminator
export type ChainType = 'solana' | 'bitcoin';

export interface DeployedProgram {
  programId: string;
  name: string;
  path: string;
  size: number;
  idlPath?: string;
  deployerAddress: string;
}

export interface ProgramAccount {
  address: string;
  programId: string; // Owner program
  size: number;
  createdBy: string; // Payer who created it
}

// Solana-specific node state
export interface SolanaNodeState {
  chainType: 'solana';
  id: string;
  port: number;
  mnemonic: string | null;
  client: SolanaClient;
  accounts: SolanaAccount[];
  deployedPrograms: DeployedProgram[];
  programAccounts: ProgramAccount[];
}

// Bitcoin-specific node state
export interface BitcoinNodeState {
  chainType: 'bitcoin';
  id: string;
  port: number;
  mnemonic: string | null;
  client: BitcoinClient;
  accounts: BitcoinAccount[];
}

// Union type for any node
export type NodeState = SolanaNodeState | BitcoinNodeState;

export interface AppState {
  nodes: NodeState[];
  activeNodeIndex: number | null;
}

export interface ProgramInfo {
  name: string;
  path: string;
  cargoTomlPath: string;
}

// IDL (Interface Definition Language) types for program interaction
export interface ProgramIDL {
  name: string;
  version: string;
  instructions: IDLInstruction[];
}

export interface IDLInstruction {
  name: string;
  discriminator: number;
  description?: string;
  accounts: IDLAccountMeta[];
  args?: IDLInstructionArg[];
}

export interface IDLAccountMeta {
  name: string;
  isSigner: boolean;
  isWritable: boolean;
  description?: string;
}

export interface IDLInstructionArg {
  name: string;
  type: 'u8' | 'u64' | 'string' | 'pubkey';
  description?: string;
}

// Type guards
export function isSolanaNode(node: NodeState): node is SolanaNodeState {
  return node.chainType === 'solana';
}

export function isBitcoinNode(node: NodeState): node is BitcoinNodeState {
  return node.chainType === 'bitcoin';
}
