use chain_forge_common::{ChainError, ChainProvider, Result};
use chain_forge_config::{Config, SolanaProfile};
use chain_forge_solana_accounts::{AccountGenerator, AccountsStorage, SolanaAccount};
use chain_forge_solana_rpc::SolanaRpcClient;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

/// Configuration for starting a Solana validator
#[derive(Debug, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub port: u16,
    pub accounts: u32,
    pub initial_balance: f64,
    pub mnemonic: Option<String>,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(),
            port: 8899,
            accounts: 10,
            initial_balance: 100.0,
            mnemonic: None,
        }
    }
}

impl From<SolanaProfile> for SolanaConfig {
    fn from(profile: SolanaProfile) -> Self {
        Self {
            rpc_url: profile.rpc_url,
            port: profile.port,
            accounts: profile.accounts,
            initial_balance: profile.initial_balance,
            mnemonic: None,
        }
    }
}

/// Solana chain provider implementation
pub struct SolanaProvider {
    config: SolanaConfig,
    rpc_client: Option<SolanaRpcClient>,
    accounts: Vec<SolanaAccount>,
    validator_process: Arc<Mutex<Option<Child>>>,
    storage: AccountsStorage,
}

impl SolanaProvider {
    /// Create a new Solana provider
    pub fn new() -> Self {
        let data_dir = Config::ensure_data_dir().expect("Failed to create data directory");
        let storage = AccountsStorage::new(&data_dir);

        Self {
            config: SolanaConfig::default(),
            rpc_client: None,
            accounts: Vec::new(),
            validator_process: Arc::new(Mutex::new(None)),
            storage,
        }
    }

    /// Create a provider with a specific configuration
    pub fn with_config(config: SolanaConfig) -> Self {
        let data_dir = Config::ensure_data_dir().expect("Failed to create data directory");
        let storage = AccountsStorage::new(&data_dir);

        Self {
            config,
            rpc_client: None,
            accounts: Vec::new(),
            validator_process: Arc::new(Mutex::new(None)),
            storage,
        }
    }

    /// Load or generate accounts
    fn load_or_generate_accounts(&mut self) -> Result<()> {
        // Try to load existing accounts
        if self.storage.exists() {
            self.accounts = self.storage.load()?;
            if self.accounts.len() >= self.config.accounts as usize {
                return Ok(());
            }
        }

        // Generate new accounts
        let generator = if let Some(mnemonic) = &self.config.mnemonic {
            AccountGenerator::from_mnemonic(mnemonic)?
        } else {
            AccountGenerator::new()?
        };

        println!("🔑 Mnemonic: {}", generator.mnemonic_phrase());
        println!("   Save this mnemonic to recover your accounts!");
        println!();

        self.accounts = generator.generate_accounts(self.config.accounts)?;
        self.storage.save(&self.accounts)?;

        Ok(())
    }

    /// Start the validator process
    fn start_validator(&mut self) -> Result<()> {
        // Check if solana-test-validator is available
        let validator_check = Command::new("solana-test-validator")
            .arg("--version")
            .output();

        if validator_check.is_err() {
            return Err(ChainError::NodeManagement(
                "solana-test-validator not found. Please install Solana CLI tools.".to_string(),
            ));
        }

        println!(
            "🚀 Starting Solana test validator on port {}...",
            self.config.port
        );

        // Start the validator
        let mut cmd = Command::new("solana-test-validator");
        cmd.arg("--rpc-port")
            .arg(self.config.port.to_string())
            .arg("--faucet-port")
            .arg("9901") // Use port 9901 for faucet (avoid default 9900 conflicts)
            .arg("--quiet")
            .arg("--reset");

        let child = cmd
            .spawn()
            .map_err(|e| ChainError::NodeManagement(format!("Failed to start validator: {}", e)))?;

        let mut process_guard = self.validator_process.lock().unwrap();
        *process_guard = Some(child);

        Ok(())
    }

    /// Get a reference to the RPC client
    pub fn rpc_client(&self) -> Result<&SolanaRpcClient> {
        self.rpc_client.as_ref().ok_or(ChainError::NotRunning)
    }
}

impl Default for SolanaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainProvider for SolanaProvider {
    type Account = SolanaAccount;
    type Transaction = ();
    type Config = SolanaConfig;

