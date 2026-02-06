use chain_forge_common::{
    ChainError, ChainProvider, ChainType, NodeInfo, NodeRegistry, NodeStatus, Result,
};
use chain_forge_config::{Config, SolanaProfile};
use chain_forge_solana_accounts::{AccountGenerator, AccountsStorage, SolanaAccount};
use chain_forge_solana_rpc::SolanaRpcClient;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

/// Instance information saved to disk for CLI discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaInstanceInfo {
    /// Instance ID
    pub instance_id: String,
    /// Human-readable name for the instance
    pub name: Option<String>,
    /// RPC URL for this instance
    pub rpc_url: String,
    /// RPC port
    pub rpc_port: u16,
    /// Number of accounts
    pub accounts_count: u32,
    /// Whether the instance is currently running (may be stale)
    pub running: bool,
}

impl SolanaInstanceInfo {
    /// Load instance info from the default location for an instance ID
    pub fn load(instance_id: &str) -> Result<Self> {
        let path = Config::data_dir()
            .join("solana")
            .join("instances")
            .join(instance_id)
            .join("instance.json");

        if !path.exists() {
            return Err(ChainError::Other(format!(
                "Instance '{}' not found. Run 'cf-solana start --instance {}' first.",
                instance_id, instance_id
            )));
        }

        let json = std::fs::read_to_string(&path)?;
        let info: SolanaInstanceInfo = serde_json::from_str(&json)?;
        Ok(info)
    }

    /// Save instance info to disk
    pub fn save(&self) -> Result<()> {
        let path = Config::data_dir()
            .join("solana")
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

/// Configuration for starting a Solana validator
#[derive(Debug, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub port: u16,
    pub accounts: u32,
    pub initial_balance: f64,
    pub mnemonic: Option<String>,
    /// Instance ID for isolation (allows multiple nodes with separate state)
    pub instance_id: String,
    /// Human-readable name for the instance
    pub name: Option<String>,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self::with_instance("default")
    }
}

