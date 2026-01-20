import { select, input } from '@inquirer/prompts';
import ora from 'ora';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import { getActiveNode, addProgramAccount } from '../state';
import { DeployedProgram } from '../types';
import { success, error, info, warning, truncateAddress, dim } from '../ui/formatters';

const CANCEL_VALUE = '__cancel__';
const DEFAULT_DATA_SIZE = 8;

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

export async function createProgramAccount(): Promise<void> {
  const node = getActiveNode();
  if (!node) {
    console.log(error('No active node. Please deploy a node first.'));
    return;
  }

  // Check if any programs are deployed
  if (node.deployedPrograms.length === 0) {
    console.log(warning('No programs deployed yet.'));
    console.log(info('Deploy a program first using "Deploy Program".'));
    console.log();
    await waitForEnter();
    return;
  }

  console.log();

  // Select which program will own the account
  const programChoices: { name: string; value: DeployedProgram | typeof CANCEL_VALUE }[] = [
    ...node.deployedPrograms.map((p) => ({
      name: `${p.name} (${truncateAddress(p.programId, 6)})`,
      value: p as DeployedProgram,
    })),
    { name: dim('Cancel'), value: CANCEL_VALUE },
  ];

  const selectedProgramOrCancel = await select({
    message: 'Select program to own the account',
    choices: programChoices,
  });

  if (selectedProgramOrCancel === CANCEL_VALUE) {
    return;
  }
  const selectedProgram = selectedProgramOrCancel as DeployedProgram;

  // Select payer account
  const accountChoices: { name: string; value: number | typeof CANCEL_VALUE }[] = [
    ...node.accounts.map((acc, i) => ({
      name: `Account ${i}: ${truncateAddress(acc.publicKey, 6)} (${acc.balance.toFixed(2)} SOL)`,
      value: i,
    })),
    { name: dim('Cancel'), value: CANCEL_VALUE },
  ];

  const payerIndexOrCancel = await select({
    message: 'Select payer account (will sign and pay rent)',
    choices: accountChoices,
  });

  if (payerIndexOrCancel === CANCEL_VALUE) {
    return;
  }
  const payerIndex = payerIndexOrCancel as number;
  const payerAccount = node.accounts[payerIndex];

  // Enter data size
  const sizeInput = await input({
    message: 'Data size in bytes',
    default: DEFAULT_DATA_SIZE.toString(),
    validate: (value) => {
      const size = parseInt(value, 10);
      if (isNaN(size) || size < 1 || size > 10240) {
        return 'Size must be between 1 and 10240 bytes';
      }
      return true;
    },
  });
  const dataSize = parseInt(sizeInput, 10);

  // Create the account
  const spinner = ora('Creating program-owned account...').start();

  try {
    const connection = node.client.getConnection();
    const payerKeypair = Keypair.fromSecretKey(new Uint8Array(payerAccount.secretKey));

    // Generate a new keypair for the account
    const newAccountKeypair = Keypair.generate();

    // Calculate rent-exempt minimum
    const rentExemptBalance = await connection.getMinimumBalanceForRentExemption(dataSize);

    // Create the account instruction
    const createAccountIx = SystemProgram.createAccount({
      fromPubkey: payerKeypair.publicKey,
      newAccountPubkey: newAccountKeypair.publicKey,
      lamports: rentExemptBalance,
      space: dataSize,
      programId: new PublicKey(selectedProgram.programId),
    });

    const transaction = new Transaction().add(createAccountIx);

    // Both payer and new account must sign
    const signature = await sendAndConfirmTransaction(connection, transaction, [
      payerKeypair,
      newAccountKeypair,
    ]);

    spinner.succeed('Account created successfully!');

    // Store in state
    addProgramAccount({
      address: newAccountKeypair.publicKey.toBase58(),
      programId: selectedProgram.programId,
      size: dataSize,
      createdBy: payerAccount.publicKey,
    });

    // Refresh payer balance
    const newBalance = await node.client.getBalance(payerAccount.publicKey);
    payerAccount.balance = newBalance;

    console.log();
    console.log(success('Program Account Created:'));
    console.log(`  Address: ${newAccountKeypair.publicKey.toBase58()}`);
    console.log(`  Owner: ${selectedProgram.name} (${truncateAddress(selectedProgram.programId, 6)})`);
    console.log(`  Size: ${dataSize} bytes`);
    console.log(`  Rent: ${(rentExemptBalance / 1e9).toFixed(6)} SOL`);
    console.log(`  Signature: ${signature}`);
    console.log();
    await waitForEnter();
  } catch (err) {
    spinner.fail('Failed to create account');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
  }
}
