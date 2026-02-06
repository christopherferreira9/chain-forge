use chain_forge_bitcoin_accounts::AccountsStorage;
use chain_forge_bitcoin_core::{BitcoinConfig, BitcoinProvider, InstanceInfo};
use chain_forge_bitcoin_rpc::BitcoinRpcClient;
use chain_forge_cli_utils::OutputFormat;
use chain_forge_common::{validate_name, ChainProvider};
use chain_forge_config::Config;
use clap::{Parser, Subcommand};
use eyre::Result;
use tabled::{Table, Tabled};

#[derive(Parser)]
#[command(name = "cf-bitcoin")]
#[command(about = "Chain Forge - Bitcoin local development tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start local Bitcoin regtest node with pre-funded accounts
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

        /// Initial balance for each account in BTC
        #[arg(short, long, default_value = "10.0")]
        balance: f64,

        /// RPC port for the node
        #[arg(long, default_value = "18443")]
        rpc_port: u16,

        /// P2P network port
        #[arg(long, default_value = "18444")]
        p2p_port: u16,

        /// Optional mnemonic phrase to use for account generation
        #[arg(short, long)]
        mnemonic: Option<String>,

        /// RPC username
        #[arg(long, default_value = "chainforge")]
        rpc_user: String,

        /// RPC password
        #[arg(long, default_value = "chainforge")]
        rpc_password: String,

        /// Show verbose bitcoind output
        #[arg(short, long, default_value = "false")]
        verbose: bool,

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

    /// Fund an account with BTC (from wallet funds)
    Fund {
        /// Account address to fund
        address: String,

        /// Amount of BTC to send
        amount: f64,

        /// Instance ID to use
        #[arg(short, long, default_value = "default")]
        instance: String,
    },

    /// Transfer BTC from one account to another
    Transfer {
        /// Source account address
        from: String,

        /// Destination account address
        to: String,

        /// Amount of BTC to send
        amount: f64,

        /// Instance ID to use
        #[arg(short, long, default_value = "default")]
        instance: String,
    },

    /// Mine blocks to an address
    Mine {
        /// Number of blocks to mine
        #[arg(short, long, default_value = "1")]
        blocks: u32,

        /// Address to receive coinbase rewards (uses account 0 if not specified)
        #[arg(short, long)]
        address: Option<String>,

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

    /// Stop the running node
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
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Balance (BTC)")]
    balance: String,
}

/// Get RPC client for a specific instance
fn get_rpc_client_for_instance(instance_id: &str) -> Result<BitcoinRpcClient> {
    let info = InstanceInfo::load(instance_id).map_err(|e| eyre::eyre!("{}", e))?;

    BitcoinRpcClient::new_with_wallet(
        info.rpc_url,
        info.rpc_user,
        info.rpc_password,
        "chain-forge",
    )
    .map_err(|e| eyre::eyre!("Failed to create RPC client: {}", e))
}