impl SolanaConfig {
    /// Create a config with a specific instance ID
    pub fn with_instance(instance_id: &str) -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(),
            port: 8899,
            accounts: 10,
            initial_balance: 100.0,
            mnemonic: None,
            instance_id: instance_id.to_string(),
            name: None,
        }
    }

    /// Get the instance directory path
    pub fn instance_dir(&self) -> PathBuf {
        Config::data_dir()
            .join("solana")
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

impl From<SolanaProfile> for SolanaConfig {
    fn from(profile: SolanaProfile) -> Self {
        Self {
            rpc_url: profile.rpc_url,
            port: profile.port,
            accounts: profile.accounts,
            initial_balance: profile.initial_balance,
            mnemonic: None,
            instance_id: "default".to_string(),
            name: None,
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
    /// Whether to keep instance data on stop (default: false)
    keep_data: bool,
}

impl SolanaProvider {
    /// Create a new Solana provider with default instance
    pub fn new() -> Self {
        Self::with_instance("default")
    }

    /// Create a provider for a specific instance ID
    pub fn with_instance(instance_id: &str) -> Self {
        let config = SolanaConfig::with_instance(instance_id);
        Self::with_config(config)
    }

    /// Create a provider with a specific configuration
    pub fn with_config(config: SolanaConfig) -> Self {
        // Use instance-specific storage path
        let storage = AccountsStorage::with_path(config.accounts_file());

        Self {
            config,
            rpc_client: None,
            accounts: Vec::new(),
            validator_process: Arc::new(Mutex::new(None)),
            storage,
            keep_data: false,
        }
    }

    /// Set whether to keep instance data on stop
    pub fn set_keep_data(&mut self, keep: bool) {
        self.keep_data = keep;
    }

    /// Generate accounts for this instance
    ///
    /// Each node start gets fresh accounts. If a mnemonic is provided, the same
    /// addresses are generated but balances start fresh (validator is reset).
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

        // Set initial balance targets (will be funded after validator starts)
        for account in &mut self.accounts {
            account.balance = self.config.initial_balance;
        }

        self.storage.save(&self.accounts)?;

        Ok(())
    }

    /// Clear all instance data
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
        let info = SolanaInstanceInfo {
            instance_id: self.config.instance_id.clone(),
            name: self.config.name.clone(),
            rpc_url: self.config.rpc_url.clone(),
            rpc_port: self.config.port,
            accounts_count: self.config.accounts,
            running: true,
        };
        info.save()
    }

    /// Register this node with the global registry
    fn register_with_registry(&self) -> Result<()> {
        let registry = NodeRegistry::new();
        let node = NodeInfo::new(
            ChainType::Solana,
            &self.config.instance_id,
            self.config.name.clone(),
            self.config.rpc_url.clone(),
            self.config.port,
            self.config.accounts,
        );
        registry.register(node)
    }

    /// Unregister this node from the global registry
    fn unregister_from_registry(&self) -> Result<()> {
        let registry = NodeRegistry::new();
        let node_id = NodeRegistry::node_id(ChainType::Solana, &self.config.instance_id);
        registry.update_status(&node_id, NodeStatus::Stopped)
    }

    /// Check if a port is available for binding
    fn check_port_available(port: u16, description: &str) -> Result<()> {
        std::net::TcpListener::bind(("0.0.0.0", port)).map_err(|_| {
            ChainError::NodeManagement(format!(
                "{} port {} is already in use. Check for other running validators or services.",
                description, port
            ))
        })?;
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

        // Calculate faucet port based on RPC port to avoid conflicts
        // Use RPC port + 1002 (e.g., 8899 -> 9901, 9000 -> 10002)
        let faucet_port = self.config.port + 1002;

        // Check if required ports are available before starting
        Self::check_port_available(self.config.port, "RPC")?;
        Self::check_port_available(faucet_port, "Faucet")?;

        let instance_name = self
            .config
            .name
            .as_ref()
            .unwrap_or(&self.config.instance_id);
        println!(
            "🚀 Starting Solana test validator '{}' on port {}...",
            instance_name, self.config.port
        );

        // Create log files for capturing startup output and errors
        let log_dir = self.config.instance_dir();
        std::fs::create_dir_all(&log_dir).ok();
        let stdout_file = std::fs::File::create(log_dir.join("validator_stdout.log"))
            .map_err(|e| {
                ChainError::NodeManagement(format!("Failed to create stdout log: {}", e))
            })?;
        let stderr_file = std::fs::File::create(log_dir.join("validator_stderr.log"))
            .map_err(|e| {
                ChainError::NodeManagement(format!("Failed to create stderr log: {}", e))
            })?;

        // Use instance-specific ledger directory to allow multiple concurrent validators
        let ledger_dir = self.config.instance_dir().join("test-ledger");

        // Each instance needs its own gossip port and dynamic port range to avoid
        // conflicts when running multiple validators concurrently.
        // Gossip port defaults to 8000 and is NOT covered by --dynamic-port-range,
        // so it must be set explicitly via --gossip-port.
        let gossip_port = faucet_port + 1;
        let dynamic_base = faucet_port + 2;
        let dynamic_end = dynamic_base + 500;

        // Pre-check gossip port availability
        Self::check_port_available(gossip_port, "Gossip")?;

        // Start the validator
        // Note: --quiet is omitted because output is redirected to log files anyway,
        // and --quiet can suppress error messages we need to diagnose startup failures.
        let mut cmd = Command::new("solana-test-validator");
        cmd.arg("--rpc-port")
            .arg(self.config.port.to_string())
            .arg("--faucet-port")
            .arg(faucet_port.to_string())
            .arg("--gossip-port")
            .arg(gossip_port.to_string())
            .arg("--dynamic-port-range")
            .arg(format!("{}-{}", dynamic_base, dynamic_end))
            .arg("--ledger")
            .arg(&ledger_dir)
            .arg("--reset")
            .stdout(stdout_file)
            .stderr(stderr_file);

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

        self.config = config.clone();

        // Check if this instance is already running in the registry
        let registry = NodeRegistry::new();
        let node_id = NodeRegistry::node_id(ChainType::Solana, &self.config.instance_id);
        if let Ok(Some(existing)) = registry.get(&node_id) {
            if existing.status == NodeStatus::Running {
                return Err(ChainError::NodeManagement(format!(
                    "Instance '{}' is already running on port {}. Stop it first or use a different instance name.",
                    self.config.instance_id, existing.rpc_port
                )));
            }
        }

        // Update storage to use instance-specific path
        self.storage = AccountsStorage::with_path(self.config.accounts_file());

        println!(
            "🧹 Clearing previous instance data for '{}'...",
            self.config.instance_id
        );

        // Clear all previous instance data for clean slate
        self.clear_instance_data()?;

        // Generate fresh accounts
        self.generate_accounts()?;

        // Start validator
        self.start_validator()?;

        // Brief pause to detect early startup failures (e.g., port already in use)
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Check if the validator process exited early
        {
            let mut process_guard = self.validator_process.lock().unwrap();
            if let Some(ref mut child) = *process_guard {
                if let Ok(Some(status)) = child.try_wait() {
                    process_guard.take();
                    let log_dir = self.config.instance_dir();

                    // Check the validator's own log for the real error
                    // (panics and internal errors go there, not to stdout/stderr)
                    let validator_log_path =
                        log_dir.join("test-ledger").join("validator.log");
                    let error_detail = std::fs::read_to_string(&validator_log_path)
                        .ok()
                        .and_then(|content| {
                            // Extract panic or error lines
                            let errors: Vec<&str> = content
                                .lines()
                                .filter(|l| {
                                    l.contains("panicked at")
                                        || (l.contains("ERROR") && !l.contains("metrics"))
                                })
                                .collect();
                            if errors.is_empty() {
                                None
                            } else {
                                Some(errors.join("\n"))
                            }
                        });

                    let error_msg = match error_detail {
                        Some(detail) => {
                            format!("Validator failed to start: {}", detail)
                        }
                        None => {
                            format!(
                                "Validator process exited unexpectedly (exit code: {}). \
                                 Check logs at: {}",
                                status,
                                validator_log_path.display()
                            )
                        }
                    };
                    return Err(ChainError::NodeManagement(error_msg));
                }
            }
        }

        // Save instance info for CLI discovery
        self.save_instance_info()?;

        // Initialize RPC and fund accounts using a separate thread
        let accounts_file = self.config.accounts_file();
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

                    // Save updated accounts to instance-specific location
                    let storage = AccountsStorage::with_path(accounts_file);
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

        // Register with global node registry
        if let Err(e) = self.register_with_registry() {
            eprintln!("Warning: Failed to register with node registry: {}", e);
        }

        let instance_name = self
            .config
            .name
            .as_ref()
            .unwrap_or(&self.config.instance_id);
        println!("🎉 Solana test validator '{}' is running!", instance_name);
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

            // Unregister from global node registry
            if let Err(e) = self.unregister_from_registry() {
                eprintln!("Warning: Failed to unregister from node registry: {}", e);
            }

            child.wait().map_err(|e| {
                ChainError::NodeManagement(format!("Failed to wait for validator: {}", e))
            })?;

            // Mark instance as stopped
            if let Ok(mut info) = SolanaInstanceInfo::load(&self.config.instance_id) {
                let _ = info.mark_stopped();
            }

            // Clean up instance data unless keep_data is set
            if !self.keep_data {
                let _ = self.clear_instance_data();
            }

            println!(
                "🛑 Solana test validator stopped (instance: {})",
                self.config.instance_id
            );
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
        assert_eq!(config.instance_id, "default");
        assert!(config.name.is_none());
    }

    #[test]
    fn test_config_with_instance() {
        let config = SolanaConfig::with_instance("test-instance");
        assert_eq!(config.instance_id, "test-instance");
        assert!(config.name.is_none());
    }

    #[test]
    fn test_provider_with_config() {
        let mut config = SolanaConfig::with_instance("test");
        config.rpc_url = "http://localhost:9000".to_string();
        config.port = 9000;
        config.accounts = 5;
        config.initial_balance = 50.0;

        let provider = SolanaProvider::with_config(config);
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
        assert_eq!(config.instance_id, "default"); // Should default to "default"
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

    #[test]
    fn test_instance_paths() {
        let config = SolanaConfig::with_instance("my-instance");
        let instance_dir = config.instance_dir();
        assert!(instance_dir.ends_with("solana/instances/my-instance"));

        let accounts_file = config.accounts_file();
        assert!(accounts_file.ends_with("solana/instances/my-instance/accounts.json"));

        let instance_info_file = config.instance_info_file();
        assert!(instance_info_file.ends_with("solana/instances/my-instance/instance.json"));
    }

    #[test]
    fn test_config_with_name() {
        let mut config = SolanaConfig::with_instance("test-instance");
        config.name = Some("Test Node".to_string());
        assert_eq!(config.instance_id, "test-instance");
        assert_eq!(config.name, Some("Test Node".to_string()));
    }

    #[test]
    fn test_provider_with_instance() {
        let provider = SolanaProvider::with_instance("my-test-instance");
        assert!(!provider.is_running());
        // Default RPC URL should be set
        assert_eq!(provider.get_rpc_url(), "http://localhost:8899");
    }

    #[test]
    fn test_keep_data_flag() {
        let mut provider = SolanaProvider::new();
        // Default should be false
        assert!(!provider.keep_data);

        provider.set_keep_data(true);
        assert!(provider.keep_data);

        provider.set_keep_data(false);
        assert!(!provider.keep_data);
    }

    #[test]
    fn test_instance_info_serialization() {
        let info = SolanaInstanceInfo {
            instance_id: "test".to_string(),
            name: Some("Test Node".to_string()),
            rpc_url: "http://localhost:8899".to_string(),
            rpc_port: 8899,
            accounts_count: 10,
            running: true,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"instance_id\":\"test\""));
        assert!(json.contains("\"name\":\"Test Node\""));
        assert!(json.contains("\"running\":true"));

        // Deserialize back
        let deserialized: SolanaInstanceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.instance_id, "test");
        assert_eq!(deserialized.name, Some("Test Node".to_string()));
        assert_eq!(deserialized.rpc_url, "http://localhost:8899");
        assert_eq!(deserialized.rpc_port, 8899);
        assert_eq!(deserialized.accounts_count, 10);
        assert!(deserialized.running);
    }

    #[test]
    fn test_instance_info_without_name() {
        let info = SolanaInstanceInfo {
            instance_id: "default".to_string(),
            name: None,
            rpc_url: "http://localhost:8899".to_string(),
            rpc_port: 8899,
            accounts_count: 5,
            running: false,
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SolanaInstanceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.instance_id, "default");
        assert!(deserialized.name.is_none());
        assert!(!deserialized.running);
    }

    #[test]
    fn test_different_instance_configs() {
        let config1 = SolanaConfig::with_instance("dev");
        let config2 = SolanaConfig::with_instance("test");

        // Each should have its own instance directory
        assert_ne!(config1.instance_dir(), config2.instance_dir());
        assert_ne!(config1.accounts_file(), config2.accounts_file());
        assert_ne!(config1.instance_info_file(), config2.instance_info_file());

        // But same default port
        assert_eq!(config1.port, config2.port);
    }
}
