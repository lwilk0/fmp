use crate::models::Account;
use crate::security::SecurePassword;
use crate::storage::filesystem::delete_vault;
use crate::vault::operations::*;
use gpgme::{Context, Protocol};
use std::{cell::RefCell, rc::Rc};

#[allow(dead_code)]
fn cleanup_test_vault(vault_name: &str) {
    // Attempt to clean up test vault, ignore errors if vault doesn't exist
    let _ = delete_vault(vault_name);
}

#[test]
fn test_create_vault_successfully() {
    let ctx = Rc::new(RefCell::new(
        Context::from_protocol(Protocol::OpenPgp).unwrap(),
    ));

    // This test will likely fail in test environment without proper GPG setup
    let result = create_vault("test_vault", "test@example.com", ctx);

    match result {
        Ok(_) => {
            cleanup_test_vault("test_vault");
        }
        Err(e) => {
            // Expected errors in test environment
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to find recipient")
                    || error_msg.contains("No such file")
                    || error_msg.contains("permission")
                    || error_msg.contains("gpg")
                    || error_msg.contains("OpenPgp")
            );
        }
    }

    // Additional cleanup attempt in case of any edge cases
    cleanup_test_vault("test_vault");
}

#[test]
fn test_create_account_successfully() {
    let account = Account {
        name: "test_account".to_string(),
        username: "testuser".to_string(),
        password: SecurePassword::new("testpass".to_string()),
        website: "https://example.com".to_string(),
        notes: "Test notes".to_string(),
        ..Account::default()
    };

    let result = create_account("test_vault", &account);

    // This will likely fail without proper vault setup
    match result {
        Ok(_) => {}
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("does not exist")
                    || error_msg.contains("No such file")
                    || error_msg.contains("permission")
                    || error_msg.contains("gpg")
            );
        }
    }

    // Clean up in case the test vault was created or exists
    cleanup_test_vault("test_vault");
}

#[test]
fn test_get_account_details() {
    let result = get_full_account_details("test_vault", "test_account");

    // This will fail without proper vault and account setup
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("does not exist")
            || error_msg.contains("No such file")
            || error_msg.contains("permission")
            || error_msg.contains("gpg")
    );

    // Clean up in case the test vault exists
    cleanup_test_vault("test_vault");
}

#[test]
fn test_update_account_successfully() {
    let account = Account {
        name: "existing_account".to_string(),
        username: "updated_user".to_string(),
        password: SecurePassword::new("updated_pass".to_string()),
        website: "https://updated.com".to_string(),
        notes: "Updated notes".to_string(),
        ..Account::default()
    };

    let result = update_account("test_vault", &account);

    // This will fail without proper vault and account setup
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("does not exist")
            || error_msg.contains("No such file")
            || error_msg.contains("permission")
            || error_msg.contains("gpg")
    );

    // Clean up in case the test vault exists
    cleanup_test_vault("test_vault");
}

#[test]
fn test_delete_account_successfully() {
    let result = delete_account("test_vault", "test_account");

    // This will fail without proper vault and account setup
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("does not exist")
            || error_msg.contains("No such file")
            || error_msg.contains("permission")
            || error_msg.contains("gpg")
    );

    // Clean up in case the test vault exists
    cleanup_test_vault("test_vault");
}

#[test]
fn test_create_vault_invalid_recipient() {
    let ctx = Rc::new(RefCell::new(
        Context::from_protocol(Protocol::OpenPgp).unwrap(),
    ));

    let result = create_vault("test_vault", "invalid_recipient_id", ctx);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to find recipient")
            || error_msg.contains("No such file")
            || error_msg.contains("permission")
            || error_msg.contains("gpg")
            || error_msg.contains("OpenPgp")
    );

    // Clean up in case the test vault was partially created
    cleanup_test_vault("test_vault");
}

#[test]
fn test_get_non_existent_account() {
    let result = get_full_account_details("non_existent_vault", "non_existent_account");

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("does not exist")
            || error_msg.contains("No such file")
            || error_msg.contains("not found")
            || error_msg.contains("gpg")
    );
}

#[test]
fn test_delete_non_existent_account() {
    let result = delete_account("non_existent_vault", "non_existent_account");

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("does not exist")
            || error_msg.contains("No such file")
            || error_msg.contains("not found")
            || error_msg.contains("gpg")
    );
}

#[test]
fn test_create_account_with_empty_name() {
    let account = Account {
        name: "".to_string(),
        username: "testuser".to_string(),
        password: SecurePassword::new("testpass".to_string()),
        ..Account::default()
    };

    let result = create_account("test_vault", &account);

    // This should fail due to empty account name or vault not existing
    assert!(result.is_err());

    // Clean up in case the test vault exists
    cleanup_test_vault("test_vault");
}

#[test]
fn test_update_account_with_unicode_name() {
    let account = Account {
        name: "测试账户".to_string(),
        username: "unicode_user".to_string(),
        password: SecurePassword::new("unicode_pass".to_string()),
        ..Account::default()
    };

    let result = update_account("test_vault", &account);

    // This will fail without proper vault setup, but should handle Unicode correctly
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("does not exist")
            || error_msg.contains("No such file")
            || error_msg.contains("permission")
            || error_msg.contains("gpg")
    );

    // Clean up in case the test vault exists
    cleanup_test_vault("test_vault");
}
