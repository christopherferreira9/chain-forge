/**
 * Program Deployment Example - Using a Mnemonic
 *
 * This example demonstrates how to deploy a Solana program using Chain Forge
 * with a specific mnemonic phrase for deterministic account generation.
 *
 * This is useful for:
 * - Reproducible deployments in CI/CD
 * - Team collaboration with shared test accounts
 * - Testing specific account scenarios
 *
 * Usage:
 *   yarn deploy:mnemonic <path-to-program.so> [mnemonic]
 *
 * Example:
 *   yarn deploy:mnemonic ./target/deploy/my_program.so
 *   yarn deploy:mnemonic ./target/deploy/my_program.so "your twelve word mnemonic phrase here..."
 */

import { SolanaClient, SolanaAccount } from '@chain-forge/solana';
import { existsSync } from 'fs';
import { resolve } from 'path';

// Default test mnemonic - DO NOT USE IN PRODUCTION
const DEFAULT_TEST_MNEMONIC = 'test test test test test test test test test test test junk';

async function main() {
  const programPath = process.argv[2];
  const mnemonic = process.argv[3] || DEFAULT_TEST_MNEMONIC;

  if (!programPath) {
    console.error('Usage: yarn deploy:mnemonic <path-to-program.so> [mnemonic]');
    console.error('Example: yarn deploy:mnemonic ./target/deploy/my_program.so');
    console.error('Example: yarn deploy:mnemonic ./my_program.so "word1 word2 ... word12"');
    process.exit(1);
  }

  const resolvedPath = resolve(programPath);

  if (!existsSync(resolvedPath)) {
    console.error(`Error: Program file not found: ${resolvedPath}`);
    process.exit(1);
  }

  console.log('Chain Forge - Program Deployment with Mnemonic\n');
  console.log('Using mnemonic for deterministic accounts\n');

  // Create a client with a specific mnemonic
  const client = new SolanaClient({
    accounts: 5,
    initialBalance: 500,
    port: 8899,
    mnemonic: mnemonic,
  });

  try {
    // Start the validator
    console.log('Starting local Solana validator...');
    await client.start();
    console.log('Validator started!\n');

    // Get accounts - these will always be the same for the same mnemonic
    const accounts = await client.getAccounts();
    console.log(`Generated ${accounts.length} deterministic accounts:`);
    accounts.forEach((acc: SolanaAccount, i: number) => {
      console.log(`  Account ${i}: ${acc.publicKey}`);
      if (acc.derivationPath) {
        console.log(`    Derivation path: ${acc.derivationPath}`);
      }
      console.log(`    Balance: ${acc.balance} SOL`);
    });
    console.log();

    // Deploy using account at index 1 (just to demonstrate using different accounts)
    const payerIndex = 1;
    console.log(`Deploying program: ${resolvedPath}`);
    console.log(`Payer: Account ${payerIndex} (${accounts[payerIndex].publicKey})\n`);

    const startTime = Date.now();
    const result = await client.deployProgram(resolvedPath, {
      payerIndex: payerIndex,
    });
    const duration = ((Date.now() - startTime) / 1000).toFixed(2);

    console.log('Deployment successful!\n');
    console.log('Deployment Details:');
    console.log(`  Program ID: ${result.programId}`);
    console.log(`  Signature: ${result.signature}`);
    console.log(`  Payer: ${result.payer}`);
    console.log(`  Program Size: ${result.programSize} bytes`);
    console.log(`  Duration: ${duration}s\n`);

    // Display all account balances after deployment
    console.log('Account balances after deployment:');
    for (let i = 0; i < accounts.length; i++) {
      const balance = await client.getBalance(accounts[i].publicKey);
      const marker = i === payerIndex ? ' (payer)' : '';
      console.log(`  Account ${i}${marker}: ${balance.toFixed(4)} SOL`);
    }
    console.log();

    console.log('Reproducibility Note:');
    console.log('  Running this script with the same mnemonic will always generate');
    console.log('  the same accounts, making deployments reproducible.\n');

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
