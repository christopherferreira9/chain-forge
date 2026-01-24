use chain_forge_bitcoin_accounts::{AccountGenerator, AccountsStorage, BitcoinAccount};
use chain_forge_bitcoin_rpc::BitcoinRpcClient;
use chain_forge_common::{ChainError, ChainProvider, Result};
use chain_forge_config::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Instance information saved to disk for CLI discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Instance ID
    pub instance_id: String,
    /// RPC URL for this instance
    pub rpc_url: String,
    /// RPC port
    pub rpc_port: u16,
    /// P2P port
    pub p2p_port: u16,
    /// RPC username
    pub rpc_user: String,
    /// RPC password
    pub rpc_password: String,
    /// Whether the instance is currently running (may be stale)
    pub running: bool,
}

impl InstanceInfo {
    /// Load instance info from the default location for an instance ID
    pub fn load(instance_id: &str) -> Result<Self> {
        let path = Config::data_dir()
            .join("bitcoin")
            .join("instances")
            .join(instance_id)
            .join("instance.json");

        if !path.exists() {
            return Err(ChainError::Other(format!(
                "Instance '{}' not found. Run 'cf-bitcoin start --instance {}' first.",
                instance_id, instance_id
            )));
        }

        let json = std::fs::read_to_string(&path)?;
        let info: InstanceInfo = serde_json::from_str(&json)?;
        Ok(info)
    }

    /// Save instance info to disk
    pub fn save(&self) -> Result<()> {
        let path = Config::data_dir()
            .join("bitcoin")
            .join("instances")
            .join(&self.instance_id)
            .join("instance.json");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Mark instance as stopped
    pub fn mark_stopped(&mut self) -> Result<()> {
        self.running = false;
        self.save()
    }
}

/// Configuration for starting a Bitcoin regtest node
#[derive(Debug, Clone)]
pub struct BitcoinConfig {
    /// RPC URL for connecting to bitcoind
    pub rpc_url: String,
    /// RPC port
    pub rpc_port: u16,
    /// P2P network port
    pub p2p_port: u16,
    /// Number of accounts to generate
    pub accounts: u32,
    /// Initial balance for each account in BTC
    pub initial_balance: f64,
    /// Optional mnemonic for deterministic account generation
    pub mnemonic: Option<String>,
    /// RPC username
    pub rpc_user: String,
    /// RPC password
    pub rpc_password: String,
    /// Data directory for Bitcoin
    pub data_dir: PathBuf,
    /// Show verbose bitcoind output
    pub verbose: bool,
    /// Instance ID for isolation (allows multiple nodes with separate state)
    pub instance_id: String,
}

impl Default for BitcoinConfig {
    fn default() -> Self {
        Self::with_instance("default")
    }
}

impl BitcoinConfig {
    /// Create a config with a specific instance ID
    pub fn with_instance(instance_id: &str) -> Self {
        let instance_dir = Config::data_dir()
            .join("bitcoin")
            .join("instances")
            .join(instance_id);
        let data_dir = instance_dir.join("regtest-data");

        Self {
            rpc_url: "http://127.0.0.1:18443".to_string(),
            rpc_port: 18443,
            p2p_port: 18444,
            accounts: 10,
            initial_balance: 10.0,
            mnemonic: None,
            rpc_user: "chainforge".to_string(),
            rpc_password: "chainforge".to_string(),
            data_dir,
            verbose: false,
            instance_id: instance_id.to_string(),
        }
    }

    /// Get the instance directory path
    pub fn instance_dir(&self) -> PathBuf {
        Config::data_dir()
            .join("bitcoin")
            .join("instances")
            .join(&self.instance_id)
    }

    /// Get the accounts file path for this instance
    pub fn accounts_file(&self) -> PathBuf {
        self.instance_dir().join("accounts.json")
    }

    /// Get the instance info file path
    pub fn instance_info_file(&self) -> PathBuf {
        self.instance_dir().join("instance.json")
    }
}

/// Bitcoin chain provider implementation
pub struct BitcoinProvider {
    config: BitcoinConfig,
    rpc_client: Option<BitcoinRpcClient>,
    accounts: Vec<BitcoinAccount>,
    bitcoind_process: Arc<Mutex<Option<Child>>>,
    storage: AccountsStorage,
}

impl BitcoinProvider {
    /// Create a new Bitcoin provider with default instance
    pub fn new() -> Self {
        Self::with_instance("default")
    }

    /// Create a provider for a specific instance ID
    pub fn with_instance(instance_id: &str) -> Self {
        let config = BitcoinConfig::with_instance(instance_id);
        Self::with_config(config)
    }

