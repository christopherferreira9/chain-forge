# chain-forge-cli-utils

CLI utilities and formatting helpers for Chain Forge command-line tools.

## Overview

Provides common utilities for building consistent CLI experiences across all Chain Forge blockchain implementations.

## Features

- **Output Formatting**: JSON and table output formats
- **Common CLI Patterns**: Shared CLI argument types
- **Consistent UX**: Standardized output across all chains

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
chain-forge-cli-utils = { path = "../../crates/cli-utils" }
clap = { version = "4", features = ["derive"] }
```

### Output Formatting

```rust
use chain_forge_cli_utils::{OutputFormat, format_accounts};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,
}

fn main() {
    let args = Args::parse();

    let accounts = vec![/* your accounts */];
    let output = format_accounts(&accounts, args.format);
    println!("{}", output);
}
```

## API Reference

### `OutputFormat` Enum

Output format selector for CLI commands.

```rust
pub enum OutputFormat {
    Json,   // JSON output
    Table,  // Table output
}
```

Use with clap:

```rust
use clap::Parser;
use chain_forge_cli_utils::OutputFormat;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,
}
```

### `format_accounts` Function

Format accounts for display.

```rust
pub fn format_accounts<T: Serialize>(
    accounts: &[T],
    format: OutputFormat
) -> String
```

**Parameters:**
- `accounts` - Slice of serializable accounts
- `format` - Output format (JSON or Table)

**Returns:** Formatted string ready for display

**Example:**

```rust
use serde::Serialize;
use chain_forge_cli_utils::{format_accounts, OutputFormat};

#[derive(Serialize)]
struct Account {
    address: String,
    balance: f64,
}

let accounts = vec![
    Account { address: "abc123".to_string(), balance: 100.0 },
    Account { address: "def456".to_string(), balance: 50.0 },
];

// JSON output
let json_output = format_accounts(&accounts, OutputFormat::Json);
println!("{}", json_output);

// Table output
let table_output = format_accounts(&accounts, OutputFormat::Table);
println!("{}", table_output);
```

## Integration with Clap

This crate is designed to work seamlessly with [clap](https://docs.rs/clap):

```rust
use clap::{Parser, Subcommand};
use chain_forge_cli_utils::OutputFormat;

#[derive(Parser)]
#[command(name = "my-chain-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Accounts {
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Accounts { format } => {
            // Use format...
        }
    }
}
```

## Future Enhancements

Planned features:
- Color output support
- Progress bars for long operations
- Interactive prompts
- Spinner animations
- More table formatting options

## Dependencies

- `clap` - CLI argument parsing
- `serde` - Serialization
- `serde_json` - JSON formatting
- `tabled` - Table formatting

## License

MIT OR Apache-2.0