/// Get accounts storage for a specific instance
fn get_storage_for_instance(instance_id: &str) -> AccountsStorage {
    let accounts_file = Config::data_dir()
        .join("bitcoin")
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
            rpc_port,
            p2p_port,
            mnemonic,
            rpc_user,
            rpc_password,
            verbose,
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
            let mut config = BitcoinConfig::with_instance(&instance);
            config.rpc_url = format!("http://localhost:{}", rpc_port);
            config.rpc_port = rpc_port;
            config.p2p_port = p2p_port;
            config.accounts = accounts;
            config.initial_balance = balance;
            config.mnemonic = mnemonic;
            config.rpc_user = rpc_user;
            config.rpc_password = rpc_password;
            config.verbose = verbose;
            config.name = name;

            let mut provider = BitcoinProvider::with_config(config.clone());
            provider.set_keep_data(keep_data);
            provider.start(config)?;

            println!("💡 Tip: Keep this terminal open to keep the node running");
            println!(
                "   Run 'cf-bitcoin accounts --instance {}' in another terminal to see your accounts",
                instance
            );
            println!(
                "   Run 'cf-bitcoin mine --instance {}' to mine new blocks",
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
                    "No accounts found for instance '{}'. Run 'cf-bitcoin start --instance {}' first.",
                    instance, instance
                );
                return Ok(());
            }

            // Update balances from blockchain
            let balances_updated = match get_rpc_client_for_instance(&instance) {
                Ok(rpc_client) => match rpc_client.update_balances(&mut accounts) {
                    Ok(_) => {
                        // Save updated balances back to storage
                        if let Err(e) = storage.save(&accounts) {
                            eprintln!("Warning: Could not save updated balances: {}", e);
                        }
                        true
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not update balances from blockchain: {}", e);
                        eprintln!("Showing cached balances from startup.");
                        false
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Could not connect to Bitcoin node: {}", e);
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
                            address: acc.address.clone(),
                            balance: format!("{:.8}", acc.balance),
                        })
                        .collect();

                    let table = Table::new(display_accounts).to_string();
                    println!("{}", table);

                    if !balances_updated {
                        println!();
                        println!(
                            "Note: Balances shown are from startup cache (node not reachable)"
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

            if !rpc_client.is_node_running() {
                eprintln!(
                    "❌ Error: Bitcoin node is not running. Start it with 'cf-bitcoin start --instance {}'",
                    instance
                );
                std::process::exit(1);
            }

            println!("💰 Sending {} BTC to {} (from wallet)...", amount, address);

            match rpc_client.send_to_address(&address, amount) {
                Ok(txid) => {
                    println!("✅ Transaction sent!");
                    println!("   TxID: {}", txid);

                    // Mine a block to confirm using a wallet address (not user account)
                    println!("⛏️  Mining block to confirm transaction...");
                    if let Ok(mining_addr) = rpc_client.get_new_address(Some("mining")) {
                        if let Ok(blocks) = rpc_client.mine_blocks(1, &mining_addr) {
                            println!("   Block mined: {}", blocks[0]);
                        }
                    }

                    // Show updated balance
                    if let Ok(balance) = rpc_client.get_balance(&address) {
                        println!("   New balance: {} BTC", balance);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Transaction failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Transfer {
            from,
            to,
            amount,
            instance,
        } => {
            let rpc_client = get_rpc_client_for_instance(&instance)?;

            if !rpc_client.is_node_running() {
                eprintln!(
                    "❌ Error: Bitcoin node is not running. Start it with 'cf-bitcoin start --instance {}'",
                    instance
                );
                std::process::exit(1);
            }

            println!("💸 Transferring {} BTC", amount);
            println!("   From: {}", &from[..40.min(from.len())]);
            println!("   To:   {}", &to[..40.min(to.len())]);
            println!();

            // Show source balance before
            if let Ok(from_balance) = rpc_client.get_balance(&from) {
                println!("   Source balance: {} BTC", from_balance);
            }

            match rpc_client.send_from_address(&from, &to, amount) {
                Ok(txid) => {
                    println!("✅ Transaction sent!");
                    println!("   TxID: {}", txid);

                    // Mine a block to confirm
                    println!("⛏️  Mining block to confirm transaction...");
                    if let Ok(mining_addr) = rpc_client.get_new_address(Some("mining")) {
                        if let Ok(blocks) = rpc_client.mine_blocks(1, &mining_addr) {
                            println!("   Block mined: {}", blocks[0]);
                        }
                    }

                    // Show updated balances
                    println!();
                    println!("Updated balances:");
                    if let Ok(from_balance) = rpc_client.get_balance(&from) {
                        println!("   From: {} BTC", from_balance);
                    }
                    if let Ok(to_balance) = rpc_client.get_balance(&to) {
                        println!("   To:   {} BTC", to_balance);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Transfer failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Mine {
            blocks,
            address,
            instance,
        } => {
            let rpc_client = get_rpc_client_for_instance(&instance)?;

            if !rpc_client.is_node_running() {
                eprintln!(
                    "❌ Error: Bitcoin node is not running. Start it with 'cf-bitcoin start --instance {}'",
                    instance
                );
                std::process::exit(1);
            }

            // Use provided address or a wallet-generated address
            // This avoids adding coinbase rewards to user accounts unintentionally
            let mining_address = match address {
                Some(addr) => addr,
                None => {
                    // Get a wallet address for mining (not a user account)
                    rpc_client
                        .get_new_address(Some("mining"))
                        .map_err(|e| eyre::eyre!("Failed to get mining address: {}", e))?
                }
            };

            println!(
                "⛏️  Mining {} block(s) to {}...",
                blocks,
                &mining_address[..20]
            );

            match rpc_client.mine_blocks(blocks, &mining_address) {
                Ok(block_hashes) => {
                    println!("✅ Mined {} block(s)!", block_hashes.len());
                    for (i, hash) in block_hashes.iter().enumerate() {
                        println!("   Block {}: {}...", i + 1, &hash[..16]);
                    }

                    // Show current block height
                    if let Ok(count) = rpc_client.get_block_count() {
                        println!("   Current height: {}", count);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Mining failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Config { instance } => {
            println!("Chain Forge Bitcoin Configuration");
            println!("==================================");
            println!();
            println!("Instance: {}", instance);

            // Try to load instance info
            match InstanceInfo::load(&instance) {
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
                    println!("  P2P Port: {}", info.p2p_port);
                    println!("  Accounts: {}", info.accounts_count);
                }
                Err(_) => {
                    println!("  Status: Not initialized");
                    println!(
                        "  Run 'cf-bitcoin start --instance {}' to create this instance",
                        instance
                    );
                }
            }

            println!();
            println!(
                "Instance Directory: {:?}",
                Config::data_dir()
                    .join("bitcoin")
                    .join("instances")
                    .join(&instance)
            );

            // Also show global config if available
            let config = Config::load()?;
            if let Some(bitcoin_config) = config.bitcoin {
                println!();
                println!("Global Config (chain-forge.toml):");
                println!("  Default Profile:");
                println!("    Accounts: {}", bitcoin_config.default.accounts);
                println!(
                    "    Initial Balance: {} BTC",
                    bitcoin_config.default.initial_balance
                );
            }
        }

        Commands::Stop { instance } => {
            println!("Note: Use Ctrl+C to stop the node running in 'start' mode");
            println!(
                "      Instance '{}' should be stopped from its terminal",
                instance
            );

            // Mark instance as stopped if info exists
            if let Ok(mut info) = InstanceInfo::load(&instance) {
                info.running = false;
                let _ = info.save();
                println!("      Marked instance '{}' as stopped", instance);
            }
        }
    }

    Ok(())
}
