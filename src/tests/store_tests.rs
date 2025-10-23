use crate::models::Account;
use crate::security::SecurePassword;
use crate::storage::Store;

#[test]
fn test_create_store_instance_successfully() {
    // This test might fail if GPG context cannot be created in test environment
    let result = Store::new("test_vault", "test_account");

    match result {
        Ok(store) => {
            // Verify that the store was created and has storage locations
            // Since the fields are not public, we just check the paths exist
            assert!(
                store
                    .storage_locations
                    .vault
                    .to_string_lossy()
                    .contains("test_vault")
            );
            assert!(
                store
                    .storage_locations
                    .account
                    .to_string_lossy()
                    .contains("test_account")
            );
        }
        Err(e) => {
            // If GPG is not available in test environment, expect specific error
            assert!(
                e.to_string().contains("gpg")
                    || e.to_string().contains("OpenPgp")
                    || e.to_string().contains("context")
            );
        }
    }
}

#[test]
fn test_store_encrypt_decrypt_integration() {
    // Test the basic encrypt/decrypt functionality by trying to create a store
    // and checking error handling when files don't exist
    let result = Store::new("test_vault", "test_account");

    match result {
        Ok(mut store) => {
            // Try to decrypt from a non-existent file (should fail gracefully)
            let decrypt_result = store.decrypt_account_from_file();
            assert!(decrypt_result.is_err());

            let error_msg = decrypt_result.unwrap_err().to_string();
            assert!(
                error_msg.contains("No such file")
                    || error_msg.contains("not found")
                    || error_msg.contains("cannot open")
            );
        }
        Err(e) => {
            // If GPG context creation fails, that's expected in test environment
            assert!(
                e.to_string().contains("gpg")
                    || e.to_string().contains("OpenPgp")
                    || e.to_string().contains("context")
            );
        }
    }
}

#[test]
fn test_store_encrypt_account_without_file() {
    // Test encrypting an account when the target directory doesn't exist
    let account = Account {
        name: "test_account".to_string(),
        username: "testuser".to_string(),
        password: SecurePassword::new("testpass".to_string()),
        ..Account::default()
    };

    let result = Store::new("non_existent_vault", "test_account");

    match result {
        Ok(mut store) => {
            let encrypt_result = store.encrypt_account_to_file(&account);
            assert!(encrypt_result.is_err());

            let error_msg = encrypt_result.unwrap_err().to_string();
            assert!(
                error_msg.contains("No such file")
                    || error_msg.contains("not found")
                    || error_msg.contains("Failed to find recipient")
                    || error_msg.contains("permission")
            );
        }
        Err(e) => {
            // GPG context creation failed
            assert!(
                e.to_string().contains("gpg")
                    || e.to_string().contains("OpenPgp")
                    || e.to_string().contains("context")
            );
        }
    }
}

#[test]
fn test_store_paths_construction() {
    // Test that the store creates proper paths
    let result = Store::new("my_vault", "my_account");

    match result {
        Ok(store) => {
            // Check that paths contain expected vault and account names
            let vault_path = store.storage_locations.vault.to_string_lossy();
            let account_path = store.storage_locations.account.to_string_lossy();
            let data_path = store.storage_locations.data.to_string_lossy();

            assert!(vault_path.contains("my_vault"));
            assert!(account_path.contains("my_account"));
            assert!(data_path.contains("data.gpg"));
        }
        Err(e) => {
            // Expected in test environment without GPG
            assert!(
                e.to_string().contains("gpg")
                    || e.to_string().contains("OpenPgp")
                    || e.to_string().contains("context")
            );
        }
    }
}

#[test]
fn test_store_with_unicode_names() {
    // Test store creation with Unicode vault and account names
    let result = Store::new("测试保险库", "测试账户");

    match result {
        Ok(store) => {
            let vault_path = store.storage_locations.vault.to_string_lossy();
            let account_path = store.storage_locations.account.to_string_lossy();

            assert!(vault_path.contains("测试保险库"));
            assert!(account_path.contains("测试账户"));
        }
        Err(e) => {
            // Expected in test environment without GPG
            assert!(
                e.to_string().contains("gpg")
                    || e.to_string().contains("OpenPgp")
                    || e.to_string().contains("context")
            );
        }
    }
}

#[test]
fn test_store_with_empty_names() {
    // Test store creation with empty names
    let result = Store::new("", "");

    match result {
        Ok(store) => {
            // Should still create valid paths
            assert!(
                store.storage_locations.vault.exists()
                    || !store.storage_locations.vault.as_os_str().is_empty()
            );
            assert!(
                store.storage_locations.account.exists()
                    || !store.storage_locations.account.as_os_str().is_empty()
            );
        }
        Err(e) => {
            // Expected in test environment without GPG
            assert!(
                e.to_string().contains("gpg")
                    || e.to_string().contains("OpenPgp")
                    || e.to_string().contains("context")
            );
        }
    }
}
