use thiserror::Error;

pub type Result<T> = std::result::Result<T, ChainError>;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Account generation error: {0}")]
    AccountGeneration(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Node management error: {0}")]
    NodeManagement(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlParsing(String),

    #[error("Chain not running")]
    NotRunning,

    #[error("Chain already running")]
    AlreadyRunning,

    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ChainError::Config("test config error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test config error");

        let err = ChainError::NotRunning;
        assert_eq!(err.to_string(), "Chain not running");

        let err = ChainError::AlreadyRunning;
        assert_eq!(err.to_string(), "Chain already running");
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let chain_error: ChainError = io_error.into();
        assert!(matches!(chain_error, ChainError::Io(_)));
    }

    #[test]
    fn test_error_from_serde() {
        let json = "{ invalid json }";
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(json);
        let serde_error = result.unwrap_err();
        let chain_error: ChainError = serde_error.into();
        assert!(matches!(chain_error, ChainError::Serialization(_)));
    }

    #[test]
    fn test_result_type() {
        fn returns_ok() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_err() -> Result<String> {
            Err(ChainError::Other("error".to_string()))
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }
}
