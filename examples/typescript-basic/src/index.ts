import { SolanaClient } from '@chain-forge/solana';
import { Connection, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';

async function main() {
  console.log('🚀 Chain Forge - TypeScript Example\n');

  // Create a Solana client with custom configuration
  const client = new SolanaClient({
    accounts: 5,
    initialBalance: 50,
    port: 8899,
  });

  try {
    // Start the local validator
    console.log('Starting local Solana validator...');
    await client.start();
    console.log('✅ Validator started!\n');

    // Get the generated accounts
    const accounts = await client.getAccounts();
    console.log(`📋 Generated ${accounts.length} accounts:\n`);

    // Display account information
    for (let i = 0; i < accounts.length; i++) {
      const account = accounts[i];
      console.log(`  Account ${i}:`);
      console.log(`    Address: ${account.publicKey}`);
      console.log(`    Balance: ${account.balance} SOL`);
      if (account.derivationPath) {
        console.log(`    Path: ${account.derivationPath}`);
      }
      console.log();
    }

    // Example 1: Fund an account
    console.log('💰 Example 1: Funding an account\n');
    const targetAccount = accounts[0];
    console.log(`  Requesting airdrop of 10 SOL to ${targetAccount.publicKey}...`);

    const signature = await client.setBalance(targetAccount.publicKey, 10);
    console.log(`  ✅ Airdrop successful! Signature: ${signature}\n`);

    // Check the updated balance
    const newBalance = await client.getBalance(targetAccount.publicKey);
    console.log(`  Updated balance: ${newBalance} SOL\n`);

    // Example 2: Using Solana Web3.js directly
    console.log('🔗 Example 2: Using Solana Web3.js directly\n');

    const connection = client.getConnection();

    // Get cluster version
    const version = await connection.getVersion();
    console.log(`  Cluster version: ${version['solana-core']}`);

    // Get block height
    const blockHeight = await connection.getBlockHeight();
    console.log(`  Current block height: ${blockHeight}`);

    // Get slot
    const slot = await connection.getSlot();
    console.log(`  Current slot: ${slot}\n`);

    // Example 3: Query multiple account balances
    console.log('💵 Example 3: Query all account balances\n');

    for (let i = 0; i < accounts.length; i++) {
      const balance = await client.getBalance(accounts[i].publicKey);
      console.log(`  Account ${i}: ${balance.toFixed(2)} SOL`);
    }
    console.log();

    // Example 4: Get account info using web3.js
    console.log('📊 Example 4: Get detailed account info\n');

    const accountInfo = await connection.getAccountInfo(
      new PublicKey(accounts[0].publicKey)
    );

    if (accountInfo) {
      console.log(`  Lamports: ${accountInfo.lamports}`);
      console.log(`  Owner: ${accountInfo.owner.toBase58()}`);
      console.log(`  Executable: ${accountInfo.executable}`);
      console.log(`  Rent Epoch: ${accountInfo.rentEpoch}\n`);
    }

    console.log('✨ All examples completed successfully!\n');
    console.log('💡 Tip: The validator will continue running until you stop it.');
    console.log('   Press Ctrl+C to stop the validator and exit.\n');

    // Keep the process alive until Ctrl+C
    await new Promise(() => {});
  } catch (error) {
    console.error('❌ Error:', error);
    process.exit(1);
  } finally {
    // Clean up - stop the validator
    process.on('SIGINT', () => {
      console.log('\n\n🛑 Stopping validator...');
      client.stop();
      console.log('✅ Validator stopped. Goodbye!\n');
      process.exit(0);
    });
  }
}

main();
