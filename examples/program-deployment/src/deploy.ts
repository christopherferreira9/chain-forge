/**
 * Program Deployment Example - Using Default Accounts
 *
 * This example demonstrates how to deploy a Solana program using Chain Forge
 * with automatically generated accounts (no mnemonic required).
 *
 * Usage:
 *   yarn deploy <path-to-program.so>
 *
 * Example:
 *   yarn deploy ./target/deploy/my_program.so
 */

import { SolanaClient, SolanaAccount } from '@chain-forge/solana';
import { existsSync } from 'fs';
import { resolve } from 'path';

async function main() {
  // Get program path from command line arguments
  const programPath = process.argv[2];

  if (!programPath) {
    console.error('Usage: yarn deploy <path-to-program.so>');
    console.error('Example: yarn deploy ./target/deploy/my_program.so');
    process.exit(1);
  }

  const resolvedPath = resolve(programPath);

  if (!existsSync(resolvedPath)) {
    console.error(`Error: Program file not found: ${resolvedPath}`);
    process.exit(1);
  }

  console.log('Chain Forge - Program Deployment Example\n');
  console.log('Using default accounts (auto-generated)\n');

  // Create a client with default accounts
  const client = new SolanaClient({
    accounts: 3,
    initialBalance: 500, // SOL - enough to cover program deployment costs
    port: 8899,
  });

  try {
    // Start the validator
    console.log('Starting local Solana validator...');
    await client.start();
    console.log('Validator started!\n');

    // Get accounts
    const accounts = await client.getAccounts();
    console.log(`Generated ${accounts.length} accounts:`);
    accounts.forEach((acc: SolanaAccount, i: number) => {
      console.log(`  Account ${i}: ${acc.publicKey} (${acc.balance} SOL)`);
    });
    console.log();

    // Deploy the program using the first account as payer
    console.log(`Deploying program: ${resolvedPath}`);
    console.log(`Payer: Account 0 (${accounts[0].publicKey})\n`);

    const startTime = Date.now();
    const result = await client.deployProgram(resolvedPath, {
      payerIndex: 0,
    });
    const duration = ((Date.now() - startTime) / 1000).toFixed(2);

    console.log('Deployment successful!\n');
    console.log('Deployment Details:');
    console.log(`  Program ID: ${result.programId}`);
    console.log(`  Signature: ${result.signature}`);
    console.log(`  Payer: ${result.payer}`);
    console.log(`  Program Size: ${result.programSize} bytes`);
    console.log(`  Duration: ${duration}s\n`);

    // Check payer balance after deployment
    const payerBalance = await client.getBalance(accounts[0].publicKey);
    console.log(`Payer balance after deployment: ${payerBalance.toFixed(4)} SOL\n`);

    console.log('You can now interact with your program at:');
    console.log(`  Program ID: ${result.programId}`);
    console.log(`  RPC URL: ${client.getRpcUrl()}\n`);

    console.log('Press Ctrl+C to stop the validator and exit.\n');

    // Keep the process alive
    await new Promise(() => {});
  } catch (error) {
    console.error('Deployment failed:', error);
    client.stop();
    process.exit(1);
  } finally {
    process.on('SIGINT', () => {
      console.log('\nStopping validator...');
      client.stop();
      console.log('Validator stopped. Goodbye!\n');
      process.exit(0);
    });
  }
}

main();
