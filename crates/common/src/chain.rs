use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Core trait that all blockchain implementations must implement
pub trait ChainProvider {
    /// Chain-specific account type
    type Account: Serialize + for<'de> Deserialize<'de>;

    /// Chain-specific transaction type
    type Transaction;

    /// Chain-specific configuration
    type Config;

    /// Start the local chain validator/node
    fn start(&mut self, config: Self::Config) -> Result<()>;

    /// Stop the running chain validator/node
    fn stop(&mut self) -> Result<()>;

    /// Check if the chain is currently running
    fn is_running(&self) -> bool;

    /// Get all generated accounts
    fn get_accounts(&self) -> Result<Vec<Self::Account>>;

    /// Set an account's balance to a specific amount
    ///
    /// This is the preferred method for managing account balances.
    /// Implementations should adjust the balance to match the target amount.
    ///
    /// Note: Some chains (like Solana) can only add funds, not reduce them.
    fn set_balance(&self, address: &str, amount: f64) -> Result<String>;

    /// Fund an account with native tokens (adds to existing balance)
    ///
    /// Deprecated: Use set_balance instead for more predictable behavior.
    #[deprecated(note = "Use set_balance instead")]
    fn fund_account(&self, address: &str, amount: f64) -> Result<String> {
        // Default implementation delegates to set_balance
        self.set_balance(address, amount)
    }

    /// Get the balance of an account
    fn get_balance(&self, address: &str) -> Result<f64>;

    /// Get the RPC URL for connecting to the chain
    fn get_rpc_url(&self) -> String;
}

/// Common account interface
pub trait Account {
    /// Get the public key/address as a string
    fn address(&self) -> String;

    /// Get the private key/secret (if available)
    fn secret(&self) -> Option<String>;

    /// Get the mnemonic phrase (if derived from one)
    fn mnemonic(&self) -> Option<String>;

    /// Get the derivation path (if applicable)
    fn derivation_path(&self) -> Option<String>;
}