    /// Create a provider with a specific configuration
    pub fn with_config(config: BitcoinConfig) -> Self {
        // Use instance-specific storage path
        let storage = AccountsStorage::with_path(config.accounts_file());

        Self {
            config,
            rpc_client: None,
            accounts: Vec::new(),
            bitcoind_process: Arc::new(Mutex::new(None)),
            storage,
        }
    }

    /// Generate accounts for this instance
    ///
    /// Each node start gets fresh accounts. If a mnemonic is provided, the same
    /// addresses are generated but balances start fresh (blockchain is cleared).
    fn generate_accounts(&mut self) -> Result<()> {
        let generator = if let Some(mnemonic) = &self.config.mnemonic {
            AccountGenerator::from_mnemonic(mnemonic)?
        } else {
            AccountGenerator::new()?
        };

        println!("🔑 Mnemonic: {}", generator.mnemonic_phrase());
        println!("   Save this mnemonic to recover your accounts!");
        println!();

        self.accounts = generator.generate_accounts(self.config.accounts)?;

        // Set initial balance targets (will be funded after node starts)
        for account in &mut self.accounts {
            account.balance = self.config.initial_balance;
        }

        self.storage.save(&self.accounts)?;

        Ok(())
    }

    /// Clear all instance data (blockchain and accounts)
    fn clear_instance_data(&self) -> Result<()> {
        let instance_dir = self.config.instance_dir();

        // Remove the entire instance directory if it exists
        if instance_dir.exists() {
            std::fs::remove_dir_all(&instance_dir).map_err(|e| {
                ChainError::NodeManagement(format!("Failed to clear instance data: {}", e))
            })?;
        }

        Ok(())
    }

    /// Save instance info for CLI discovery
    fn save_instance_info(&self) -> Result<()> {
        let info = InstanceInfo {
            instance_id: self.config.instance_id.clone(),
            rpc_url: self.config.rpc_url.clone(),
            rpc_port: self.config.rpc_port,
            p2p_port: self.config.p2p_port,
            rpc_user: self.config.rpc_user.clone(),
            rpc_password: self.config.rpc_password.clone(),
            running: true,
        };
        info.save()
    }

    /// Start the bitcoind process in regtest mode
    fn start_bitcoind(&mut self) -> Result<()> {
        // Check if bitcoind is available
        let bitcoind_check = Command::new("bitcoind").arg("--version").output();

        if bitcoind_check.is_err() {
            return Err(ChainError::NodeManagement(
                "bitcoind not found. Please install Bitcoin Core.".to_string(),
            ));
        }

        // Ensure data directory exists
        std::fs::create_dir_all(&self.config.data_dir).map_err(|e| {
            ChainError::NodeManagement(format!("Failed to create data directory: {}", e))
        })?;

        println!(
            "🚀 Starting Bitcoin regtest node on port {}...",
            self.config.rpc_port
        );

        // Start bitcoind in regtest mode
        let mut cmd = Command::new("bitcoind");
        cmd.arg("-regtest")
            .arg(format!("-rpcport={}", self.config.rpc_port))
            .arg(format!("-port={}", self.config.p2p_port))
            .arg(format!("-datadir={}", self.config.data_dir.display()))
            .arg(format!("-rpcuser={}", self.config.rpc_user))
            .arg(format!("-rpcpassword={}", self.config.rpc_password))
            .arg("-server=1")
            .arg("-txindex=1")
            .arg("-fallbackfee=0.0001")
            .arg("-daemon=0"); // Run in foreground so we can manage the process

        // Only enable console output in verbose mode
        if self.config.verbose {
            cmd.arg("-printtoconsole=1");
        } else {
            cmd.arg("-printtoconsole=0");
        }

        // On Unix, we need to set file descriptor limits before spawning.
        // When the limit is "unlimited", bitcoind incorrectly sees -1 available.
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                // Set file descriptor limit to 10240 (more than enough for bitcoind)
                let limit = libc::rlimit {
                    rlim_cur: 10240,
                    rlim_max: 10240,
                };
                if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
                    // If setrlimit fails, try to continue anyway
                    eprintln!("Warning: Failed to set file descriptor limit");
                }
                Ok(())
            });
        }

        // Configure stdio based on verbose mode
        let child = if self.config.verbose {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::inherit()) // Forward output to parent
                .stderr(Stdio::inherit()) // Forward errors to parent
                .spawn()
        } else {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null()) // Suppress output
                .stderr(Stdio::null()) // Suppress errors
                .spawn()
        }
        .map_err(|e| ChainError::NodeManagement(format!("Failed to start bitcoind: {}", e)))?;

        let mut process_guard = self.bitcoind_process.lock().unwrap();
        *process_guard = Some(child);

        Ok(())
    }

    /// Get a reference to the RPC client
    pub fn rpc_client(&self) -> Result<&BitcoinRpcClient> {
        self.rpc_client.as_ref().ok_or(ChainError::NotRunning)
    }

    /// Mine blocks to a specific address
    pub fn mine_blocks(&self, count: u32, address: Option<&str>) -> Result<Vec<String>> {
        let client = self.rpc_client()?;

        // Use first account's address if none specified
        let mining_address = match address {
            Some(addr) => addr.to_string(),
            None => {
                if self.accounts.is_empty() {
                    return Err(ChainError::AccountGeneration(
                        "No accounts available for mining".to_string(),
                    ));
                }
                self.accounts[0].address.clone()
            }
        };

        client.mine_blocks(count, &mining_address)
    }
}

