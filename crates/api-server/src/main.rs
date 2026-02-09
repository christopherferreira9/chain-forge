//! Chain Forge API Server CLI
//!
//! Starts the REST API server for the Chain Forge web dashboard.

use chain_forge_api_server::start_server;
use clap::Parser;
use eyre::Result;

#[derive(Parser)]
#[command(name = "cf-api")]
#[command(about = "Chain Forge REST API Server", long_about = None)]
#[command(version)]
struct Cli {
    /// Port to run the API server on
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Open API documentation in browser after starting
    #[arg(short, long, default_value = "false")]
    open: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.open {
        // Try to open the dashboard URL in the default browser
        let url = format!("http://localhost:{}", cli.port);
        println!("🌐 Opening dashboard at {}...", url);

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(&url).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", &url])
                .spawn();
        }
    }

    start_server(cli.port).await
}
