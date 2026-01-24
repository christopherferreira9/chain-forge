import { select, input } from '@inquirer/prompts';
import ora from 'ora';
import {
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  Keypair,
} from '@solana/web3.js';
import { getActiveSolanaNode } from '../state';
import { loadIDL } from '../utils/idl';
import { DeployedProgram, IDLInstruction, IDLInstructionArg } from '../types';
import { success, error, info, warning, truncateAddress, dim } from '../ui/formatters';

const CANCEL_VALUE = '__cancel__';
const CUSTOM_ADDRESS_VALUE = '__custom__';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

/**
 * Serialize instruction arguments based on their types.
 */
function serializeArgs(args: IDLInstructionArg[], values: Record<string, string>): Buffer {
  const buffers: Buffer[] = [];

  for (const arg of args) {
    const value = values[arg.name];
    switch (arg.type) {
      case 'u8': {
        const buf = Buffer.alloc(1);
        buf.writeUInt8(parseInt(value, 10));
        buffers.push(buf);
        break;
      }
      case 'u64': {
        const buf = Buffer.alloc(8);
        buf.writeBigUInt64LE(BigInt(value));
        buffers.push(buf);
        break;
      }
      case 'string': {
        const strBytes = Buffer.from(value, 'utf-8');
        const lenBuf = Buffer.alloc(4);
        lenBuf.writeUInt32LE(strBytes.length);
        buffers.push(lenBuf);
        buffers.push(strBytes);
        break;
      }
      case 'pubkey': {
        buffers.push(new PublicKey(value).toBuffer());
        break;
      }
    }
  }

  return Buffer.concat(buffers);
}

