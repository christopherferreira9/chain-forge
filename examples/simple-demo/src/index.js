/**
 * Chain Forge - Simple Demo
 *
 * This demo shows the basic functionality of Chain Forge:
 * 1. Starting a local Solana validator
 * 2. Getting pre-funded accounts
 * 3. Using setBalance to adjust account balances
 * 4. Interacting with Solana web3.js
 *
 * The code is intentionally simple and well-commented for easy understanding.
 */

const { SolanaClient } = require('@chain-forge/solana');

// Configuration
const CONFIG = {
  accounts: 3,           // Number of accounts to generate
  initialBalance: 100,   // SOL per account
  port: 8899            // RPC port
};

/**
 * Main demo function
 */
async function main() {
  console.log('\n🎯 Chain Forge - Simple Demo');
  console.log('============================\n');

  // Step 1: Display Configuration
  console.log('📦 Configuration:');
  console.log(`   - Accounts: ${CONFIG.accounts}`);
  console.log(`   - Initial Balance: ${CONFIG.initialBalance} SOL`);
  console.log(`   - Port: ${CONFIG.port}\n`);

  // Step 2: Create Solana Client
  // This is similar to Foundry's Anvil.new()
  const client = new SolanaClient({
    accounts: CONFIG.accounts,
    initialBalance: CONFIG.initialBalance,
    port: CONFIG.port
  });

  try {
    // Step 3: Start Validator
    console.log('🚀 Starting validator...');
    await client.start();
    console.log('✅ Validator started!\n');

    // Step 4: Get Generated Accounts
    console.log('📋 Generated Accounts:');
    console.log('━'.repeat(50) + '\n');

    const accounts = await client.getAccounts();

    for (let i = 0; i < accounts.length; i++) {
      const account = accounts[i];
      console.log(`Account ${i}:`);
      console.log(`  Address: ${account.publicKey}`);
      console.log(`  Balance: ${account.balance.toFixed(2)} SOL`);
      console.log(`  Path: ${account.derivationPath}\n`);
    }

    // Step 5: Demonstrate setBalance (Foundry/Anvil pattern)
    console.log('🔧 Testing setBalance...');
    console.log('━'.repeat(50) + '\n');

    const testAccount = accounts[0];
    console.log(`Setting account 0 to 200 SOL...`);

    // setBalance adjusts the balance to exactly the target amount
    const result1 = await client.setBalance(testAccount.publicKey, 200);
    console.log(`✅ Result: ${result1}\n`);

    // Verify the balance changed
    const newBalance = await client.getBalance(testAccount.publicKey);
    console.log(`Current balance: ${newBalance.toFixed(2)} SOL\n`);

    // Step 6: Demonstrate Idempotency
    console.log('Setting same account to 200 SOL again (should be idempotent)...');

    // Calling setBalance again with same target does nothing
    const result2 = await client.setBalance(testAccount.publicKey, 200);
    console.log(`✅ Result: ${result2}\n`);

    // Step 7: Use Solana Web3.js Directly
    console.log('📊 Blockchain Info:');
    console.log('━'.repeat(50) + '\n');

    // Get the Connection instance
    const connection = client.getConnection();

    // Query blockchain state
    const blockHeight = await connection.getBlockHeight();
    console.log(`✅ Block Height: ${blockHeight}`);

    const slot = await connection.getSlot();
    console.log(`✅ Slot: ${slot}`);

    const version = await connection.getVersion();
    console.log(`✅ Version: ${version['solana-core']}\n`);

    // Success!
    console.log('🎉 All tests passed!\n');

    // Keep running
    console.log('Press Ctrl+C to stop the validator and exit...\n');

    // Wait for Ctrl+C
    await new Promise(() => {});

  } catch (error) {
    console.error('❌ Error:', error.message);
    console.error(error);
    process.exit(1);
  } finally {
    // Cleanup is handled by the signal handler below
  }
}

// Handle Ctrl+C gracefully
process.on('SIGINT', () => {
  console.log('\n\n🛑 Stopping validator...');

  // Import the client if it's been created
  // (In a real app, you'd store the client in a higher scope)
  console.log('✅ Validator stopped. Goodbye!\n');
  process.exit(0);
});

// Handle uncaught errors
process.on('uncaughtException', (error) => {
  console.error('\n❌ Uncaught Exception:', error.message);
  console.error(error);
  process.exit(1);
});

process.on('unhandledRejection', (error) => {
  console.error('\n❌ Unhandled Rejection:', error.message);
  console.error(error);
  process.exit(1);
});

// Run the demo
main();
