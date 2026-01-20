import { input, select, confirm } from '@inquirer/prompts';
import ora from 'ora';
import { SolanaClient } from '@chain-forge/solana';
import { addNode, generateNodeId, getState } from '../state';
import { success, error, info, dim } from '../ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

const DEFAULT_PORT = 8899;
const DEFAULT_ACCOUNTS = 5;
const DEFAULT_BALANCE = 100;
const DEFAULT_MNEMONIC = 'test test test test test test test test test test test junk';

function suggestPort(): number {
  const state = getState();
  const usedPorts = state.nodes.map(n => n.port);

  let port = DEFAULT_PORT;
  while (usedPorts.includes(port)) {
    port++;
  }
  return port;
}

export async function deployNode(): Promise<boolean> {
  console.log();

  // Port selection
  const suggestedPort = suggestPort();
  const portInput = await input({
    message: 'RPC Port',
    default: suggestedPort.toString(),
    validate: (value) => {
      const port = parseInt(value, 10);
      if (isNaN(port) || port < 1024 || port > 65535) {
        return 'Port must be between 1024 and 65535';
      }
      return true;
    },
  });
  const port = parseInt(portInput, 10);

  // Mnemonic selection
  const mnemonicChoice = await select({
    message: 'Mnemonic for account generation',
    choices: [
      {
        name: 'Use default (deterministic)',
        value: 'default',
        description: 'Use the standard test mnemonic for reproducible accounts',
      },
      {
        name: 'Generate random',
        value: 'random',
        description: 'Let the validator generate random accounts',
      },
      {
        name: 'Enter custom',
        value: 'custom',
        description: 'Enter your own 12-word mnemonic',
      },
    ],
  });

  let mnemonic: string | null = null;
  if (mnemonicChoice === 'default') {
    mnemonic = DEFAULT_MNEMONIC;
  } else if (mnemonicChoice === 'custom') {
    mnemonic = await input({
      message: 'Enter 12-word mnemonic',
      validate: (value) => {
        const words = value.trim().split(/\s+/);
        if (words.length !== 12) {
          return 'Mnemonic must be exactly 12 words';
        }
        return true;
      },
    });
  }

  // Number of accounts
  const accountsInput = await input({
    message: 'Number of accounts',
    default: DEFAULT_ACCOUNTS.toString(),
    validate: (value) => {
      const num = parseInt(value, 10);
      if (isNaN(num) || num < 1 || num > 100) {
        return 'Number of accounts must be between 1 and 100';
      }
      return true;
    },
  });
  const accounts = parseInt(accountsInput, 10);

  // Initial balance
  const balanceInput = await input({
    message: 'Initial balance per account (SOL)',
    default: DEFAULT_BALANCE.toString(),
    validate: (value) => {
      const num = parseFloat(value);
      if (isNaN(num) || num < 1 || num > 10000) {
        return 'Balance must be between 1 and 10000 SOL';
      }
      return true;
    },
  });
  const initialBalance = parseFloat(balanceInput);

  // Confirmation
  console.log();
  console.log(info('Node Configuration:'));
  console.log(`  Port: ${port}`);
  console.log(`  Mnemonic: ${mnemonic ? 'Custom/Default' : 'Random'}`);
  console.log(`  Accounts: ${accounts}`);
  console.log(`  Initial Balance: ${initialBalance} SOL`);
  console.log();

  const confirmed = await confirm({
    message: 'Start the validator with these settings?',
    default: true,
  });

  if (!confirmed) {
    console.log(info('Node deployment cancelled.'));
    return false;
  }

  // Start the validator
  const spinner = ora('Starting Solana test validator...').start();

  try {
    const client = new SolanaClient({
      port,
      accounts,
      initialBalance,
      mnemonic: mnemonic || undefined,
    });

    await client.start();
    spinner.succeed('Validator started successfully!');

    // Get accounts
    spinner.start('Fetching accounts...');
    const accountList = await client.getAccounts();
    spinner.succeed(`Loaded ${accountList.length} accounts`);

    // Add to state
    addNode({
      id: generateNodeId(),
      port,
      mnemonic,
      client,
      accounts: accountList,
      deployedPrograms: [],
      programAccounts: [],
    });

    console.log();
    console.log(success('Node is ready!'));
    console.log();

    return true;
  } catch (err) {
    spinner.fail('Failed to start validator');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
    return false;
  }
}