impl Default for BitcoinProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainProvider for BitcoinProvider {
    type Account = BitcoinAccount;
    type Transaction = ();
    type Config = BitcoinConfig;

    fn start(&mut self, config: Self::Config) -> Result<()> {
        if self.is_running() {
            return Err(ChainError::AlreadyRunning);
        }

        self.config = config.clone();

        // Update storage to use instance-specific path
        self.storage = AccountsStorage::with_path(self.config.accounts_file());

        println!(
            "🧹 Clearing previous instance data for '{}'...",
            self.config.instance_id
        );

        // Clear all previous instance data (blockchain + accounts) for clean slate
        self.clear_instance_data()?;

        // Generate fresh accounts
        self.generate_accounts()?;

        // Start bitcoind
        self.start_bitcoind()?;

        // Save instance info for CLI discovery
        self.save_instance_info()?;

        // Initialize RPC and fund accounts using a separate thread
        let accounts_file = self.config.accounts_file();
        let result = std::thread::spawn({
            let rpc_url = self.config.rpc_url.clone();
            let rpc_user = self.config.rpc_user.clone();
            let rpc_password = self.config.rpc_password.clone();
            let initial_balance = self.config.initial_balance;
            let mut accounts_vec = self.accounts.clone();

            move || {
                // Create a new runtime in this thread
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| ChainError::Other(format!("Failed to create runtime: {}", e)))?;

                rt.block_on(async {
                    // Create base RPC client first (no wallet)
                    let base_client = BitcoinRpcClient::new(
                        rpc_url.clone(),
                        rpc_user.clone(),
                        rpc_password.clone(),
                    )?;

                    println!("⏳ Waiting for Bitcoin node to be ready...");
                    base_client.wait_for_node(60).await?;

                    println!("✅ Bitcoin node is ready!");
                    println!();

                    // Create wallet
                    println!("📦 Creating wallet...");
                    base_client.create_wallet("chain-forge")?;

                    // Small delay to ensure wallet is fully initialized
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    // Connect to wallet
                    let wallet_client = BitcoinRpcClient::new_with_wallet(
                        rpc_url,
                        rpc_user,
                        rpc_password,
                        "chain-forge",
                    )?;

                    // Get a wallet address for mining (not one of our accounts)
                    // IMPORTANT: We import accounts AFTER funding to prevent the wallet
                    // from spending the newly created UTXOs in subsequent transactions
                    let mining_address = wallet_client.get_new_address(Some("mining"))?;
                    println!("📍 Mining address: {}", &mining_address[..20]);

                    // Calculate how many blocks to mine for sufficient funds
                    // Each coinbase gives 50 BTC, needs 100 confirmations to be spendable
                    // We need: num_accounts * initial_balance BTC (fund ALL accounts)
                    let total_btc_needed = accounts_vec.len() as f64 * initial_balance;
                    let coinbases_needed = (total_btc_needed / 50.0).ceil() as u32;
                    // Mine 100 blocks for maturity + enough blocks for spendable coinbases
                    let blocks_to_mine = 100 + coinbases_needed.max(1);

                    println!(
                        "⛏️  Mining {} initial blocks (this may take a moment)...",
                        blocks_to_mine
                    );
                    wallet_client.mine_blocks(blocks_to_mine, &mining_address)?;

                    // Wait for UTXO set to stabilize
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    // Check wallet balance before funding
                    let wallet_balance = wallet_client.get_wallet_balance()?;
                    let needed = accounts_vec.len() as f64 * initial_balance;
                    println!(
                        "   Wallet balance: {} BTC (need {} BTC for {} accounts)",
                        wallet_balance,
                        needed,
                        accounts_vec.len()
                    );

                    if wallet_balance < needed {
                        return Err(ChainError::Other(format!(
                            "Insufficient wallet balance: {} BTC available, {} BTC needed",
                            wallet_balance, needed
                        )));
                    }

                    // Fund ALL accounts with initial balance
                    // We do this BEFORE importing so the wallet doesn't spend from them
                    println!(
                        "💰 Funding {} accounts with {} BTC each...",
                        accounts_vec.len(),
                        initial_balance
                    );

                    // Set amount to send for each account
                    for account in accounts_vec.iter_mut() {
                        account.balance = initial_balance;
                    }

                    // Fund accounts (may fail partially, will error if any fail)
                    if let Err(e) = wallet_client.fund_accounts(&mut accounts_vec).await {
                        eprintln!("Warning: Some accounts failed to fund: {}", e);
                        eprintln!("         Continuing with partially funded accounts...");
                    }

                    // Mine blocks to confirm all transactions
                    println!("⛏️  Mining 6 blocks to confirm transactions...");
                    wallet_client.mine_blocks(6, &mining_address)?;

                    // Wait for UTXO set to update after mining
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    // NOW import accounts into the wallet so we can track their balances
                    println!("📥 Importing accounts into wallet...");
                    for (i, account) in accounts_vec.iter().enumerate() {
                        wallet_client.import_address(
                            &account.address,
                            &account.wif,
                            &format!("account-{}", i),
                        )?;
                    }

                    // Wait for wallet to process imports
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    // Update all balances
                    wallet_client.update_balances(&mut accounts_vec)?;

                    // Save updated accounts to instance-specific location
                    let storage = AccountsStorage::with_path(accounts_file);
                    storage.save(&accounts_vec)?;

                    println!("✅ All accounts funded!");
                    println!();

                    Ok::<(BitcoinRpcClient, Vec<BitcoinAccount>), ChainError>((
                        wallet_client,
                        accounts_vec,
                    ))
                })
            }
        })
        .join()
        .map_err(|_| ChainError::Other("Initialization thread panicked".to_string()))??;

