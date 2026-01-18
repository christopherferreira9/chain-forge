use chain_forge_cli_utils::OutputFormat;
use chain_forge_common::ChainProvider;
use chain_forge_config::Config;
use chain_forge_solana_accounts::AccountsStorage;
use chain_forge_solana_core::{SolanaConfig, SolanaProvider};
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
    },

    /// List all generated accounts with their balances
    Accounts {
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
    },

    /// Show current configuration
    Config,

    /// Stop the running validator
    Stop,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            accounts,
            balance,
            port,
            mnemonic,
        } => {
            let config = SolanaConfig {
                rpc_url: format!("http://localhost:{}", port),
                port,
                accounts,
                initial_balance: balance,
                mnemonic,
            };

            let mut provider = SolanaProvider::with_config(config.clone());
            provider.start(config)?;

            println!("💡 Tip: Keep this terminal open to keep the validator running");
            println!("   Run 'cf-solana accounts' in another terminal to see your accounts");
            println!();

            // Keep the process alive
            tokio::signal::ctrl_c().await?;
            println!();
            provider.stop()?;
        }

        Commands::Accounts { format } => {
            let data_dir = Config::ensure_data_dir()?;
            let storage = AccountsStorage::new(&data_dir);

            let accounts = storage.load()?;

            if accounts.is_empty() {
                println!("No accounts found. Run 'cf-solana start' first.");
                return Ok(());
            }

            // Try to update balances if validator is running
            let rpc_client = SolanaRpcClient::new("http://localhost:8899".to_string());
            let mut accounts = accounts;

            if rpc_client.is_validator_running() {
                let _ = rpc_client.update_balances(&mut accounts);
            }

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
                }
            }
        }

        Commands::Fund { address, amount } => {
            let rpc_client = SolanaRpcClient::new("http://localhost:8899".to_string());

            if !rpc_client.is_validator_running() {
                eprintln!("❌ Error: Validator is not running. Start it with 'cf-solana start'");
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

        Commands::Config => {
            let config = Config::load()?;

            println!("Chain Forge Configuration");
            println!("========================");
            println!();

            if let Some(solana_config) = config.solana {
                println!("Solana:");
                println!("  Default Profile:");
                println!("    RPC URL: {}", solana_config.default.rpc_url);
                println!("    Accounts: {}", solana_config.default.accounts);
                println!(
                    "    Initial Balance: {} SOL",
                    solana_config.default.initial_balance
                );
                println!("    Port: {}", solana_config.default.port);

                if !solana_config.profiles.is_empty() {
                    println!();
                    println!("  Other Profiles:");
                    for (name, profile) in solana_config.profiles {
                        if name != "default" {
                            println!("    {}:", name);
                            println!("      RPC URL: {}", profile.rpc_url);
                        }
                    }
                }
            } else {
                println!("No Solana configuration found.");
                println!("Using defaults: 10 accounts, 100 SOL each, port 8899");
            }

            println!();
            println!("Data Directory: {:?}", Config::data_dir());
        }

        Commands::Stop => {
            println!("Note: Use Ctrl+C to stop the validator running in 'start' mode");
            println!("      This command is for future daemon mode support");
        }
    }

    Ok(())
}
