use bip39::Mnemonic;
use chain_forge_common::{ChainError, Result};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use std::path::Path;

#[cfg(test)]
mod tests;

/// Solana account with keypair and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaAccount {
    pub public_key: String,
    pub secret_key: Vec<u8>,
    pub mnemonic: Option<String>,
    pub derivation_path: Option<String>,
    pub balance: f64,
}

impl SolanaAccount {
    /// Create a new account from a keypair
    pub fn from_keypair(keypair: Keypair, mnemonic: Option<String>, path: Option<String>) -> Self {
        Self {
            public_key: keypair.pubkey().to_string(),
            secret_key: keypair.to_bytes().to_vec(),
            mnemonic,
            derivation_path: path,
            balance: 0.0,
        }
    }

    /// Get the keypair for this account
    pub fn keypair(&self) -> Result<Keypair> {
        Keypair::try_from(&self.secret_key[..])
            .map_err(|e| ChainError::AccountGeneration(format!("Invalid keypair bytes: {}", e)))
    }

    /// Get the public key as bs58 string
    pub fn address(&self) -> String {
        self.public_key.clone()
    }
}

/// Account generator for Solana
pub struct AccountGenerator {
    mnemonic: Mnemonic,
}

impl AccountGenerator {
    /// Create a new generator with a random mnemonic
    pub fn new() -> Result<Self> {
        // Generate 16 bytes (128 bits) of entropy for a 12-word mnemonic
        use rand::RngCore;
        let mut entropy = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy).map_err(|e| {
            ChainError::AccountGeneration(format!("Failed to generate mnemonic: {}", e))
        })?;
        Ok(Self { mnemonic })
    }

    /// Create a generator from an existing mnemonic phrase
    pub fn from_mnemonic(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse(phrase)
            .map_err(|e| ChainError::AccountGeneration(format!("Invalid mnemonic: {}", e)))?;
        Ok(Self { mnemonic })
    }

    /// Get the mnemonic phrase
    pub fn mnemonic_phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    /// Generate multiple accounts from the mnemonic
    pub fn generate_accounts(&self, count: u32) -> Result<Vec<SolanaAccount>> {
        let mut accounts = Vec::new();

        for index in 0..count {
            let account = self.derive_account(index)?;
            accounts.push(account);
        }

        Ok(accounts)
    }

    /// Derive a single account at the given index
    /// Uses Solana's standard derivation path: m/44'/501'/index'/0'
    pub fn derive_account(&self, index: u32) -> Result<SolanaAccount> {
        let derivation_path = format!("m/44'/501'/{}'/0'", index);

        // Get seed from mnemonic
        let seed = self.mnemonic.to_seed("");

        // Derive key using BIP44 path for Solana (coin type 501)
        let derived_key = derive_key_from_path(&seed, &derivation_path)?;

        // Create keypair from derived key
        let signing_key = SigningKey::from_bytes(&derived_key);
        let keypair_bytes = signing_key.to_keypair_bytes();
        let keypair = Keypair::try_from(&keypair_bytes[..]).map_err(|e| {
            ChainError::AccountGeneration(format!("Failed to create keypair: {}", e))
        })?;

        Ok(SolanaAccount::from_keypair(
            keypair,
            Some(self.mnemonic_phrase()),
            Some(derivation_path),
        ))
    }
}

impl Default for AccountGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default account generator")
    }
}

/// Derive a key from a BIP44 derivation path
fn derive_key_from_path(seed: &[u8], path: &str) -> Result<[u8; 32]> {
    // Parse the derivation path
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 || parts[0] != "m" {
        return Err(ChainError::AccountGeneration(
            "Invalid derivation path".to_string(),
        ));
    }

    let mut key = seed[..32]
        .try_into()
        .map_err(|_| ChainError::AccountGeneration("Invalid seed length".to_string()))?;
    let mut chain_code = seed[32..64]
        .try_into()
        .map_err(|_| ChainError::AccountGeneration("Invalid seed length".to_string()))?;

    // Derive for each level in the path
    for part in &parts[1..] {
        let index = if let Some(num_str) = part.strip_suffix('\'') {
            let num: u32 = num_str
                .parse()
                .map_err(|_| ChainError::AccountGeneration("Invalid path index".to_string()))?;
            0x80000000 | num // Hardened derivation
        } else {
            part.parse()
                .map_err(|_| ChainError::AccountGeneration("Invalid path index".to_string()))?
        };

        let (new_key, new_chain_code) = derive_child_key(&key, &chain_code, index)?;
        key = new_key;
        chain_code = new_chain_code;
    }

    Ok(key)
}

fn derive_child_key(
    parent_key: &[u8; 32],
    parent_chain_code: &[u8; 32],
    index: u32,
) -> Result<([u8; 32], [u8; 32])> {
    use hmac::Hmac;
    use hmac::Mac;
    use sha2::Sha512;

    let mut hmac = Hmac::<Sha512>::new_from_slice(parent_chain_code)
        .map_err(|e| ChainError::AccountGeneration(format!("HMAC error: {}", e)))?;

    // For hardened keys, use 0x00 || parent_key || index
    if index >= 0x80000000 {
        hmac.update(&[0x00]);
        hmac.update(parent_key);
    } else {
        // For non-hardened, we would use the public key, but ed25519 doesn't support this
        return Err(ChainError::AccountGeneration(
            "Non-hardened derivation not supported for ed25519".to_string(),
        ));
    }

    hmac.update(&index.to_be_bytes());

    let result = hmac.finalize();
    let bytes = result.into_bytes();

    let mut key = [0u8; 32];
    let mut chain_code = [0u8; 32];
    key.copy_from_slice(&bytes[..32]);
    chain_code.copy_from_slice(&bytes[32..64]);

    Ok((key, chain_code))
}

/// Accounts storage manager
pub struct AccountsStorage {
    accounts_file: std::path::PathBuf,
}

impl AccountsStorage {
    /// Create a new storage manager
    pub fn new(data_dir: &Path) -> Self {
        let accounts_file = data_dir.join("solana").join("accounts.json");
        Self { accounts_file }
    }

    /// Save accounts to file
    pub fn save(&self, accounts: &[SolanaAccount]) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.accounts_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(accounts)?;
        std::fs::write(&self.accounts_file, json)?;
        Ok(())
    }

    /// Load accounts from file
    pub fn load(&self) -> Result<Vec<SolanaAccount>> {
        if !self.accounts_file.exists() {
            return Ok(Vec::new());
        }

        let json = std::fs::read_to_string(&self.accounts_file)?;
        let accounts: Vec<SolanaAccount> = serde_json::from_str(&json)?;
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
