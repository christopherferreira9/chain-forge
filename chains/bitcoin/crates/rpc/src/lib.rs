use bitcoincore_rpc::bitcoin::address::NetworkUnchecked;
use bitcoincore_rpc::bitcoin::{Address, Amount};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use chain_forge_bitcoin_accounts::BitcoinAccount;
use chain_forge_common::{ChainError, Result};
use std::time::Duration;

/// Bitcoin Core RPC client wrapper
pub struct BitcoinRpcClient {
    client: Client,
    rpc_url: String,
    wallet_name: String,
}

impl BitcoinRpcClient {
    /// Create a new RPC client with authentication
    pub fn new(rpc_url: String, user: String, password: String) -> Result<Self> {
        let auth = Auth::UserPass(user, password);
        let client = Client::new(&rpc_url, auth)
            .map_err(|e| ChainError::Rpc(format!("Failed to create RPC client: {}", e)))?;

        Ok(Self {
            client,
            rpc_url,
            wallet_name: "chain-forge".to_string(),
        })
    }

    /// Create a new RPC client connected to a specific wallet
    pub fn new_with_wallet(
        rpc_url: String,
        user: String,
        password: String,
        wallet_name: &str,
    ) -> Result<Self> {
        let auth = Auth::UserPass(user, password);
        let wallet_url = format!("{}/wallet/{}", rpc_url, wallet_name);
        let client = Client::new(&wallet_url, auth)
            .map_err(|e| ChainError::Rpc(format!("Failed to create wallet RPC client: {}", e)))?;

        Ok(Self {
            client,
            rpc_url,
            wallet_name: wallet_name.to_string(),
        })
    }

    /// Get the RPC URL
    pub fn url(&self) -> &str {
        &self.rpc_url
    }

    /// Get the wallet name
    pub fn wallet_name(&self) -> &str {
        &self.wallet_name
    }

    /// Check if the node is running by trying to get blockchain info
    pub fn is_node_running(&self) -> bool {
        self.client.get_blockchain_info().is_ok()
    }

