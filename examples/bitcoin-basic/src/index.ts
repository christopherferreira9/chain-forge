import { BitcoinClient } from '@chain-forge/bitcoin';

async function main() {
  console.log('Chain Forge - Bitcoin Basic Example');
  console.log('====================================\n');

  // Create client with 3 accounts, 10 BTC each
  const client = new BitcoinClient({
    accounts: 3,
    initialBalance: 10,
    rpcPort: 18443,
  });

  try {
    console.log('Starting Bitcoin regtest node...');
    console.log('(This may take a moment as initial blocks are mined)\n');

    await client.start();

    console.log('Node started successfully!\n');

    // Get and display accounts
    const accounts = await client.getAccounts();
    console.log(`Generated ${accounts.length} accounts:`);
    console.log('─'.repeat(80));

    for (let i = 0; i < accounts.length; i++) {
      const account = accounts[i];
      console.log(`[${i}] ${account.address}`);
      console.log(`    Balance: ${account.balance.toFixed(8)} BTC`);
      console.log(`    WIF: ${account.wif.substring(0, 20)}...`);
    }

    console.log('─'.repeat(80));
    console.log();

    // Send 1 BTC from the first account's holdings to account 1
    console.log('Sending 1 BTC from wallet to account 1...');
    // const sendResult = await client.sendToAddress(accounts[1].address, 1);
    const sendResult = await client.transfer(accounts[0].address, accounts[1].address, 1);
    console.log(`Transaction sent: ${sendResult.txid.substring(0, 32)}...\n`);

    // Mine a block to confirm the transaction
    console.log('Mining block to confirm transaction...');
    const mineResult = await client.mine(1);
    console.log(`Block mined: ${mineResult.blockHashes[0]?.substring(0, 32)}...`);
    if (mineResult.height) {
      console.log(`Current height: ${mineResult.height}`);
    }
    console.log();

    // Refresh balances from the blockchain
    const updatedAccounts = await client.refreshBalances();
    console.log('Updated balances (from blockchain):');
    console.log('─'.repeat(80));

    for (let i = 0; i < updatedAccounts.length; i++) {
      const account = updatedAccounts[i];
      console.log(`[${i}] ${account.address.substring(0, 40)}...`);
      console.log(`    Balance: ${account.balance.toFixed(8)} BTC`);
    }

    console.log('─'.repeat(80));
    console.log();

    console.log('Example completed successfully!');

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  } finally {
    console.log('\nStopping Bitcoin node...');
    client.stop();
    console.log('Done.');
  }
}

main().catch(console.error);
