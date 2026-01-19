/**
 * Chain Forge - Program Deployment Example
 *
 * This example shows the programmatic usage of program deployment.
 * For CLI-based deployment, use:
 *   - yarn deploy <program.so>           - Deploy with default accounts
 *   - yarn deploy:mnemonic <program.so>  - Deploy with mnemonic-based accounts
 *
 * This script demonstrates the complete deployment workflow:
 * 1. Start local validator with pre-funded accounts
 * 2. Get accounts and keypairs
 * 3. Deploy a program
 * 4. Interact with the deployed program
 */

import { SolanaClient, SolanaAccount, DeployProgramResult } from '@chain-forge/solana';
import { Connection, PublicKey, TransactionInstruction, Transaction, sendAndConfirmTransaction } from '@solana/web3.js';
import { existsSync } from 'fs';
import { resolve } from 'path';

// Configuration
const CONFIG = {
  accounts: 5,
  initialBalance: 500,
  port: 8899,
  // Uncomment to use a fixed mnemonic for reproducible accounts:
  // mnemonic: 'test test test test test test test test test test test junk',
};

async function deployProgram(client: SolanaClient, programPath: string): Promise<DeployProgramResult | null> {
  const resolvedPath = resolve(programPath);

  if (!existsSync(resolvedPath)) {
    console.log(`Program file not found: ${resolvedPath}`);
    console.log('Skipping deployment demonstration.\n');
    console.log('To test deployment, provide a compiled Solana program:');
    console.log('  1. Build your program: cargo build-sbf');
    console.log('  2. Run: yarn dev <path-to-your-program.so>\n');
    return null;
  }

  console.log(`Deploying program: ${resolvedPath}\n`);

  const result = await client.deployProgram(resolvedPath, {
    payerIndex: 0, // Use first account as payer
  });

  console.log('Program deployed successfully!');
  console.log(`  Program ID: ${result.programId}`);
  console.log(`  Size: ${result.programSize} bytes\n`);

  return result;
}

async function demonstrateKeypairUsage(client: SolanaClient) {
  console.log('Demonstrating Keypair Access\n');

  // Get a keypair for signing transactions
  const keypair = await client.getKeypair(0);
  console.log(`Keypair 0 public key: ${keypair.publicKey.toBase58()}`);

  // Get another keypair
  const keypair1 = await client.getKeypair(1);
  console.log(`Keypair 1 public key: ${keypair1.publicKey.toBase58()}\n`);

  // These keypairs can be used for signing transactions
  console.log('These keypairs can be used for:');
  console.log('  - Signing transactions');
  console.log('  - Program deployment');
  console.log('  - Creating program-derived addresses (PDAs)');
  console.log('  - Any operation requiring a signer\n');
}

async function main() {
  console.log('Chain Forge - Program Deployment Demonstration\n');
  console.log('=' .repeat(50) + '\n');

  // Accept optional program path as argument
  const programPath = process.argv[2];

  const client = new SolanaClient(CONFIG);

  try {
    // Start validator
    console.log('1. Starting Local Validator\n');
    await client.start();
    console.log('Validator is running!\n');

    // Show accounts
    console.log('2. Available Accounts\n');
    const accounts = await client.getAccounts();
    accounts.forEach((acc: SolanaAccount, i: number) => {
      console.log(`   Account ${i}: ${acc.publicKey} (${acc.balance} SOL)`);
    });
    console.log();

    // Demonstrate keypair access
    console.log('3. Keypair Access\n');
    await demonstrateKeypairUsage(client);

    // Deploy program if path provided
    console.log('4. Program Deployment\n');
    if (programPath) {
      const deployResult = await deployProgram(client, programPath);

      if (deployResult) {
        // Show balance changes
        console.log('5. Balance After Deployment\n');
        for (let i = 0; i < accounts.length; i++) {
          const balance = await client.getBalance(accounts[i].publicKey);
          console.log(`   Account ${i}: ${balance.toFixed(4)} SOL`);
        }
        console.log();

        // Show how to interact with the program
        console.log('6. Interacting with Your Program\n');
        console.log('   Use the following to interact with your deployed program:\n');
        console.log('   ```typescript');
        console.log('   const connection = client.getConnection();');
        console.log(`   const programId = new PublicKey('${deployResult.programId}');`);
        console.log('   const payer = await client.getKeypair(0);');
        console.log('   ```\n');
      }
    } else {
      console.log('   No program path provided.');
      console.log('   To deploy a program, run: yarn dev <path-to-program.so>\n');
    }

    console.log('=' .repeat(50));
    console.log('\nValidator is running. Press Ctrl+C to stop.\n');

    // Keep alive
    await new Promise(() => {});
  } catch (error) {
    console.error('Error:', error);
    client.stop();
    process.exit(1);
  } finally {
    process.on('SIGINT', () => {
      console.log('\nStopping validator...');
      client.stop();
      console.log('Done!\n');
      process.exit(0);
    });
  }
}

main();
