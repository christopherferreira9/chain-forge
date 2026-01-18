use super::*;
use tempfile::TempDir;

#[test]
fn test_account_generation() {
    let generator = AccountGenerator::new().unwrap();
    let accounts = generator.generate_accounts(5).unwrap();

    assert_eq!(accounts.len(), 5);

    // All accounts should have unique public keys
    let mut pubkeys = std::collections::HashSet::new();
    for account in &accounts {
        assert!(pubkeys.insert(account.public_key.clone()));
        assert!(account.mnemonic.is_some());
        assert!(account.derivation_path.is_some());
        assert_eq!(account.balance, 0.0);
    }
}

#[test]
fn test_mnemonic_persistence() {
    let generator = AccountGenerator::new().unwrap();
    let mnemonic = generator.mnemonic_phrase();

    // Create new generator from same mnemonic
    let generator2 = AccountGenerator::from_mnemonic(&mnemonic).unwrap();

    // Should generate the same accounts
    let accounts1 = generator.generate_accounts(3).unwrap();
    let accounts2 = generator2.generate_accounts(3).unwrap();

    for (a1, a2) in accounts1.iter().zip(accounts2.iter()) {
        assert_eq!(a1.public_key, a2.public_key);
        assert_eq!(a1.secret_key, a2.secret_key);
    }
}

#[test]
fn test_invalid_mnemonic() {
    let result = AccountGenerator::from_mnemonic("invalid mnemonic phrase");
    assert!(result.is_err());
}

#[test]
fn test_account_keypair_conversion() {
    let generator = AccountGenerator::new().unwrap();
    let account = generator.derive_account(0).unwrap();

    // Should be able to get keypair from account
    let keypair = account.keypair();
    assert!(keypair.is_ok());

    let kp = keypair.unwrap();
    assert_eq!(kp.pubkey().to_string(), account.public_key);
}

#[test]
fn test_account_address() {
    let generator = AccountGenerator::new().unwrap();
    let account = generator.derive_account(0).unwrap();

    assert_eq!(account.address(), account.public_key);
    assert!(!account.address().is_empty());
}

#[test]
fn test_derivation_paths() {
    let generator = AccountGenerator::new().unwrap();
    let accounts = generator.generate_accounts(3).unwrap();

    assert_eq!(
        accounts[0].derivation_path.as_ref().unwrap(),
        "m/44'/501'/0'/0'"
    );
    assert_eq!(
        accounts[1].derivation_path.as_ref().unwrap(),
        "m/44'/501'/1'/0'"
    );
    assert_eq!(
        accounts[2].derivation_path.as_ref().unwrap(),
        "m/44'/501'/2'/0'"
    );
}

#[test]
fn test_accounts_storage_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let storage = AccountsStorage::new(temp_dir.path());

    let generator = AccountGenerator::new().unwrap();
    let accounts = generator.generate_accounts(3).unwrap();

    // Save accounts
    assert!(!storage.exists());
    storage.save(&accounts).unwrap();
    assert!(storage.exists());

    // Load accounts
    let loaded = storage.load().unwrap();
    assert_eq!(loaded.len(), 3);

    for (original, loaded) in accounts.iter().zip(loaded.iter()) {
        assert_eq!(original.public_key, loaded.public_key);
        assert_eq!(original.secret_key, loaded.secret_key);
        assert_eq!(original.mnemonic, loaded.mnemonic);
        assert_eq!(original.derivation_path, loaded.derivation_path);
    }
}

#[test]
fn test_accounts_storage_delete() {
    let temp_dir = TempDir::new().unwrap();
    let storage = AccountsStorage::new(temp_dir.path());

    let generator = AccountGenerator::new().unwrap();
    let accounts = generator.generate_accounts(2).unwrap();

    storage.save(&accounts).unwrap();
    assert!(storage.exists());

    storage.delete().unwrap();
    assert!(!storage.exists());
}

#[test]
fn test_accounts_storage_load_empty() {
    let temp_dir = TempDir::new().unwrap();
    let storage = AccountsStorage::new(temp_dir.path());

    let accounts = storage.load().unwrap();
    assert_eq!(accounts.len(), 0);
}

#[test]
fn test_deterministic_generation() {
    let mnemonic = "test test test test test test test test test test test junk";

    let gen1 = AccountGenerator::from_mnemonic(mnemonic).unwrap();
    let gen2 = AccountGenerator::from_mnemonic(mnemonic).unwrap();

    let accounts1 = gen1.generate_accounts(10).unwrap();
    let accounts2 = gen2.generate_accounts(10).unwrap();

    // Same mnemonic should produce identical accounts
    for (a1, a2) in accounts1.iter().zip(accounts2.iter()) {
        assert_eq!(a1.public_key, a2.public_key);
        assert_eq!(a1.secret_key, a2.secret_key);
    }
}

#[test]
fn test_account_serialization() {
    let generator = AccountGenerator::new().unwrap();
    let account = generator.derive_account(0).unwrap();

    let json = serde_json::to_string(&account).unwrap();
    let deserialized: SolanaAccount = serde_json::from_str(&json).unwrap();

    assert_eq!(account.public_key, deserialized.public_key);
    assert_eq!(account.secret_key, deserialized.secret_key);
    assert_eq!(account.mnemonic, deserialized.mnemonic);
    assert_eq!(account.derivation_path, deserialized.derivation_path);
}

#[test]
fn test_multiple_account_indices() {
    let generator = AccountGenerator::new().unwrap();

    let account_0 = generator.derive_account(0).unwrap();
    let account_5 = generator.derive_account(5).unwrap();
    let account_100 = generator.derive_account(100).unwrap();

    // All should have different keys
    assert_ne!(account_0.public_key, account_5.public_key);
    assert_ne!(account_0.public_key, account_100.public_key);
    assert_ne!(account_5.public_key, account_100.public_key);
}
