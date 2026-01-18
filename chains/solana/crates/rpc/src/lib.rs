use chain_forge_common::{ChainError, Result};
use chain_forge_solana_accounts::SolanaAccount;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
};
use std::str::FromStr;
use std::time::Duration;

/// Wrapper around Solana RPC client
pub struct SolanaRpcClient {
    client: RpcClient,
    rpc_url: String,
}

impl SolanaRpcClient {
    /// Create a new RPC client
    pub fn new(rpc_url: String) -> Self {
        let client = RpcClient::new_with_timeout_and_commitment(
            rpc_url.clone(),
            Duration::from_secs(30),
            CommitmentConfig::confirmed(),
        );

        Self { client, rpc_url }
    }

    /// Get the RPC URL
    pub fn url(&self) -> &str {
        &self.rpc_url
    }

    /// Check if the validator is running by trying to get the version
    pub fn is_validator_running(&self) -> bool {
        self.client.get_version().is_ok()
    }

    /// Wait for the validator to be ready
    pub async fn wait_for_validator(&self, max_attempts: u32) -> Result<()> {
        for attempt in 1..=max_attempts {
            if self.is_validator_running() {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(500)).await;

            if attempt == max_attempts {
                return Err(ChainError::NodeManagement(
                    "Validator did not start in time".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the balance of an account in SOL
    pub fn get_balance(&self, address: &str) -> Result<f64> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| ChainError::Rpc(format!("Invalid public key: {}", e)))?;

        let lamports = self
            .client
            .get_balance(&pubkey)
            .map_err(|e| ChainError::Rpc(format!("Failed to get balance: {}", e)))?;

        Ok(lamports as f64 / LAMPORTS_PER_SOL as f64)
    }

    /// Set the balance of an account to a specific amount
    ///
    /// This adjusts the account balance to match the target amount.
    /// If current balance is less than target, requests an airdrop for the difference.
    /// If current balance is already >= target, does nothing.
    ///
    /// Note: On Solana test validators, we can only ADD funds via airdrops,
    /// we cannot reduce balances. This is different from Ethereum's Anvil
    /// which can set exact balances.
    pub fn set_balance(&self, address: &str, target_sol: f64) -> Result<String> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| ChainError::Rpc(format!("Invalid public key: {}", e)))?;

        // Get current balance
        let current_lamports = self
            .client
            .get_balance(&pubkey)
            .map_err(|e| ChainError::Rpc(format!("Failed to get balance: {}", e)))?;

        let current_sol = current_lamports as f64 / LAMPORTS_PER_SOL as f64;
        let target_lamports = (target_sol * LAMPORTS_PER_SOL as f64) as u64;

        // If already at or above target, no action needed
        if current_lamports >= target_lamports {
            return Ok(format!(
                "Balance already at {} SOL (target: {} SOL)",
                current_sol, target_sol
            ));
        }

        // Calculate difference and request airdrop
        let diff_lamports = target_lamports - current_lamports;
        let diff_sol = diff_lamports as f64 / LAMPORTS_PER_SOL as f64;

        let signature = self
            .client
            .request_airdrop(&pubkey, diff_lamports)
            .map_err(|e| ChainError::Rpc(format!("Airdrop request failed: {}", e)))?;

        // Wait for confirmation
        self.client
            .confirm_transaction(&signature)
            .map_err(|e| ChainError::Rpc(format!("Failed to confirm airdrop: {}", e)))?;

        Ok(format!(
            "Added {} SOL ({}  → {} SOL). Signature: {}",
            diff_sol, current_sol, target_sol, signature
        ))
    }

    /// Request an airdrop to an account (adds to existing balance)
    pub fn request_airdrop(&self, address: &str, amount_sol: f64) -> Result<String> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| ChainError::Rpc(format!("Invalid public key: {}", e)))?;

        let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