        self.rpc_client = Some(result.0);
        self.accounts = result.1;

        println!("🎉 Bitcoin regtest node is running!");
        println!("   RPC URL: {}", self.config.rpc_url);
        println!();

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let mut process_guard = self.bitcoind_process.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            // Kill the process
            let _ = child.kill();
            child.wait().map_err(|e| {
                ChainError::NodeManagement(format!("Failed to wait for bitcoind: {}", e))
            })?;

            // Mark instance as stopped
            if let Ok(mut info) = InstanceInfo::load(&self.config.instance_id) {
                let _ = info.mark_stopped();
            }

            println!(
                "🛑 Bitcoin regtest node stopped (instance: {})",
                self.config.instance_id
            );
        }

        self.rpc_client = None;

        Ok(())
    }

    fn is_running(&self) -> bool {
        let process_guard = self.bitcoind_process.lock().unwrap();
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
        client.send_to_address(address, amount)
    }

    fn get_balance(&self, address: &str) -> Result<f64> {
        let client = self.rpc_client()?;
        client.get_balance(address)
    }

    fn get_rpc_url(&self) -> String {
        self.config.rpc_url.clone()
    }
}

impl Drop for BitcoinProvider {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = BitcoinProvider::new();
        assert!(!provider.is_running());
    }

    #[test]
    fn test_config_defaults() {
        let config = BitcoinConfig::default();
        assert_eq!(config.rpc_port, 18443);
        assert_eq!(config.p2p_port, 18444);
        assert_eq!(config.accounts, 10);
        assert_eq!(config.initial_balance, 10.0);
        assert_eq!(config.rpc_url, "http://127.0.0.1:18443");
        assert_eq!(config.rpc_user, "chainforge");
        assert_eq!(config.rpc_password, "chainforge");
        assert!(config.mnemonic.is_none());
    }

    #[test]
    fn test_provider_with_config() {
        let config = BitcoinConfig {
            rpc_url: "http://localhost:19000".to_string(),
            rpc_port: 19000,
            p2p_port: 19001,
            accounts: 5,
            initial_balance: 50.0,
            mnemonic: None,
            rpc_user: "test".to_string(),
            rpc_password: "test".to_string(),
            data_dir: PathBuf::from("/tmp/bitcoin-test"),
            verbose: false,
            instance_id: "test".to_string(),
        };

        let provider = BitcoinProvider::with_config(config);
        assert!(!provider.is_running());
        assert_eq!(provider.get_rpc_url(), "http://localhost:19000");
    }

    #[test]
    fn test_provider_not_running_initially() {
        let provider = BitcoinProvider::new();
        assert!(!provider.is_running());
    }

    #[test]
    fn test_rpc_client_when_not_running() {
        let provider = BitcoinProvider::new();
        let result = provider.rpc_client();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, ChainError::NotRunning));
        }
    }

    #[test]
    fn test_get_accounts_when_not_started() {
        let provider = BitcoinProvider::new();
        let accounts = provider.get_accounts().unwrap();
        // Should return empty accounts if not started
        assert_eq!(accounts.len(), 0);
    }
}
