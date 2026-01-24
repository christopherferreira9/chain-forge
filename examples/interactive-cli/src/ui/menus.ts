import { select } from '@inquirer/prompts';
import { getActiveNode, isSolanaNode, isBitcoinNode } from '../state';

export type InitialMenuChoice = 'deploy-node' | 'exit';

// Solana-specific menu choices
export type SolanaMenuChoice =
  | 'deploy-program'
  | 'create-program-account'
  | 'interact-program'
  | 'send-funds'
  | 'refresh'
  | 'stop-node'
  | 'exit';

// Bitcoin-specific menu choices
export type BitcoinMenuChoice =
  | 'send-btc'
  | 'mine-blocks'
  | 'refresh'
  | 'stop-node'
  | 'exit';

// Union type for main menu
export type MainMenuChoice = SolanaMenuChoice | BitcoinMenuChoice;

export async function showInitialMenu(): Promise<InitialMenuChoice> {
  const choice = await select<InitialMenuChoice>({
    message: 'What would you like to do?',
    choices: [
      {
        name: 'Deploy Node',
        value: 'deploy-node',
        description: 'Start a new blockchain node (Solana or Bitcoin)',
      },
      {
        name: 'Exit',
        value: 'exit',
        description: 'Exit the CLI',
      },
    ],
  });

  return choice;
}

export async function showMainMenu(): Promise<MainMenuChoice> {
  const node = getActiveNode();

  if (!node) {
    // This shouldn't happen, but handle it gracefully
    return 'exit';
  }

  if (isSolanaNode(node)) {
    return showSolanaMenu(node);
  } else if (isBitcoinNode(node)) {
    return showBitcoinMenu();
  }

  return 'exit';
}

async function showSolanaMenu(node: { deployedPrograms: unknown[] }): Promise<SolanaMenuChoice> {
  const hasPrograms = node.deployedPrograms.length > 0;

  const choices: { name: string; value: SolanaMenuChoice; description: string }[] = [
    {
      name: 'Deploy Program',
      value: 'deploy-program',
      description: 'Build and deploy a Solana program',
    },
  ];

  // Only show program-related options when programs are deployed
  if (hasPrograms) {
    choices.push({
      name: 'Create Program Account',
      value: 'create-program-account',
      description: 'Create an account owned by a deployed program',
    });
    choices.push({
      name: 'Interact with Program',
      value: 'interact-program',
      description: 'Execute instructions on a deployed program',
    });
  }

  choices.push(
    {
      name: 'Send SOL',
      value: 'send-funds',
      description: 'Transfer SOL between accounts',
    },
    {
      name: 'Refresh Balances',
      value: 'refresh',
      description: 'Update account balances from the validator',
    },
    {
      name: 'Stop Node',
      value: 'stop-node',
      description: 'Stop the validator and return to initial menu',
    },
    {
      name: 'Exit',
      value: 'exit',
      description: 'Exit the CLI',
    },
  );

  const choice = await select<SolanaMenuChoice>({
    message: 'What would you like to do?',
    choices,
  });

  return choice;
}

async function showBitcoinMenu(): Promise<BitcoinMenuChoice> {
  const choices: { name: string; value: BitcoinMenuChoice; description: string }[] = [
    {
      name: 'Send BTC',
      value: 'send-btc',
      description: 'Transfer BTC between accounts',
    },
    {
      name: 'Mine Blocks',
      value: 'mine-blocks',
      description: 'Mine new blocks and receive coinbase rewards',
    },
    {
      name: 'Refresh Balances',
      value: 'refresh',
      description: 'Update account balances from the node',
    },
    {
      name: 'Stop Node',
      value: 'stop-node',
      description: 'Stop the Bitcoin node and return to initial menu',
    },
    {
      name: 'Exit',
      value: 'exit',
      description: 'Exit the CLI',
    },
  ];

  const choice = await select<BitcoinMenuChoice>({
    message: 'What would you like to do?',
    choices,
  });

  return choice;
}