        let signature = self
            .client
            .request_airdrop(&pubkey, lamports)
            .map_err(|e| ChainError::Rpc(format!("Airdrop request failed: {}", e)))?;

        // Wait for confirmation
        self.client
            .confirm_transaction(&signature)
            .map_err(|e| ChainError::Rpc(format!("Failed to confirm airdrop: {}", e)))?;

        Ok(signature.to_string())
    }

    /// Set balances for multiple accounts to target amounts
    ///
    /// This is the primary method for ensuring accounts have specific balances.
    /// Similar to Foundry/Anvil where you specify target balances upfront.
    pub async fn set_balances(&self, accounts: &mut [SolanaAccount]) -> Result<()> {
        for account in accounts.iter_mut() {
            let target_balance = account.balance;

            // Retry logic for rate-limited airdrops
            let mut retries = 3;
            let mut success = false;

            while retries > 0 && !success {
                match self.set_balance(&account.public_key, target_balance) {
                    Ok(msg) => {
                        // Balance is now set to target
                        if msg.contains("Already") {
                            // Balance was already sufficient
                        } else {
                            // Balance was adjusted
                            account.balance = target_balance;
                        }
                        success = true;
                    }
                    Err(e) => {
                        if retries > 1 {
                            eprintln!(
                                "Warning: Failed to set balance for {}: {}. Retrying in 2s...",
                                account.public_key, e
                            );
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        } else {
                            eprintln!(
                                "Warning: Failed to set balance for {} after 3 attempts: {}",
                                account.public_key, e
                            );
                        }
                        retries -= 1;
                    }
                }
            }

            // Longer delay between accounts to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Update all balances to reflect actual state
        self.update_balances(accounts)?;

        Ok(())
    }

    /// Fund multiple accounts (legacy method, use set_balances instead)
    #[deprecated(note = "Use set_balances instead for more consistent behavior")]
    pub async fn fund_accounts(
        &self,
        accounts: &mut [SolanaAccount],
        amount_sol: f64,
    ) -> Result<()> {
        for account in accounts.iter_mut() {
            match self.request_airdrop(&account.public_key, amount_sol) {
                Ok(_) => {
                    account.balance = amount_sol;
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to fund account {}: {}",
                        account.public_key, e
                    );
                }
            }
            // Small delay between airdrops to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Update account balances
    pub fn update_balances(&self, accounts: &mut [SolanaAccount]) -> Result<()> {
        for account in accounts.iter_mut() {
            match self.get_balance(&account.public_key) {
                Ok(balance) => {
                    account.balance = balance;
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to get balance for {}: {}",
                        account.public_key, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Get the latest blockhash
    pub fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.client
            .get_latest_blockhash()
            .map_err(|e| ChainError::Rpc(format!("Failed to get latest blockhash: {}", e)))
    }

    /// Get cluster version
    pub fn get_version(&self) -> Result<String> {
        let version = self
            .client
            .get_version()
            .map_err(|e| ChainError::Rpc(format!("Failed to get version: {}", e)))?;

        Ok(version.solana_core)
    }

    /// Get the inner RPC client for advanced operations
    pub fn inner(&self) -> &RpcClient {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_client_creation() {
        let client = SolanaRpcClient::new("http://localhost:8899".to_string());
        assert_eq!(client.url(), "http://localhost:8899");
    }

    #[test]
    fn test_rpc_client_url() {
        let client = SolanaRpcClient::new("http://example.com:9000".to_string());
        assert_eq!(client.url(), "http://example.com:9000");
    }

    #[test]
    fn test_rpc_client_inner() {
        let client = SolanaRpcClient::new("http://localhost:8899".to_string());
        let _inner = client.inner();
        // Just verify we can get the inner client
    }

    #[test]
    fn test_validator_running_check_no_server() {
        let client = SolanaRpcClient::new("http://localhost:19999".to_string());
        // Should return false when no validator is running
        assert!(!client.is_validator_running());
    }
}
