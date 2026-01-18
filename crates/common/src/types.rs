use serde::{Deserialize, Serialize};

/// Network type for blockchain operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Localnet,
    Devnet,
    Testnet,
    Mainnet,
}

impl Network {
    pub fn as_str(&self) -> &str {
        match self {
            Network::Localnet => "localnet",
            Network::Devnet => "devnet",
            Network::Testnet => "testnet",
            Network::Mainnet => "mainnet",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "localnet" | "local" => Ok(Network::Localnet),
            "devnet" | "dev" => Ok(Network::Devnet),
            "testnet" | "test" => Ok(Network::Testnet),
            "mainnet" | "main" => Ok(Network::Mainnet),
            _ => Err(format!("Invalid network: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_network_as_str() {
        assert_eq!(Network::Localnet.as_str(), "localnet");
        assert_eq!(Network::Devnet.as_str(), "devnet");
        assert_eq!(Network::Testnet.as_str(), "testnet");
        assert_eq!(Network::Mainnet.as_str(), "mainnet");
    }

    #[test]
    fn test_network_display() {
        assert_eq!(Network::Localnet.to_string(), "localnet");
        assert_eq!(Network::Devnet.to_string(), "devnet");
        assert_eq!(Network::Testnet.to_string(), "testnet");
        assert_eq!(Network::Mainnet.to_string(), "mainnet");
    }

    #[test]
    fn test_network_from_str() {
        assert_eq!(Network::from_str("localnet").unwrap(), Network::Localnet);
        assert_eq!(Network::from_str("local").unwrap(), Network::Localnet);
        assert_eq!(Network::from_str("LOCALNET").unwrap(), Network::Localnet);

        assert_eq!(Network::from_str("devnet").unwrap(), Network::Devnet);
        assert_eq!(Network::from_str("dev").unwrap(), Network::Devnet);

        assert_eq!(Network::from_str("testnet").unwrap(), Network::Testnet);
        assert_eq!(Network::from_str("test").unwrap(), Network::Testnet);

        assert_eq!(Network::from_str("mainnet").unwrap(), Network::Mainnet);
        assert_eq!(Network::from_str("main").unwrap(), Network::Mainnet);

        assert!(Network::from_str("invalid").is_err());
    }

    #[test]
    fn test_network_serialization() {
        let network = Network::Devnet;
        let json = serde_json::to_string(&network).unwrap();
        let deserialized: Network = serde_json::from_str(&json).unwrap();
        assert_eq!(network, deserialized);
    }

    #[test]
    fn test_network_equality() {
        assert_eq!(Network::Localnet, Network::Localnet);
        assert_ne!(Network::Localnet, Network::Devnet);
    }

    #[test]
    fn test_network_copy() {
        let network = Network::Mainnet;
        let copied = network;
        assert_eq!(network, copied);
    }
}
