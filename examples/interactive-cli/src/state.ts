import { AppState, NodeState, ProgramAccount, SolanaNodeState, BitcoinNodeState, isSolanaNode, isBitcoinNode } from './types';

let state: AppState = {
  nodes: [],
  activeNodeIndex: null,
};

export function getState(): AppState {
  return state;
}

export function getActiveNode(): NodeState | null {
  if (state.activeNodeIndex === null || state.activeNodeIndex >= state.nodes.length) {
    return null;
  }
  return state.nodes[state.activeNodeIndex];
}

export function getActiveSolanaNode(): SolanaNodeState | null {
  const node = getActiveNode();
  if (node && isSolanaNode(node)) {
    return node;
  }
  return null;
}

export function getActiveBitcoinNode(): BitcoinNodeState | null {
  const node = getActiveNode();
  if (node && isBitcoinNode(node)) {
    return node;
  }
  return null;
}

export function addNode(node: NodeState): void {
  state.nodes.push(node);
  state.activeNodeIndex = state.nodes.length - 1;
}

export function removeActiveNode(): void {
  if (state.activeNodeIndex !== null) {
    state.nodes.splice(state.activeNodeIndex, 1);
    state.activeNodeIndex = state.nodes.length > 0 ? 0 : null;
  }
}

export function updateActiveNodeAccounts(accounts: NodeState['accounts']): void {
  const node = getActiveNode();
  if (node) {
    (node as any).accounts = accounts;
  }
}

export function addDeployedProgram(program: SolanaNodeState['deployedPrograms'][0]): void {
  const node = getActiveSolanaNode();
  if (node) {
    node.deployedPrograms.push(program);
  }
}

export function addProgramAccount(account: ProgramAccount): void {
  const node = getActiveSolanaNode();
  if (node) {
    node.programAccounts.push(account);
  }
}

export function generateNodeId(): string {
  return `node-${Date.now().toString(36)}`;
}

export function stopAllNodes(): void {
  for (const node of state.nodes) {
    try {
      node.client.stop();
    } catch {
      // Ignore errors when stopping
    }
  }
  state.nodes = [];
  state.activeNodeIndex = null;
}

// Re-export type guards
export { isSolanaNode, isBitcoinNode };