export async function interactProgram(): Promise<void> {
  const node = getActiveSolanaNode();
  if (!node) {
    console.log(error('No active Solana node. Please deploy a Solana node first.'));
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

  // Filter programs that have IDL files
  const programsWithIDL = node.deployedPrograms.filter((p) => p.idlPath);

  if (programsWithIDL.length === 0) {
    console.log(warning('No deployed programs have IDL files.'));
    console.log(info('Create a <program_name>.idl.json file in the program directory to enable interaction.'));
    console.log();
    await waitForEnter();
    return;
  }

  console.log();

  // Select a program
  const programChoices: { name: string; value: DeployedProgram | typeof CANCEL_VALUE }[] = [
    ...programsWithIDL.map((p) => ({
      name: `${p.name} (${truncateAddress(p.programId, 6)})`,
      value: p as DeployedProgram,
    })),
    { name: dim('Cancel'), value: CANCEL_VALUE },
  ];

  const selectedProgramOrCancel = await select({
    message: 'Select a program to interact with',
    choices: programChoices,
  });

  if (selectedProgramOrCancel === CANCEL_VALUE) {
    return;
  }
  const selectedProgram = selectedProgramOrCancel as DeployedProgram;

  // Load the IDL
  const idl = loadIDL(selectedProgram.idlPath!);
  if (!idl) {
    console.log(error('Failed to load IDL file.'));
    console.log();
    await waitForEnter();
    return;
  }

  // Select an account to use as payer/signer
  const accountChoices: { name: string; value: number | typeof CANCEL_VALUE }[] = [
    ...node.accounts.map((acc, i) => ({
      name: `Account ${i}: ${truncateAddress(acc.publicKey, 6)} (${acc.balance.toFixed(2)} SOL)`,
      value: i,
    })),
    { name: dim('Cancel'), value: CANCEL_VALUE },
  ];

  const payerIndexOrCancel = await select({
    message: 'Select account for transaction',
    choices: accountChoices,
  });

  if (payerIndexOrCancel === CANCEL_VALUE) {
    return;
  }
  const payerIndex = payerIndexOrCancel as number;
  const payerAccount = node.accounts[payerIndex];

  // Select an instruction
  const instructionChoices: { name: string; value: IDLInstruction | typeof CANCEL_VALUE }[] = [
    ...idl.instructions.map((instr) => ({
      name: instr.description ? `${instr.name} - ${instr.description}` : instr.name,
      value: instr as IDLInstruction,
    })),
    { name: dim('Cancel'), value: CANCEL_VALUE },
  ];

  const selectedInstructionOrCancel = await select({
    message: 'Select instruction to execute',
    choices: instructionChoices,
  });

  if (selectedInstructionOrCancel === CANCEL_VALUE) {
    return;
  }
  const selectedInstruction = selectedInstructionOrCancel as IDLInstruction;

  // Collect account addresses for the instruction
  const keys: { pubkey: PublicKey; isSigner: boolean; isWritable: boolean }[] = [];

  for (const accountMeta of selectedInstruction.accounts) {
    // For signer accounts, use the payer account
    if (accountMeta.isSigner) {
      keys.push({
        pubkey: new PublicKey(payerAccount.publicKey),
        isSigner: true,
        isWritable: accountMeta.isWritable,
      });
      console.log(info(`Using payer account for "${accountMeta.name}" (signer)`));
    } else {
      // Provide account selection for non-signer accounts
      const description = accountMeta.description ? ` - ${accountMeta.description}` : '';

      // Filter program accounts owned by the current program
      const relevantProgramAccounts = node.programAccounts.filter(
        (pa) => pa.programId === selectedProgram.programId
      );

      const addressChoices: { name: string; value: string }[] = [];

      // Add program accounts first (most relevant for program instructions)
      if (relevantProgramAccounts.length > 0) {
        for (const pa of relevantProgramAccounts) {
          addressChoices.push({
            name: `Program Account: ${truncateAddress(pa.address, 6)} (${pa.size} bytes)`,
            value: pa.address,
          });
        }
      }

      // Add regular wallet accounts
      for (let i = 0; i < node.accounts.length; i++) {
        const acc = node.accounts[i];
        addressChoices.push({
          name: `Account ${i}: ${acc.publicKey}`,
          value: acc.publicKey,
        });
      }

      addressChoices.push(
        { name: 'Enter custom address', value: CUSTOM_ADDRESS_VALUE },
        { name: dim('Cancel'), value: CANCEL_VALUE }
      );

      const selectedAddressOrAction = await select({
        message: `Select ${accountMeta.name} account${description}`,
        choices: addressChoices,
      });

      if (selectedAddressOrAction === CANCEL_VALUE) {
        return;
      }

      let address: string;
      if (selectedAddressOrAction === CUSTOM_ADDRESS_VALUE) {
        address = await input({
          message: `Enter ${accountMeta.name} account address`,
          validate: (value) => {
            if (!value.trim()) {
              return 'Account address is required';
            }
            try {
              new PublicKey(value);
              return true;
            } catch {
              return 'Invalid Solana address';
            }
          },
        });
      } else {
        address = selectedAddressOrAction;
      }

      keys.push({
        pubkey: new PublicKey(address),
        isSigner: false,
        isWritable: accountMeta.isWritable,
      });
    }
  }

  // Collect instruction arguments
  const argValues: Record<string, string> = {};
  if (selectedInstruction.args && selectedInstruction.args.length > 0) {
    console.log();
    console.log(info('Enter instruction arguments:'));

    for (const arg of selectedInstruction.args) {
      const description = arg.description ? ` (${arg.description})` : '';
      const value = await input({
        message: `${arg.name} (${arg.type})${description}`,
        validate: (val) => {
          if (!val.trim()) {
            return 'Value is required';
          }
          switch (arg.type) {
            case 'u8':
            case 'u64': {
              const num = parseInt(val, 10);
              if (isNaN(num) || num < 0) {
                return 'Must be a non-negative integer';
              }
              return true;
            }
            case 'pubkey': {
              try {
                new PublicKey(val);
                return true;
              } catch {
                return 'Invalid public key';
              }
            }
            case 'string':
              return true;
          }
        },
      });
      argValues[arg.name] = value;
    }
  }

  // Build instruction data
  // First byte is the discriminator, followed by serialized args
  const discriminatorBuf = Buffer.alloc(1);
  discriminatorBuf.writeUInt8(selectedInstruction.discriminator);

  let instructionData: Buffer;
  if (selectedInstruction.args && selectedInstruction.args.length > 0) {
    const argData = serializeArgs(selectedInstruction.args, argValues);
    instructionData = Buffer.concat([discriminatorBuf, argData]);
  } else {
    instructionData = discriminatorBuf;
  }

  // Build and send transaction
  const spinner = ora(`Executing ${selectedInstruction.name} instruction...`).start();

  try {
    const connection = node.client.getConnection();
    const payerKeypair = Keypair.fromSecretKey(new Uint8Array(payerAccount.secretKey));

    const instruction = new TransactionInstruction({
      keys,
      programId: new PublicKey(selectedProgram.programId),
      data: instructionData,
    });

    const transaction = new Transaction().add(instruction);

    const signature = await sendAndConfirmTransaction(connection, transaction, [payerKeypair]);

    spinner.succeed('Transaction confirmed!');

    console.log();
    console.log(success('Transaction Result:'));
    console.log(`  Signature: ${signature}`);

    // Fetch transaction to get logs
    const txDetails = await connection.getTransaction(signature, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });

    if (txDetails?.meta?.logMessages) {
      console.log();
      console.log(info('Program Logs:'));
      for (const log of txDetails.meta.logMessages) {
        // Filter to show only program logs (not system messages)
        if (log.startsWith('Program log:')) {
          console.log(`  ${dim(log)}`);
        }
      }
    }

    // Refresh payer balance
    const newBalance = await node.client.getBalance(payerAccount.publicKey);
    payerAccount.balance = newBalance;

    console.log();
    await waitForEnter();
  } catch (err) {
    spinner.fail('Transaction failed');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
  }
}
