use super::*;
use tempfile::tempdir;

const TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn test_account_generation() {
    let generator = AccountGenerator::new().unwrap();
    let accounts = generator.generate_accounts(5).unwrap();

    assert_eq!(accounts.len(), 5);

    // Verify all accounts have unique addresses
    let mut addresses = std::collections::HashSet::new();
    for account in &accounts {
        assert!(addresses.insert(account.address.clone()));
        // Regtest addresses start with bcrt1 for P2WPKH
        assert!(
            account.address.starts_with("bcrt1"),
            "Expected bcrt1 prefix, got: {}",
            account.address
        );
    }
}

#[test]
fn test_deterministic_derivation() {
    let generator1 = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();
    let generator2 = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();

    let accounts1 = generator1.generate_accounts(3).unwrap();
    let accounts2 = generator2.generate_accounts(3).unwrap();

    for i in 0..3 {
        assert_eq!(accounts1[i].address, accounts2[i].address);
        assert_eq!(accounts1[i].public_key, accounts2[i].public_key);
        assert_eq!(accounts1[i].private_key, accounts2[i].private_key);
        assert_eq!(accounts1[i].wif, accounts2[i].wif);
    }
}

#[test]
fn test_derivation_paths() {
    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();

    for i in 0..5 {
        let account = generator.derive_account(i).unwrap();
        let expected_path = format!("m/44'/0'/0'/0/{}", i);
        assert_eq!(account.derivation_path.unwrap(), expected_path);
    }
}

#[test]
fn test_mnemonic_persistence() {
    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();
    assert_eq!(generator.mnemonic_phrase(), TEST_MNEMONIC);

    let account = generator.derive_account(0).unwrap();
    assert_eq!(account.mnemonic.unwrap(), TEST_MNEMONIC);
}

#[test]
fn test_invalid_mnemonic() {
    let result = AccountGenerator::from_mnemonic("invalid mnemonic phrase");
    assert!(result.is_err());
}

#[test]
fn test_secret_key_recovery() {
    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();
    let account = generator.derive_account(0).unwrap();

    let secret_key = account.secret_key().unwrap();
    assert_eq!(secret_key.secret_bytes().to_vec(), account.private_key);
}

#[test]
fn test_wif_format() {
    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();
    let account = generator.derive_account(0).unwrap();

    // Regtest WIF keys start with 'c' (compressed) or '9' (uncompressed)
    // Since we use compressed keys, it should start with 'c'
    assert!(
        account.wif.starts_with('c'),
        "Expected WIF to start with 'c' for regtest compressed key, got: {}",
        account.wif
    );
}

#[test]
fn test_storage_save_load() {
    let temp_dir = tempdir().unwrap();
    let storage = AccountsStorage::new(temp_dir.path());

    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();
    let accounts = generator.generate_accounts(3).unwrap();

    // Save and load
    storage.save(&accounts).unwrap();
    assert!(storage.exists());

    let loaded = storage.load().unwrap();
    assert_eq!(loaded.len(), accounts.len());

    for i in 0..accounts.len() {
        assert_eq!(loaded[i].address, accounts[i].address);
        assert_eq!(loaded[i].public_key, accounts[i].public_key);
        assert_eq!(loaded[i].private_key, accounts[i].private_key);
        assert_eq!(loaded[i].wif, accounts[i].wif);
    }
}

#[test]
fn test_storage_delete() {
    let temp_dir = tempdir().unwrap();
    let storage = AccountsStorage::new(temp_dir.path());

    let generator = AccountGenerator::new().unwrap();
    let accounts = generator.generate_accounts(1).unwrap();

    storage.save(&accounts).unwrap();
    assert!(storage.exists());

    storage.delete().unwrap();
    assert!(!storage.exists());
}

#[test]
fn test_storage_empty_load() {
    let temp_dir = tempdir().unwrap();
    let storage = AccountsStorage::new(temp_dir.path());

    let loaded = storage.load().unwrap();
    assert!(loaded.is_empty());
}

#[test]
fn test_account_serialization() {
    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();
    let account = generator.derive_account(0).unwrap();

    let json = serde_json::to_string(&account).unwrap();
    let deserialized: BitcoinAccount = serde_json::from_str(&json).unwrap();

    assert_eq!(account.address, deserialized.address);
    assert_eq!(account.public_key, deserialized.public_key);
    assert_eq!(account.private_key, deserialized.private_key);
    assert_eq!(account.wif, deserialized.wif);
}

#[test]
fn test_different_networks() {
    let regtest_gen = AccountGenerator::new_with_network(bitcoin::Network::Regtest).unwrap();
    let regtest_account = regtest_gen.derive_account(0).unwrap();

    // Regtest P2WPKH addresses start with bcrt1
    assert!(regtest_account.address.starts_with("bcrt1"));

    // Regtest WIF starts with 'c' for compressed keys
    assert!(regtest_account.wif.starts_with('c'));
}

#[test]
fn test_multiple_account_indices() {
    let generator = AccountGenerator::from_mnemonic(TEST_MNEMONIC).unwrap();

    // Test various indices
    let indices = [0, 1, 10, 100, 999];
    let mut addresses = std::collections::HashSet::new();

    for &index in &indices {
        let account = generator.derive_account(index).unwrap();
        assert!(addresses.insert(account.address.clone()));
        assert_eq!(
            account.derivation_path.unwrap(),
            format!("m/44'/0'/0'/0/{}", index)
        );
    }
}
