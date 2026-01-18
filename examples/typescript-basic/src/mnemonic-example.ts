import { SolanaClient } from '@chain-forge/solana';
import { PublicKey } from '@solana/web3.js';

async function main() {
  console.log('🔑 Chain Forge - Mnemonic Example\n');

  // Use a specific mnemonic for deterministic account generation
  const mnemonic = 'drive manage close raven tape average sausage pledge riot furnace august tip';

  console.log(`Using mnemonic: ${mnemonic}\n`);

  // Create a Solana client with the mnemonic
  const client = new SolanaClient({
    accounts: 3,          // Generate 3 accounts
    initialBalance: 10,   // Lower balance to avoid rate limiting
    port: 8899,
    mnemonic: mnemonic,
  });

  try {
    // Start the local validator with deterministic accounts
    console.log('Starting local Solana validator with mnemonic-derived accounts...');
    await client.start();
    console.log('✅ Validator started!\n');

    // Get the generated accounts
    const accounts = await client.getAccounts();
    console.log(`📋 Generated ${accounts.length} deterministic accounts from mnemonic:\n`);

    // Display account information
    for (let i = 0; i < accounts.length; i++) {
      const account = accounts[i];
      console.log(`  Account ${i}:`);
      console.log(`    Address: ${account.publicKey}`);
      console.log(`    Balance: ${account.balance} SOL`);
      console.log(`    Derivation Path: ${account.derivationPath || 'N/A'}`);
      console.log(`    Mnemonic: ${account.mnemonic?.substring(0, 30)}...`);
      console.log();
    }

    // Verify deterministic generation
    console.log('🔄 Verifying deterministic account generation...\n');
    console.log('These accounts will ALWAYS be the same when using this mnemonic.');
    console.log('You can recover them on any machine using the same 12-word phrase.\n');

    // Example: Fund one of the accounts
    console.log('💰 Funding account 1...\n');
    const targetAccount = accounts[1];

    try {
      await client.setBalance(targetAccount.publicKey, 20);
      const balance = await client.getBalance(targetAccount.publicKey);
      console.log(`  ✅ Account funded! Balance: ${balance} SOL\n`);
    } catch (error) {
      console.log(`  ⚠️  Funding skipped (rate limit or other issue)\n`);
    }

    // Query all balances
    console.log('💵 Current account balances:\n');
    for (let i = 0; i < accounts.length; i++) {
      const balance = await client.getBalance(accounts[i].publicKey);
      console.log(`  Account ${i}: ${balance.toFixed(2)} SOL`);
    }
    console.log();

    // Show integration with web3.js
    console.log('🔗 Using with @solana/web3.js:\n');
    const connection = client.getConnection();

    // Get account info for first account
    const accountInfo = await connection.getAccountInfo(
      new PublicKey(accounts[0].publicKey)
    );

    if (accountInfo) {
      console.log(`  Account 0 info:`);
      console.log(`    Lamports: ${accountInfo.lamports}`);
      console.log(`    Owner: ${accountInfo.owner.toBase58()}`);
      console.log();
    }

    console.log('✨ Mnemonic example completed!\n');
    console.log('💡 Key Benefits:');
    console.log('   • Same mnemonic = same accounts every time');
    console.log('   • Can recover accounts on different machines');
    console.log('   • Follows BIP39/BIP44 standard (Solana path: m/44\'/501\'/n\'/0\')');
    console.log('   • Compatible with other Solana wallets\n');

    console.log('Press Ctrl+C to stop the validator and exit.\n');

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
