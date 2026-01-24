import { select, confirm, input } from '@inquirer/prompts';
import ora from 'ora';
import { spawn } from 'child_process';
import { Keypair } from '@solana/web3.js';
import { getActiveSolanaNode, addDeployedProgram } from '../state';
import { discoverPrograms, getProgramSoPath, isProgramBuilt } from '../utils/programs';
import { getIDLPath, hasIDL } from '../utils/idl';
import { success, error, info, warning, formatSize, truncateAddress, dim } from '../ui/formatters';

async function waitForEnter(): Promise<void> {
  await input({ message: dim('Press Enter to continue...') });
}

export async function deployProgram(): Promise<void> {
  const node = getActiveSolanaNode();
  if (!node) {
    console.log(error('No active Solana node. Please deploy a Solana node first.'));
    return;
  }

  // Discover available programs
  const programs = discoverPrograms();

  if (programs.length === 0) {
    console.log(warning('No programs found in the programs/ directory.'));
    console.log(info('Create a Solana program in programs/<name>/ with a Cargo.toml file.'));
    return;
  }

  // Select program
  console.log();
  const programChoices = programs.map(p => ({
    name: `${p.name}${isProgramBuilt(p) ? ' (built)' : ''}`,
    value: p.name,
    description: p.path,
  }));

  const selectedName = await select({
    message: 'Select a program to deploy',
    choices: programChoices,
  });

  const program = programs.find(p => p.name === selectedName);
  if (!program) {
    console.log(error('Program not found.'));
    return;
  }

  // Select payer account
  const accountChoices = node.accounts.map((acc, i) => ({
    name: `Account ${i}: ${truncateAddress(acc.publicKey, 6)} (${acc.balance.toFixed(2)} SOL)`,
    value: i,
  }));

  const payerIndex = await select({
    message: 'Select payer account',
    choices: accountChoices,
  });

  // Build if not already built
  const soPath = getProgramSoPath(program);
  if (!isProgramBuilt(program)) {
    const shouldBuild = await confirm({
      message: `Program not built. Build now?`,
      default: true,
    });

    if (!shouldBuild) {
      console.log(info('Deployment cancelled.'));
      return;
    }

    const buildSpinner = ora('Building program with cargo build-sbf...').start();

    try {
      await new Promise<void>((resolve, reject) => {
        const proc = spawn('cargo', ['build-sbf'], {
          cwd: program.path,
          stdio: 'pipe',
        });

        let stderr = '';
        proc.stderr?.on('data', (data) => {
          stderr += data.toString();
        });

        proc.on('close', (code) => {
          if (code === 0) {
            resolve();
          } else {
            reject(new Error(stderr || `Build failed with code ${code}`));
          }
        });

        proc.on('error', reject);
      });

      buildSpinner.succeed('Program built successfully!');
    } catch (err) {
      buildSpinner.fail('Build failed');
      console.log(error(`Error: ${(err as Error).message}`));
      return;
    }
  }

  // Deploy with a fresh program keypair to create a new program instance
  const deploySpinner = ora('Deploying program...').start();

  try {
    // Generate a new keypair for this program deployment
    // This ensures each deployment creates a new program with a unique ID
    const programKeypair = Keypair.generate();

    const result = await node.client.deployProgram(soPath, {
      payerIndex,
      programKeypair: programKeypair.secretKey,
    });

    deploySpinner.succeed('Program deployed successfully!');

    // Add to state with IDL path if available
    const idlPath = hasIDL(program) ? getIDLPath(program) : undefined;
    addDeployedProgram({
      programId: result.programId,
      name: program.name,
      path: soPath,
      size: result.programSize,
      idlPath,
      deployerAddress: node.accounts[payerIndex].publicKey,
    });

    // Refresh payer balance
    const newBalance = await node.client.getBalance(node.accounts[payerIndex].publicKey);
    node.accounts[payerIndex].balance = newBalance;

    console.log();
    console.log(success('Deployment Result:'));
    console.log(`  Program ID: ${result.programId}`);
    console.log(`  Size: ${formatSize(result.programSize)}`);
    console.log(`  Payer: ${truncateAddress(result.payer, 6)} (${newBalance.toFixed(2)} SOL remaining)`);
    console.log();
    await waitForEnter();
  } catch (err) {
    deploySpinner.fail('Deployment failed');
    console.log(error(`Error: ${(err as Error).message}`));
    console.log();
    await waitForEnter();
  }
}
