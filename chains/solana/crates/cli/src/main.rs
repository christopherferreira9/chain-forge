use chain_forge_cli_utils::OutputFormat;
use chain_forge_common::{validate_name, ChainProvider};
use chain_forge_config::Config;
use chain_forge_solana_accounts::AccountsStorage;
use chain_forge_solana_core::{SolanaConfig, SolanaInstanceInfo, SolanaProvider};
use chain_forge_solana_rpc::SolanaRpcClient;
use clap::{Parser, Subcommand};
use eyre::Result;
use tabled::{Table, Tabled};

#[derive(Parser)]
#[command(name = "cf-solana")]
#[command(about = "Chain Forge - Solana local development tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start local Solana test validator with pre-funded accounts
    Start {
        /// Instance ID for isolation (allows multiple nodes with separate state)
        #[arg(short, long, default_value = "default")]
        instance: String,

        /// Human-readable name for the instance
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// Number of accounts to generate
        #[arg(short, long, default_value = "10")]
        accounts: u32,

        /// Initial balance for each account in SOL
        #[arg(short, long, default_value = "100.0")]
        balance: f64,

        /// RPC port for the validator
        #[arg(short, long, default_value = "8899")]
        port: u16,

        /// Optional mnemonic phrase to use for account generation
        #[arg(short, long)]
        mnemonic: Option<String>,

        /// Keep instance data on stop (default: clean up)
        #[arg(long, default_value = "false")]
        keep_data: bool,
    },

    /// List all generated accounts with their balances
    Accounts {
        /// Instance ID to query
        #[arg(short, long, default_value = "default")]
        instance: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Fund an account with SOL
    Fund {
        /// Account address to fund
        address: String,

        /// Amount of SOL to send
        amount: f64,

        /// Instance ID to use
        #[arg(short, long, default_value = "default")]
        instance: String,
    },

    /// Show current configuration
    Config {
        /// Instance ID to show config for
        #[arg(short, long, default_value = "default")]
        instance: String,
    },

    /// Stop the running validator
    Stop {
        /// Instance ID to stop
        #[arg(short, long, default_value = "default")]
        instance: String,
    },
}

#[derive(Tabled)]
struct AccountDisplay {
    #[tabled(rename = "Index")]
    index: usize,
    #[tabled(rename = "Public Key")]
    public_key: String,
    #[tabled(rename = "Balance (SOL)")]
    balance: String,
}

/// Get RPC client for a specific instance
fn get_rpc_client_for_instance(instance_id: &str) -> Result<SolanaRpcClient> {
    let info = SolanaInstanceInfo::load(instance_id).map_err(|e| eyre::eyre!("{}", e))?;
    Ok(SolanaRpcClient::new(info.rpc_url))
}

