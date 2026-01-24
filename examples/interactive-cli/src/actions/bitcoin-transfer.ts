import { select, input, confirm } from '@inquirer/prompts';
import ora from 'ora';
import { getActiveBitcoinNode } from '../state';
import { success, error, info, truncateAddress, formatBalance, dim } from '../ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

export async function bitcoinTransfer(): Promise<void> {
  const node = getActiveBitcoinNode();
  if (!node) {
    console.log(error('No active Bitcoin node. Please deploy a Bitcoin node first.'));
    return;
  }

  console.log();

  // Select source account
  const sourceChoices = node.accounts.map((acc, i) => ({
    name: `Account ${i}: ${truncateAddress(acc.address, 6)} (${formatBalance(acc.balance)} BTC)`,
    value: i,
  }));

  const sourceIndex = await select({
    message: 'Select source account',
    choices: sourceChoices,
  });

  const sourceAccount = node.accounts[sourceIndex];

  // Select destination
  const destChoices = [
    ...node.accounts
      .filter((_, i) => i !== sourceIndex)
      .map((acc, i) => ({
        name: `Account ${i >= sourceIndex ? i + 1 : i}: ${truncateAddress(acc.address, 6)} (${formatBalance(acc.balance)} BTC)`,
        value: acc.address,
      })),
    {
      name: 'Enter custom address',
      value: '__custom__',
    },
  ];

  let destAddress = await select({
    message: 'Select destination',
    choices: destChoices,
  });

  if (destAddress === '__custom__') {
    destAddress = await input({
      message: 'Enter destination address',
      validate: (value) => {
        // Basic Bitcoin address validation (bcrt1, tb1, bc1, or legacy)
        if (value.length < 26 || value.length > 90) {
          return 'Invalid Bitcoin address length';
        }
        return true;
      },
    });
  }

  // Enter amount
  const maxAmount = sourceAccount.balance - 0.0001; // Leave some for fees
  const amountInput = await input({
    message: `Amount to send (max: ${formatBalance(maxAmount)} BTC)`,
    validate: (value) => {
      const amount = parseFloat(value);
      if (isNaN(amount) || amount <= 0) {
        return 'Amount must be a positive number';
      }
      if (amount > maxAmount) {
        return `Amount exceeds maximum (${formatBalance(maxAmount)} BTC)`;
      }
      return true;
    },
  });
  const amount = parseFloat(amountInput);

  // Ask if user wants to mine a block to confirm
  const mineBlock = await confirm({
    message: 'Mine a block to confirm the transaction?',
    default: true,
  });

  // Confirmation
  console.log();
  console.log(info('Transaction Summary:'));
  console.log(`  From: ${truncateAddress(sourceAccount.address, 8)}`);
  console.log(`  To: ${truncateAddress(destAddress, 8)}`);
  console.log(`  Amount: ${formatBalance(amount)} BTC`);
  console.log(`  Mine block: ${mineBlock ? 'Yes' : 'No'}`);
  console.log();

  const confirmed = await confirm({
    message: 'Send this transaction?',
    default: true,
  });

  if (!confirmed) {
    console.log(info('Transaction cancelled.'));
    return;
  }

  // Execute transaction (transfer from source account to destination)
  const spinner = ora('Sending transaction...').start();

  try {
    const result = await node.client.transfer(sourceAccount.address, destAddress, amount);
    spinner.succeed('Transaction sent!');

    console.log();
    console.log(success('Transaction Result:'));
    console.log(`  TxID: ${truncateAddress(result.txid, 12)}`);

    // Mine a block if requested
    if (mineBlock) {
      spinner.start('Mining block to confirm transaction...');
      const mineResult = await node.client.mine(1);
      spinner.succeed('Block mined!');
      if (mineResult.height) {
        console.log(`  Block height: ${mineResult.height}`);
      }
    }

    // Refresh balances from blockchain
    spinner.start('Refreshing balances from blockchain...');
    const updatedAccounts = await node.client.refreshBalances();
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
    spinner.fail('Transaction failed');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
  }
}
