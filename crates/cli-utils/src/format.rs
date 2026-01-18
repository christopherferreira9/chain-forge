use clap::ValueEnum;
use serde::Serialize;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

/// Format accounts for display
pub fn format_accounts<T: Serialize>(accounts: &[T], format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(accounts)
            .unwrap_or_else(|e| format!("Error formatting JSON: {}", e)),
        OutputFormat::Table => {
            // For table format, we'll use the JSON representation
            // Individual chain implementations can provide better table formatting
            serde_json::to_string_pretty(accounts)
                .unwrap_or_else(|e| format!("Error formatting: {}", e))
        }
    }
}
