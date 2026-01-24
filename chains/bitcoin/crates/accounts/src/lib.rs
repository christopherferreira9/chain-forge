use bip39::Mnemonic;
use bitcoin::address::NetworkChecked;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, CompressedPublicKey, Network, PrivateKey};
use chain_forge_common::{ChainError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[cfg(test)]
mod tests;

/// Bitcoin account with keypair and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinAccount {
    /// Bitcoin address (P2WPKH bech32 format, bcrt1... for regtest)
    pub address: String,
    /// Hex-encoded compressed public key
    pub public_key: String,
    /// Private key bytes (32 bytes)
    pub private_key: Vec<u8>,
    /// WIF-encoded private key for wallet import
    pub wif: String,
    /// BIP39 mnemonic phrase used to derive this account
    pub mnemonic: Option<String>,
    /// BIP44 derivation path
    pub derivation_path: Option<String>,
    /// Balance in BTC
    pub balance: f64,
}

impl BitcoinAccount {
    /// Create a new account from a secret key
    pub fn from_secret_key(
        secret_key: SecretKey,
        network: Network,
        mnemonic: Option<String>,
        path: Option<String>,
    ) -> Result<Self> {
        let secp = Secp256k1::new();
        let private_key = PrivateKey::new(secret_key, network);
        let public_key = private_key.public_key(&secp);

        // Create compressed public key for P2WPKH address
        let compressed = CompressedPublicKey(public_key.inner);

        // Create P2WPKH (native SegWit bech32) address
        let address = Address::p2wpkh(&compressed, network);

        Ok(Self {
            address: address.to_string(),
            public_key: public_key.to_string(),
            private_key: secret_key.secret_bytes().to_vec(),
            wif: private_key.to_wif(),
            mnemonic,
            derivation_path: path,
            balance: 0.0,
        })
    }

    /// Get the secret key for this account
    pub fn secret_key(&self) -> Result<SecretKey> {
        SecretKey::from_slice(&self.private_key)
            .map_err(|e| ChainError::AccountGeneration(format!("Invalid secret key bytes: {}", e)))
    }

    /// Get the address as a typed Bitcoin address
    pub fn typed_address(&self, network: Network) -> Result<Address<NetworkChecked>> {
        Address::from_str(&self.address)
            .map_err(|e| ChainError::AccountGeneration(format!("Invalid address: {}", e)))?
            .require_network(network)
            .map_err(|e| ChainError::AccountGeneration(format!("Network mismatch: {}", e)))
    }
}

/// Account generator for Bitcoin using BIP39/BIP44
pub struct AccountGenerator {
    mnemonic: Mnemonic,
    network: Network,
}

impl AccountGenerator {
    /// Create a new generator with a random mnemonic for regtest
    pub fn new() -> Result<Self> {
        Self::new_with_network(Network::Regtest)
    }

    /// Create a new generator with a random mnemonic for the specified network
    pub fn new_with_network(network: Network) -> Result<Self> {
        use rand::RngCore;
        let mut entropy = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy).map_err(|e| {
            ChainError::AccountGeneration(format!("Failed to generate mnemonic: {}", e))
        })?;
        Ok(Self { mnemonic, network })
    }

    /// Create a generator from an existing mnemonic phrase for regtest
    pub fn from_mnemonic(phrase: &str) -> Result<Self> {
        Self::from_mnemonic_with_network(phrase, Network::Regtest)
    }

    /// Create a generator from an existing mnemonic phrase for the specified network
    pub fn from_mnemonic_with_network(phrase: &str, network: Network) -> Result<Self> {
        let mnemonic = Mnemonic::parse(phrase)
            .map_err(|e| ChainError::AccountGeneration(format!("Invalid mnemonic: {}", e)))?;
        Ok(Self { mnemonic, network })
    }

    /// Get the mnemonic phrase
    pub fn mnemonic_phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    /// Get the network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Generate multiple accounts from the mnemonic
    pub fn generate_accounts(&self, count: u32) -> Result<Vec<BitcoinAccount>> {
        let mut accounts = Vec::new();

        for index in 0..count {
            let account = self.derive_account(index)?;
            accounts.push(account);
        }

        Ok(accounts)
    }

    /// Derive a single account at the given index
    /// Uses Bitcoin's standard BIP44 derivation path: m/44'/0'/0'/0/{index}
    /// Note: coin type 0 is for Bitcoin mainnet, but works for regtest too
    pub fn derive_account(&self, index: u32) -> Result<BitcoinAccount> {
        let derivation_path = format!("m/44'/0'/0'/0/{}", index);

        // Get seed from mnemonic
        let seed = self.mnemonic.to_seed("");

        // Create master key from seed
        let secp = Secp256k1::new();
        let master_key = Xpriv::new_master(self.network, &seed).map_err(|e| {
            ChainError::AccountGeneration(format!("Failed to create master key: {}", e))
        })?;

        // Parse derivation path
        let path = DerivationPath::from_str(&derivation_path).map_err(|e| {
            ChainError::AccountGeneration(format!("Invalid derivation path: {}", e))
        })?;

        // Derive child key
        let derived_key = master_key
            .derive_priv(&secp, &path)
            .map_err(|e| ChainError::AccountGeneration(format!("Failed to derive key: {}", e)))?;

        BitcoinAccount::from_secret_key(
            derived_key.private_key,
            self.network,
            Some(self.mnemonic_phrase()),
            Some(derivation_path),
        )
    }
}

impl Default for AccountGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default account generator")
    }
}

/// Accounts storage manager for Bitcoin
pub struct AccountsStorage {
    accounts_file: std::path::PathBuf,
}

impl AccountsStorage {
    /// Create a new storage manager (uses default path under data_dir/bitcoin/accounts.json)
    pub fn new(data_dir: &Path) -> Self {
        let accounts_file = data_dir.join("bitcoin").join("accounts.json");
        Self { accounts_file }
    }

    /// Create a storage manager with a specific file path
    pub fn with_path(accounts_file: std::path::PathBuf) -> Self {
        Self { accounts_file }
    }

    /// Get the accounts file path
    pub fn accounts_file(&self) -> &Path {
        &self.accounts_file
    }

    /// Save accounts to file
    pub fn save(&self, accounts: &[BitcoinAccount]) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.accounts_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(accounts)?;
        std::fs::write(&self.accounts_file, json)?;
        Ok(())
    }

    /// Load accounts from file
    pub fn load(&self) -> Result<Vec<BitcoinAccount>> {
        if !self.accounts_file.exists() {
            return Ok(Vec::new());
        }

        let json = std::fs::read_to_string(&self.accounts_file)?;
        let accounts: Vec<BitcoinAccount> = serde_json::from_str(&json)?;
        Ok(accounts)
    }

    /// Check if accounts file exists
    pub fn exists(&self) -> bool {
        self.accounts_file.exists()
    }

    /// Delete the accounts file
    pub fn delete(&self) -> Result<()> {
        if self.accounts_file.exists() {
            std::fs::remove_file(&self.accounts_file)?;
        }
        Ok(())
    }
}
