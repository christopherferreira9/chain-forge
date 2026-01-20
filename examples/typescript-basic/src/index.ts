/**
 * Chain Forge - Basic TypeScript Example
 *
 * A minimal example demonstrating:
 * 1. Starting a local Solana validator
 * 2. Getting pre-funded accounts
 * 3. Transferring SOL between accounts
 *
 * Designed for CI - runs to completion without interaction.
 */

import { SolanaClient } from '@chain-forge/solana';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';

async function main() {
  console.log('Chain Forge - Basic TypeScript Example\n');

  // Create client with 3 accounts
  const client = new SolanaClient({
    accounts: 3,
    initialBalance: 100,
    port: 8899,
  });

  try {
    // Start validator
    console.log('Starting validator...');
    await client.start();
    console.log('Validator started!\n');

    // Get accounts
    const accounts = await client.getAccounts();
    console.log(`Generated ${accounts.length} accounts:`);
    for (let i = 0; i < accounts.length; i++) {
      console.log(`  [${i}] ${accounts[i].publicKey} - ${accounts[i].balance} SOL`);
    }
    console.log();

    // Transfer 5 SOL from account 0 to account 1
    console.log('Sending 5 SOL from account 0 to account 1...');

    const sender = accounts[0];
    const receiver = accounts[1];
    const senderKeypair = Keypair.fromSecretKey(new Uint8Array(sender.secretKey));
    const connection = client.getConnection();

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: new PublicKey(sender.publicKey),
        toPubkey: new PublicKey(receiver.publicKey),
        lamports: 5 * LAMPORTS_PER_SOL,
      })
    );

    const signature = await sendAndConfirmTransaction(connection, transaction, [senderKeypair]);
    console.log(`Transaction confirmed: ${signature.slice(0, 20)}...`);

    // Show updated balances
    console.log('\nUpdated balances:');
    for (let i = 0; i < accounts.length; i++) {
      const balance = await client.getBalance(accounts[i].publicKey);
      console.log(`  [${i}] ${balance.toFixed(2)} SOL`);
    }

    console.log('\nExample completed successfully!');
  } finally {
    client.stop();
  }
}

main().catch((err) => {
  console.error('Error:', err);
  process.exit(1);
});
