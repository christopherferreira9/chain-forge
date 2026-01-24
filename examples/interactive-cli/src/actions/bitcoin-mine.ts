import { select, input, confirm } from '@inquirer/prompts';
import ora from 'ora';
import { getActiveBitcoinNode } from '../state';
import { success, error, info, truncateAddress, dim } from '../ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

export async function bitcoinMine(): Promise<void> {
  const node = getActiveBitcoinNode();
  if (!node) {
    console.log(error('No active Bitcoin node. Please deploy a Bitcoin node first.'));
    return;
  }

  console.log();

  // Number of blocks to mine
  const blocksInput = await input({
    message: 'Number of blocks to mine',
    default: '1',
    validate: (value) => {
      const num = parseInt(value, 10);
      if (isNaN(num) || num < 1 || num > 1000) {
        return 'Number of blocks must be between 1 and 1000';
      }
      return true;
    },
  });
  const blocks = parseInt(blocksInput, 10);

  // Select address to receive coinbase rewards
  const addressChoices = [
    ...node.accounts.map((acc, i) => ({
      name: `Account ${i}: ${truncateAddress(acc.address, 8)}`,
      value: acc.address,
    })),
    {
      name: 'Enter custom address',
      value: '__custom__',
    },
  ];

  let minerAddress = await select({
    message: 'Select address to receive coinbase rewards',
    choices: addressChoices,
  });

  if (minerAddress === '__custom__') {
    minerAddress = await input({
      message: 'Enter miner address',
      validate: (value) => {
        if (value.length < 26 || value.length > 90) {
          return 'Invalid Bitcoin address length';
        }
        return true;
      },
    });
  }

  // Confirmation
  console.log();
  console.log(info('Mining Configuration:'));
  console.log(`  Blocks: ${blocks}`);
  console.log(`  Miner: ${truncateAddress(minerAddress, 8)}`);
  console.log();

  const confirmed = await confirm({
    message: `Mine ${blocks} block${blocks > 1 ? 's' : ''}?`,
    default: true,
  });

  if (!confirmed) {
    console.log(info('Mining cancelled.'));
    return;
  }

  // Mine blocks
  const spinner = ora(`Mining ${blocks} block${blocks > 1 ? 's' : ''}...`).start();

  try {
    const result = await node.client.mine(blocks, minerAddress);
    spinner.succeed(`Mined ${result.blockHashes.length} block${result.blockHashes.length > 1 ? 's' : ''}!`);

    console.log();
    console.log(success('Mining Result:'));

    // Show first few block hashes
    const hashesToShow = result.blockHashes.slice(0, 5);
    for (let i = 0; i < hashesToShow.length; i++) {
      console.log(`  Block ${i + 1}: ${truncateAddress(hashesToShow[i], 12)}`);
    }
    if (result.blockHashes.length > 5) {
      console.log(`  ... and ${result.blockHashes.length - 5} more blocks`);
    }

    if (result.height) {
      console.log(`  Current height: ${result.height}`);
    }

    // Refresh balances
    spinner.start('Refreshing balances...');
    const updatedAccounts = await node.client.getAccounts();
    for (let i = 0; i < node.accounts.length; i++) {
      const updated = updatedAccounts.find(a => a.address === node.accounts[i].address);
      if (updated) {
        node.accounts[i].balance = updated.balance;
      }
    }
    spinner.succeed('Balances updated');

    console.log();
    await waitForEnter();
  } catch (err) {
    spinner.fail('Mining failed');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
  }
}