    /// Wait for the node to be ready
    pub async fn wait_for_node(&self, max_attempts: u32) -> Result<()> {
        for attempt in 1..=max_attempts {
            if self.is_node_running() {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(500)).await;

            if attempt == max_attempts {
                return Err(ChainError::NodeManagement(
                    "Bitcoin node did not start in time".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Create a new wallet (descriptor wallet with default keys for change addresses)
    pub fn create_wallet(&self, wallet_name: &str) -> Result<()> {
        // Check if wallet already exists
        let wallets = self
            .client
            .list_wallets()
            .map_err(|e| ChainError::Rpc(format!("Failed to list wallets: {}", e)))?;

        if wallets.contains(&wallet_name.to_string()) {
            return Ok(());
        }

        // Try to load existing wallet first
        match self.client.load_wallet(wallet_name) {
            Ok(_) => return Ok(()),
            Err(_) => {
                // Wallet doesn't exist, create a descriptor wallet with default keys
                // We need the default keys for change addresses when sending
                self.client
                    .create_wallet(wallet_name, None, None, None, None)
                    .map_err(|e| ChainError::Rpc(format!("Failed to create wallet: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Import an account with its private key for spending (uses importdescriptors)
    ///
    /// For P2WPKH (bech32) addresses, uses wpkh(WIF) descriptor format.
    pub fn import_address(&self, address: &str, wif: &str, label: &str) -> Result<()> {
        // For descriptor wallets with P2WPKH addresses, use wpkh(WIF) descriptor
        let raw_desc = format!("wpkh({})", wif);

        // Get the checksum for the descriptor
        let desc_info: serde_json::Value = self
            .client
            .call("getdescriptorinfo", &[serde_json::json!(raw_desc)])
            .map_err(|e| ChainError::Rpc(format!("Failed to get descriptor info: {}", e)))?;

        let checksum = desc_info["checksum"]
            .as_str()
            .ok_or_else(|| ChainError::Rpc("Invalid descriptor info response".to_string()))?;

        // Create the full descriptor with checksum (keeping private key)
        let descriptor_with_checksum = format!("{}#{}", raw_desc, checksum);

        // Import the descriptor - use "now" timestamp since blockchain is fresh
        // and we'll be sending to these addresses AFTER import
        let import_request = serde_json::json!([{
            "desc": descriptor_with_checksum,
            "timestamp": "now",
            "label": label
        }]);

        let result: serde_json::Value = self
            .client
            .call("importdescriptors", &[import_request])
            .map_err(|e| ChainError::Rpc(format!("Failed to import descriptor: {}", e)))?;

        // Check if import was successful
        if let Some(arr) = result.as_array() {
            if let Some(first) = arr.first() {
                if first["success"].as_bool() != Some(true) {
                    let error = first["error"]["message"]
                        .as_str()
                        .unwrap_or("Unknown error");

                    // Treat "Rescan failed" as a warning for fresh addresses
                    // The descriptor is still imported, it just couldn't verify no pre-existing transactions
                    if error.contains("Rescan failed") {
                        // This is expected for fresh addresses and can be safely ignored
                        // The descriptor is imported and will track future transactions
                    } else {
                        return Err(ChainError::Rpc(format!(
                            "Failed to import address {}: {}",
                            address, error
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the balance of a specific address
    /// Uses scantxoutset for direct UTXO query without relying on wallet state
    pub fn get_balance(&self, address: &str) -> Result<f64> {
        // Use scantxoutset to directly query the UTXO set for this address
        // This doesn't rely on wallet descriptor tracking
        let scan_result: serde_json::Value = self
            .client
            .call(
                "scantxoutset",
                &[
                    serde_json::json!("start"),
                    serde_json::json!([format!("addr({})", address)]),
                ],
            )
            .map_err(|e| ChainError::Rpc(format!("Failed to scan UTXO set: {}", e)))?;

        // Extract total amount from scan result
        let total = scan_result["total_amount"].as_f64().unwrap_or(0.0);

        Ok(total)
    }

    /// Get the total wallet balance
    pub fn get_wallet_balance(&self) -> Result<f64> {
        let balance = self
            .client
            .get_balance(None, None)
            .map_err(|e| ChainError::Rpc(format!("Failed to get wallet balance: {}", e)))?;

        Ok(balance.to_btc())
    }

    /// Mine blocks to a specific address
    pub fn mine_blocks(&self, count: u32, address: &str) -> Result<Vec<String>> {
        let addr: Address<NetworkUnchecked> = address
            .parse()
            .map_err(|e| ChainError::Rpc(format!("Invalid address: {}", e)))?;

        let addr = addr.assume_checked();

        let block_hashes = self
            .client
            .generate_to_address(count as u64, &addr)
            .map_err(|e| ChainError::Rpc(format!("Failed to mine blocks: {}", e)))?;

        Ok(block_hashes.iter().map(|h| h.to_string()).collect())
    }

    /// Send BTC to an address (from wallet funds, not a specific account)
    ///
    /// This sends from the wallet's available UTXOs. For sending from a specific
    /// account, use `send_from_address` instead.
    pub fn send_to_address(&self, address: &str, amount_btc: f64) -> Result<String> {
        let addr: Address<NetworkUnchecked> = address
            .parse()
            .map_err(|e| ChainError::Rpc(format!("Invalid address: {}", e)))?;

        let addr = addr.assume_checked();

        let amount = Amount::from_btc(amount_btc)
            .map_err(|e| ChainError::Rpc(format!("Invalid amount: {}", e)))?;

        let txid = self
            .client
            .send_to_address(&addr, amount, None, None, None, None, None, None)
            .map_err(|e| ChainError::Rpc(format!("Failed to send transaction: {}", e)))?;

        Ok(txid.to_string())
    }

    /// Send BTC from a specific address to another address
    ///
    /// This creates a transaction that specifically spends UTXOs owned by `from_address`.
    /// The source address must be imported into the wallet with its private key.
    pub fn send_from_address(
        &self,
        from_address: &str,
        to_address: &str,
        amount_btc: f64,
    ) -> Result<String> {
        // Get UTXOs for the source address
        let utxos: Vec<serde_json::Value> = self
            .client
            .call(
                "listunspent",
                &[
                    serde_json::json!(1),              // minconf
                    serde_json::json!(9999999),        // maxconf
                    serde_json::json!([from_address]), // addresses to filter
                ],
            )
            .map_err(|e| ChainError::Rpc(format!("Failed to list UTXOs: {}", e)))?;

        if utxos.is_empty() {
            return Err(ChainError::Rpc(format!(
                "No UTXOs found for address {}",
                from_address
            )));
        }

        // Calculate total available
        let total_available: f64 = utxos
            .iter()
            .map(|u| u["amount"].as_f64().unwrap_or(0.0))
            .sum();

        // Estimate fee (simple: 0.0001 BTC per KB, ~250 bytes per input)
        let estimated_fee = 0.0001 * (utxos.len() as f64 * 0.25).max(0.25);
        let amount_with_fee = amount_btc + estimated_fee;

        if total_available < amount_with_fee {
            return Err(ChainError::Rpc(format!(
                "Insufficient funds in {}: {} BTC available, {} BTC needed (including ~{} BTC fee)",
                &from_address[..20],
                total_available,
                amount_with_fee,
                estimated_fee
            )));
        }

        // Select UTXOs (simple: use all until we have enough)
        let mut selected_utxos = Vec::new();
        let mut selected_total = 0.0;
        for utxo in &utxos {
            selected_utxos.push(serde_json::json!({
                "txid": utxo["txid"],
                "vout": utxo["vout"],
            }));
            selected_total += utxo["amount"].as_f64().unwrap_or(0.0);
            if selected_total >= amount_with_fee {
                break;
            }
        }

        // Calculate change
        let change = selected_total - amount_btc - estimated_fee;

        // Build outputs
        let mut outputs = serde_json::Map::new();
        outputs.insert(to_address.to_string(), serde_json::json!(amount_btc));
        if change > 0.00001 {
            // Send change back to source
            outputs.insert(from_address.to_string(), serde_json::json!(change));
        }

        // Create raw transaction
        let raw_tx: String = self
            .client
            .call(
                "createrawtransaction",
                &[
                    serde_json::json!(selected_utxos),
                    serde_json::json!(outputs),
                ],
            )
            .map_err(|e| ChainError::Rpc(format!("Failed to create raw transaction: {}", e)))?;

        // Sign the transaction
        let signed: serde_json::Value = self
            .client
            .call("signrawtransactionwithwallet", &[serde_json::json!(raw_tx)])
            .map_err(|e| ChainError::Rpc(format!("Failed to sign transaction: {}", e)))?;

        if signed["complete"].as_bool() != Some(true) {
            return Err(ChainError::Rpc(format!(
                "Transaction signing incomplete: {:?}",
                signed["errors"]
            )));
        }

        let signed_hex = signed["hex"]
            .as_str()
            .ok_or_else(|| ChainError::Rpc("No signed transaction hex".to_string()))?;

        // Broadcast the transaction
        let txid: String = self
            .client
            .call("sendrawtransaction", &[serde_json::json!(signed_hex)])
            .map_err(|e| ChainError::Rpc(format!("Failed to broadcast transaction: {}", e)))?;

        Ok(txid)
    }

    /// Set the balance of an address to a target amount
    ///
    /// In Bitcoin regtest, this works by:
    /// 1. Getting current balance
    /// 2. If below target, sending the difference from the wallet
    ///
    /// Note: Unlike Solana's airdrop, this requires the wallet to have funds
    pub fn set_balance(&self, address: &str, target_btc: f64) -> Result<String> {
        let current = self.get_balance(address)?;

        if current >= target_btc {
            return Ok(format!(
                "Balance already at {} BTC (target: {} BTC)",
                current, target_btc
            ));
        }

        let diff = target_btc - current;

        // Send the difference
        let txid = self.send_to_address(address, diff)?;

        Ok(format!(
            "Added {} BTC ({} -> {} BTC). TxID: {}",
            diff, current, target_btc, txid
        ))
    }

    /// Fund multiple accounts with their target balances
    ///
    /// This is a simplified version that:
    /// 1. Sends the exact target amount to each account (assuming they start at 0)
    /// 2. Does NOT mine blocks (caller should mine for confirmation after all sends)
    ///
    /// Returns an error if any account fails to fund.
    pub async fn fund_accounts(&self, accounts: &mut [BitcoinAccount]) -> Result<()> {
        let mut errors = Vec::new();

        for (i, account) in accounts.iter_mut().enumerate() {
            let target_balance = account.balance;

            // Skip if target is 0
            if target_balance <= 0.0 {
                continue;
            }

            // Send the target amount directly
            match self.send_to_address(&account.address, target_balance) {
                Ok(txid) => {
                    println!(
                        "   Sent {} BTC to account {} {} (txid: {}...)",
                        target_balance,
                        i,
                        &account.address[..20],
                        &txid[..16]
                    );
                }
                Err(e) => {
                    errors.push(format!("account {}: {}", i, e));
                    // Set balance to 0 since funding failed
                    account.balance = 0.0;
                }
            }

            // Small delay between transactions to allow UTXO set to update
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        if !errors.is_empty() {
            return Err(ChainError::Rpc(format!(
                "Failed to fund {} account(s): {}",
                errors.len(),
                errors.join("; ")
            )));
        }

        Ok(())
    }

    /// Update account balances from the blockchain
    ///
    /// Returns an error if any balance queries fail, listing which accounts failed.
    pub fn update_balances(&self, accounts: &mut [BitcoinAccount]) -> Result<()> {
        let mut errors = Vec::new();

        for account in accounts.iter_mut() {
            match self.get_balance(&account.address) {
                Ok(balance) => {
                    account.balance = balance;
                }
                Err(e) => {
                    errors.push(format!("{}...: {}", &account.address[..20], e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(ChainError::Rpc(format!(
                "Failed to get balance for {} account(s): {}",
                errors.len(),
                errors.join(", ")
            )));
        }

        Ok(())
    }

    /// Get blockchain info
    pub fn get_blockchain_info(&self) -> Result<bitcoincore_rpc::json::GetBlockchainInfoResult> {
        self.client
            .get_blockchain_info()
            .map_err(|e| ChainError::Rpc(format!("Failed to get blockchain info: {}", e)))
    }

    /// Get the current block count
    pub fn get_block_count(&self) -> Result<u64> {
        self.client
            .get_block_count()
            .map_err(|e| ChainError::Rpc(format!("Failed to get block count: {}", e)))
    }

    /// Get a new address from the wallet for receiving funds
    /// This is useful for mining rewards where we don't want to use user accounts
    pub fn get_new_address(&self, label: Option<&str>) -> Result<String> {
        let label = label.unwrap_or("mining");
        let address: String = self
            .client
            .call(
                "getnewaddress",
                &[serde_json::json!(label), serde_json::json!("bech32")],
            )
            .map_err(|e| ChainError::Rpc(format!("Failed to get new address: {}", e)))?;

        Ok(address)
    }

    /// Get the inner RPC client for advanced operations
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_client_url() {
        let client = BitcoinRpcClient::new(
            "http://localhost:18443".to_string(),
            "user".to_string(),
            "pass".to_string(),
        )
        .unwrap();
        assert_eq!(client.url(), "http://localhost:18443");
    }

    #[test]
    fn test_wallet_name() {
        let client = BitcoinRpcClient::new(
            "http://localhost:18443".to_string(),
            "user".to_string(),
            "pass".to_string(),
        )
        .unwrap();
        assert_eq!(client.wallet_name(), "chain-forge");
    }

    #[test]
    fn test_node_running_check_no_server() {
        let client = BitcoinRpcClient::new(
            "http://localhost:19999".to_string(),
            "user".to_string(),
            "pass".to_string(),
        )
        .unwrap();
        // Should return false when no node is running
        assert!(!client.is_node_running());
    }
}
