import { select, input, confirm } from '@inquirer/prompts';
import ora from 'ora';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  sendAndConfirmTransaction,
  Keypair,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { getActiveNode, updateActiveNodeAccounts } from '../state';
import { success, error, info, truncateAddress, formatBalance, dim } from '../ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

export async function sendFunds(): Promise<void> {
  const node = getActiveNode();
  if (!node) {
    console.log(error('No active node. Please deploy a node first.'));
    return;
  }

  console.log();

  // Select source account
  const sourceChoices = node.accounts.map((acc, i) => ({
    name: `Account ${i}: ${truncateAddress(acc.publicKey, 6)} (${formatBalance(acc.balance)} SOL)`,
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
        name: `Account ${i >= sourceIndex ? i + 1 : i}: ${truncateAddress(acc.publicKey, 6)} (${formatBalance(acc.balance)} SOL)`,
        value: acc.publicKey,
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
        try {
          new PublicKey(value);
          return true;
        } catch {
          return 'Invalid Solana address';
        }
      },
    });
  }

  // Enter amount
  const maxAmount = sourceAccount.balance - 0.001; // Leave some for fees
  const amountInput = await input({
    message: `Amount to send (max: ${formatBalance(maxAmount)} SOL)`,
    validate: (value) => {
      const amount = parseFloat(value);
      if (isNaN(amount) || amount <= 0) {
        return 'Amount must be a positive number';
      }
      if (amount > maxAmount) {
        return `Amount exceeds maximum (${formatBalance(maxAmount)} SOL)`;
      }
      return true;
    },
  });
  const amount = parseFloat(amountInput);

  // Confirmation
  console.log();
  console.log(info('Transaction Summary:'));
  console.log(`  From: ${truncateAddress(sourceAccount.publicKey, 8)}`);
  console.log(`  To: ${truncateAddress(destAddress, 8)}`);
  console.log(`  Amount: ${formatBalance(amount)} SOL`);
  console.log();

  const confirmed = await confirm({
    message: 'Send this transaction?',
    default: true,
  });

  if (!confirmed) {
    console.log(info('Transaction cancelled.'));
    return;
  }

  // Execute transaction
  const spinner = ora('Sending transaction...').start();

  try {
    const connection = node.client.getConnection();
    const senderKeypair = Keypair.fromSecretKey(new Uint8Array(sourceAccount.secretKey));

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: new PublicKey(sourceAccount.publicKey),
        toPubkey: new PublicKey(destAddress),
        lamports: Math.floor(amount * LAMPORTS_PER_SOL),
      })
    );

    const signature = await sendAndConfirmTransaction(connection, transaction, [senderKeypair]);

    spinner.succeed('Transaction confirmed!');

    console.log();
    console.log(success('Transaction Result:'));
    console.log(`  Signature: ${truncateAddress(signature, 12)}`);

    // Refresh balances
    spinner.start('Refreshing balances...');
    const updatedAccounts = await node.client.getAccounts();
    for (let i = 0; i < node.accounts.length; i++) {
      node.accounts[i].balance = await node.client.getBalance(node.accounts[i].publicKey);
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
