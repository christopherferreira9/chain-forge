import { select } from '@inquirer/prompts';
import { getActiveNode } from '../state';

export type InitialMenuChoice = 'deploy-node' | 'exit';
export type MainMenuChoice = 'deploy-program' | 'create-program-account' | 'interact-program' | 'send-funds' | 'refresh' | 'stop-node' | 'exit';

export async function showInitialMenu(): Promise<InitialMenuChoice> {
  const choice = await select<InitialMenuChoice>({
    message: 'What would you like to do?',
    choices: [
      {
        name: 'Deploy Node',
        value: 'deploy-node',
        description: 'Start a new Solana test validator',
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
  const hasPrograms = node && node.deployedPrograms.length > 0;

  const choices: { name: string; value: MainMenuChoice; description: string }[] = [
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
      name: 'Send Funds',
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

  const choice = await select<MainMenuChoice>({
    message: 'What would you like to do?',
    choices,
  });

  return choice;
}
