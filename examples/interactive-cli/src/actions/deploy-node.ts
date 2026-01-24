import { input, select, confirm } from '@inquirer/prompts';
import ora from 'ora';
import { SolanaClient } from '@chain-forge/solana';
import { BitcoinClient } from '@chain-forge/bitcoin';
import { addNode, generateNodeId, getState } from '../state';
import { ChainType, SolanaNodeState, BitcoinNodeState } from '../types';
import { success, error, info, dim } from '../ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

const SOLANA_DEFAULT_PORT = 8899;
const BITCOIN_DEFAULT_PORT = 18443;
const DEFAULT_ACCOUNTS = 5;
const SOLANA_DEFAULT_BALANCE = 100;
const BITCOIN_DEFAULT_BALANCE = 10;
const DEFAULT_MNEMONIC = 'test test test test test test test test test test test junk';

function suggestPort(chainType: ChainType): number {
  const state = getState();
  const usedPorts = state.nodes.map(n => n.port);
  const defaultPort = chainType === 'solana' ? SOLANA_DEFAULT_PORT : BITCOIN_DEFAULT_PORT;

  let port = defaultPort;
  while (usedPorts.includes(port)) {
    port++;
  }
  return port;
}

export async function deployNode(): Promise<boolean> {
  console.log();

  // Chain selection
  const chainType = await select<ChainType>({
    message: 'Select blockchain',
    choices: [
      {
        name: 'Solana',
        value: 'solana',
        description: 'Start a Solana test validator',
      },
      {
        name: 'Bitcoin',
        value: 'bitcoin',
        description: 'Start a Bitcoin regtest node',
      },
    ],
  });

  if (chainType === 'solana') {
    return await deploySolanaNode();
  } else {
    return await deployBitcoinNode();
  }
}

async function deploySolanaNode(): Promise<boolean> {
  // Port selection
  const suggestedPort = suggestPort('solana');
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
    default: SOLANA_DEFAULT_BALANCE.toString(),
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
  console.log(info('Solana Node Configuration:'));
  console.log(`  Port: ${port}`);
  console.log(`  Mnemonic: ${mnemonic ? 'Custom/Default' : 'Random'}`);
  console.log(`  Accounts: ${accounts}`);
  console.log(`  Initial Balance: ${initialBalance} SOL`);
  console.log();

  const confirmed = await confirm({
    message: 'Start the Solana validator with these settings?',
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
    spinner.succeed('Solana validator started successfully!');

    // Get accounts
    spinner.start('Fetching accounts...');
    const accountList = await client.getAccounts();
    spinner.succeed(`Loaded ${accountList.length} accounts`);

    // Add to state
    const nodeState: SolanaNodeState = {
      chainType: 'solana',
      id: generateNodeId(),
      port,
      mnemonic,
      client,
      accounts: accountList,
      deployedPrograms: [],
      programAccounts: [],
    };
    addNode(nodeState);

    console.log();
    console.log(success('Solana node is ready!'));
    console.log();

    return true;
  } catch (err) {
    spinner.fail('Failed to start Solana validator');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
    return false;
  }
}

async function deployBitcoinNode(): Promise<boolean> {
  // Port selection
  const suggestedPort = suggestPort('bitcoin');
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
  const rpcPort = parseInt(portInput, 10);

  // P2P Port
  const p2pPortInput = await input({
    message: 'P2P Port',
    default: (rpcPort + 1).toString(),
    validate: (value) => {
      const port = parseInt(value, 10);
      if (isNaN(port) || port < 1024 || port > 65535) {
        return 'Port must be between 1024 and 65535';
      }
      return true;
    },
  });
  const p2pPort = parseInt(p2pPortInput, 10);

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
        description: 'Generate random accounts',
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
    message: 'Initial balance per account (BTC)',
    default: BITCOIN_DEFAULT_BALANCE.toString(),
    validate: (value) => {
      const num = parseFloat(value);
      if (isNaN(num) || num < 0.001 || num > 1000) {
        return 'Balance must be between 0.001 and 1000 BTC';
      }
      return true;
    },
  });
  const initialBalance = parseFloat(balanceInput);

  // Confirmation
  console.log();
  console.log(info('Bitcoin Node Configuration:'));
  console.log(`  RPC Port: ${rpcPort}`);
  console.log(`  P2P Port: ${p2pPort}`);
  console.log(`  Mnemonic: ${mnemonic ? 'Custom/Default' : 'Random'}`);
  console.log(`  Accounts: ${accounts}`);
  console.log(`  Initial Balance: ${initialBalance} BTC`);
  console.log();

  const confirmed = await confirm({
    message: 'Start the Bitcoin regtest node with these settings?',
    default: true,
  });

  if (!confirmed) {
    console.log(info('Node deployment cancelled.'));
    return false;
  }

  // Start the node
  const spinner = ora('Starting Bitcoin regtest node (this may take a moment)...').start();

  try {
    const client = new BitcoinClient({
      accounts,
      initialBalance,
      rpcPort,
      p2pPort,
      mnemonic: mnemonic || undefined,
    });

    await client.start();
    spinner.succeed('Bitcoin regtest node started successfully!');

    // Get accounts
    spinner.start('Fetching accounts...');
    const accountList = await client.getAccounts();
    spinner.succeed(`Loaded ${accountList.length} accounts`);

    // Add to state
    const nodeState: BitcoinNodeState = {
      chainType: 'bitcoin',
      id: generateNodeId(),
      port: rpcPort,
      mnemonic,
      client,
      accounts: accountList,
    };
    addNode(nodeState);

    console.log();
    console.log(success('Bitcoin node is ready!'));
    console.log();

    return true;
  } catch (err) {
    spinner.fail('Failed to start Bitcoin node');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
    return false;
  }
}
