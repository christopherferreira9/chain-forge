import { SolanaClient, SolanaAccount } from '@chain-forge/solana';

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

export interface NodeState {
  id: string;
  port: number;
  mnemonic: string | null;
  client: SolanaClient;
  accounts: SolanaAccount[];
  deployedPrograms: DeployedProgram[];
  programAccounts: ProgramAccount[];
}

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