/// Get accounts storage for a specific instance
fn get_storage_for_instance(instance_id: &str) -> AccountsStorage {
    let accounts_file = Config::data_dir()
        .join("solana")
        .join("instances")
        .join(instance_id)
        .join("accounts.json");
    AccountsStorage::with_path(accounts_file)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            instance,
            name,
            accounts,
            balance,
            port,
            mnemonic,
            keep_data,
        } => {
            // Validate instance name
            if let Err(e) = validate_name(&instance) {
                eprintln!("❌ Invalid instance name: {}", e);
                std::process::exit(1);
            }

            // Validate display name if provided
            if let Some(ref n) = name {
                if let Err(e) = validate_name(n) {
                    eprintln!("❌ Invalid display name: {}", e);
                    std::process::exit(1);
                }
            }

            // Create instance-specific config
            let mut config = SolanaConfig::with_instance(&instance);
            config.rpc_url = format!("http://localhost:{}", port);
            config.port = port;
            config.accounts = accounts;
            config.initial_balance = balance;
            config.mnemonic = mnemonic;
            config.name = name;

            let mut provider = SolanaProvider::with_config(config.clone());
            provider.set_keep_data(keep_data);
            provider.start(config)?;

            println!("💡 Tip: Keep this terminal open to keep the validator running");
            println!(
                "   Run 'cf-solana accounts --instance {}' in another terminal to see your accounts",
                instance
            );
            println!();

            // Keep the process alive
            tokio::signal::ctrl_c().await?;
            println!();
            provider.stop()?;
        }

        Commands::Accounts { instance, format } => {
            let storage = get_storage_for_instance(&instance);

            let mut accounts = storage.load()?;

            if accounts.is_empty() {
                println!(
                    "No accounts found for instance '{}'. Run 'cf-solana start --instance {}' first.",
                    instance, instance
                );
                return Ok(());
            }

            // Update balances from validator
            let balances_updated = match get_rpc_client_for_instance(&instance) {
                Ok(rpc_client) => {
                    if rpc_client.is_validator_running() {
                        match rpc_client.update_balances(&mut accounts) {
                            Ok(_) => {
                                // Save updated balances back to storage
                                if let Err(e) = storage.save(&accounts) {
                                    eprintln!("Warning: Could not save updated balances: {}", e);
                                }
                                true
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Could not update balances from validator: {}",
                                    e
                                );
                                false
                            }
                        }
                    } else {
                        eprintln!("Warning: Validator is not running. Showing cached balances.");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Could not connect to instance: {}", e);
                    eprintln!("Showing cached balances from startup.");
                    false
                }
            };

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&accounts)?);
                }
                OutputFormat::Table => {
                    let display_accounts: Vec<AccountDisplay> = accounts
                        .iter()
                        .enumerate()
                        .map(|(i, acc)| AccountDisplay {
                            index: i,
                            public_key: acc.public_key.clone(),
                            balance: format!("{:.2}", acc.balance),
                        })
                        .collect();

                    let table = Table::new(display_accounts).to_string();
                    println!("{}", table);

                    if !balances_updated {
                        println!();
                        println!(
                            "Note: Balances shown are from startup cache (validator not reachable)"
                        );
                    }
                }
            }
        }

        Commands::Fund {
            address,
            amount,
            instance,
        } => {
            let rpc_client = get_rpc_client_for_instance(&instance)?;

            if !rpc_client.is_validator_running() {
                eprintln!(
                    "❌ Error: Validator is not running. Start it with 'cf-solana start --instance {}'",
                    instance
                );
                std::process::exit(1);
            }

            println!("💰 Requesting airdrop of {} SOL to {}...", amount, address);

            match rpc_client.request_airdrop(&address, amount) {
                Ok(signature) => {
                    println!("✅ Airdrop successful!");
                    println!("   Signature: {}", signature);

                    // Show updated balance
                    if let Ok(balance) = rpc_client.get_balance(&address) {
                        println!("   New balance: {} SOL", balance);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Airdrop failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Config { instance } => {
            println!("Chain Forge Solana Configuration");
            println!("=================================");
            println!();
            println!("Instance: {}", instance);

            // Try to load instance info
            match SolanaInstanceInfo::load(&instance) {
                Ok(info) => {
                    println!(
                        "  Status: {}",
                        if info.running {
                            "Running (may be stale)"
                        } else {
                            "Stopped"
                        }
                    );
                    if let Some(name) = &info.name {
                        println!("  Name: {}", name);
                    }
                    println!("  RPC URL: {}", info.rpc_url);
                    println!("  RPC Port: {}", info.rpc_port);
                    println!("  Accounts: {}", info.accounts_count);
                }
                Err(_) => {
                    println!("  Status: Not initialized");
                    println!(
                        "  Run 'cf-solana start --instance {}' to create this instance",
                        instance
                    );
                }
            }

            println!();
            println!(
                "Instance Directory: {:?}",
                Config::data_dir()
                    .join("solana")
                    .join("instances")
                    .join(&instance)
            );

            // Also show global config if available
            let config = Config::load()?;
            if let Some(solana_config) = config.solana {
                println!();
                println!("Global Config (chain-forge.toml):");
                println!("  Default Profile:");
                println!("    RPC URL: {}", solana_config.default.rpc_url);
                println!("    Accounts: {}", solana_config.default.accounts);
                println!(
                    "    Initial Balance: {} SOL",
                    solana_config.default.initial_balance
                );
                println!("    Port: {}", solana_config.default.port);
            }
        }

        Commands::Stop { instance } => {
            println!("Note: Use Ctrl+C to stop the validator running in 'start' mode");
            println!(
                "      Instance '{}' should be stopped from its terminal",
                instance
            );

            // Mark instance as stopped if info exists
            if let Ok(mut info) = SolanaInstanceInfo::load(&instance) {
                info.running = false;
                let _ = info.save();
                println!("      Marked instance '{}' as stopped", instance);
            }
        }
    }

    Ok(())
}