    fn start(&mut self, config: Self::Config) -> Result<()> {
        if self.is_running() {
            return Err(ChainError::AlreadyRunning);
        }

        self.config = config;

        // Load or generate accounts
        self.load_or_generate_accounts()?;

        // Start validator
        self.start_validator()?;

        // Initialize using spawn_blocking to avoid nested runtime issues
        let data_dir = Config::ensure_data_dir()?;
        let result = std::thread::spawn({
            let config_url = self.config.rpc_url.clone();
            let accounts = self.config.accounts;
            let initial_balance = self.config.initial_balance;
            let mut accounts_vec = self.accounts.clone();

            move || {
                // Create a new runtime in this thread
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| ChainError::Other(format!("Failed to create runtime: {}", e)))?;

                rt.block_on(async {
                    let rpc_client = SolanaRpcClient::new(config_url);

                    println!("⏳ Waiting for validator to be ready...");
                    rpc_client.wait_for_validator(60).await?;

                    println!("✅ Validator is ready!");
                    println!();

                    // Initialize account balances to the target amount before funding
                    for account in accounts_vec.iter_mut() {
                        account.balance = initial_balance;
                    }

                    println!(
                        "💰 Setting {} accounts to {} SOL each...",
                        accounts, initial_balance
                    );
                    rpc_client.set_balances(&mut accounts_vec).await?;
                    rpc_client.update_balances(&mut accounts_vec)?;

                    // Save updated accounts
                    let storage = AccountsStorage::new(&data_dir);
                    storage.save(&accounts_vec)?;

                    println!("✅ All accounts funded!");
                    println!();

                    Ok::<(SolanaRpcClient, Vec<SolanaAccount>), ChainError>((
                        rpc_client,
                        accounts_vec,
                    ))
                })
            }
        })
        .join()
        .map_err(|_| ChainError::Other("Initialization thread panicked".to_string()))??;

        self.rpc_client = Some(result.0);
        self.accounts = result.1;

        println!("🎉 Solana test validator is running!");
        println!("   RPC URL: {}", self.config.rpc_url);
        println!();

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let mut process_guard = self.validator_process.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            child.kill().map_err(|e| {
                ChainError::NodeManagement(format!("Failed to stop validator: {}", e))
            })?;

            child.wait().map_err(|e| {
                ChainError::NodeManagement(format!("Failed to wait for validator: {}", e))
            })?;

            println!("🛑 Solana test validator stopped");
        }

        self.rpc_client = None;

        Ok(())
    }

    fn is_running(&self) -> bool {
        let process_guard = self.validator_process.lock().unwrap();
        process_guard.is_some()
    }

    fn get_accounts(&self) -> Result<Vec<Self::Account>> {
        Ok(self.accounts.clone())
    }

    fn set_balance(&self, address: &str, amount: f64) -> Result<String> {
        let client = self.rpc_client()?;
        client.set_balance(address, amount)
    }

    fn fund_account(&self, address: &str, amount: f64) -> Result<String> {
        let client = self.rpc_client()?;
        client.request_airdrop(address, amount)
    }

    fn get_balance(&self, address: &str) -> Result<f64> {
        let client = self.rpc_client()?;
        client.get_balance(address)
    }

    fn get_rpc_url(&self) -> String {
        self.config.rpc_url.clone()
    }
}

impl Drop for SolanaProvider {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = SolanaProvider::new();
        assert!(!provider.is_running());
    }

    #[test]
    fn test_config_defaults() {
        let config = SolanaConfig::default();
        assert_eq!(config.port, 8899);
        assert_eq!(config.accounts, 10);
        assert_eq!(config.initial_balance, 100.0);
        assert_eq!(config.rpc_url, "http://localhost:8899");
        assert!(config.mnemonic.is_none());
    }

    #[test]
    fn test_provider_with_config() {
        let config = SolanaConfig {
            rpc_url: "http://localhost:9000".to_string(),
            port: 9000,
            accounts: 5,
            initial_balance: 50.0,
            mnemonic: None,
        };

        let provider = SolanaProvider::with_config(config.clone());
        assert!(!provider.is_running());
        assert_eq!(provider.get_rpc_url(), "http://localhost:9000");
    }

    #[test]
    fn test_config_from_profile() {
        use chain_forge_config::SolanaProfile;

        let profile = SolanaProfile {
            rpc_url: "http://localhost:8900".to_string(),
            accounts: 15,
            initial_balance: 200.0,
            port: 8900,
        };

        let config: SolanaConfig = profile.into();
        assert_eq!(config.rpc_url, "http://localhost:8900");
        assert_eq!(config.accounts, 15);
        assert_eq!(config.initial_balance, 200.0);
        assert_eq!(config.port, 8900);
    }

    #[test]
    fn test_provider_not_running_initially() {
        let provider = SolanaProvider::new();
        assert!(!provider.is_running());
    }

    #[test]
    fn test_rpc_client_when_not_running() {
        let provider = SolanaProvider::new();
        let result = provider.rpc_client();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, ChainError::NotRunning));
        }
    }

    #[test]
    fn test_get_accounts_when_not_started() {
        let provider = SolanaProvider::new();
        let accounts = provider.get_accounts().unwrap();
        // Should return empty accounts if not started
        assert_eq!(accounts.len(), 0);
    }
}
