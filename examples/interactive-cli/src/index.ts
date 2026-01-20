import { renderHeader } from './ui/header';
import { showInitialMenu, showMainMenu, InitialMenuChoice, MainMenuChoice } from './ui/menus';
import { deployNode } from './actions/deploy-node';
import { deployProgram } from './actions/deploy-program';
import { createProgramAccount } from './actions/create-program-account';
import { interactProgram } from './actions/interact-program';
import { sendFunds } from './actions/send-funds';
import { getActiveNode, removeActiveNode, stopAllNodes, getState } from './state';
import { input } from '@inquirer/prompts';
import ora from 'ora';
import { info, success, dim } from './ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

async function refreshBalances(): Promise<void> {
  const node = getActiveNode();
  if (!node) return;

  const spinner = ora('Refreshing balances...').start();
  try {
    for (let i = 0; i < node.accounts.length; i++) {
      node.accounts[i].balance = await node.client.getBalance(node.accounts[i].publicKey);
    }
    spinner.succeed('Balances updated');
  } catch (err) {
    spinner.fail('Failed to refresh balances');
  }
}

async function stopNode(): Promise<void> {
  const node = getActiveNode();
  if (!node) return;

  const spinner = ora('Stopping validator...').start();
  try {
    node.client.stop();
    removeActiveNode();
    spinner.succeed('Validator stopped');
    console.log();
    await waitForEnter();
  } catch (err) {
    spinner.fail('Failed to stop validator');
    await waitForEnter();
  }
}

async function handleInitialMenu(): Promise<boolean> {
  const choice = await showInitialMenu();

  switch (choice) {
    case 'deploy-node':
      return await deployNode();
    case 'exit':
      return false;
  }
}

async function handleMainMenu(): Promise<boolean> {
  const choice = await showMainMenu();

  switch (choice) {
    case 'deploy-program':
      await deployProgram();
      return true;
    case 'create-program-account':
      await createProgramAccount();
      return true;
    case 'interact-program':
      await interactProgram();
      return true;
    case 'send-funds':
      await sendFunds();
      return true;
    case 'refresh':
      await refreshBalances();
      return true;
    case 'stop-node':
      await stopNode();
      return true;
    case 'exit':
      return false;
  }
}

async function main(): Promise<void> {
  // Handle SIGINT
  process.on('SIGINT', () => {
    const state = getState();
    if (state.nodes.length > 0) {
      console.log('\n');
      console.log(info('Stopping all validators...'));
      stopAllNodes();
    }
    console.log(success('Goodbye!'));
    process.exit(0);
  });

  // Main loop
  let running = true;

  while (running) {
    renderHeader();

    const node = getActiveNode();

    if (!node) {
      running = await handleInitialMenu();
    } else {
      running = await handleMainMenu();
    }
  }

  // Cleanup - stop all nodes on exit
  const state = getState();
  if (state.nodes.length > 0) {
    console.log(info('Stopping all validators...'));
    stopAllNodes();
  }

  console.log(success('Goodbye!'));
  process.exit(0);
}

main().catch((err) => {
  console.error('Fatal error:', err);
  process.exit(1);
});
