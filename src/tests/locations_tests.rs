use crate::storage::locations::Locations;

#[test]
fn test_locations_new() {
    let locations = Locations::new("test_vault", "test_account");

    // Check that paths contain expected components
    assert!(locations.vault.to_string_lossy().contains("test_vault"));
    assert!(locations.account.to_string_lossy().contains("test_account"));
    assert!(locations.fmp.to_string_lossy().contains("fmp"));

    // Check file extensions
    assert_eq!(
        locations.data.extension(),
        Some(std::ffi::OsStr::new("gpg"))
    );
    assert_eq!(
        locations.totp.extension(),
        Some(std::ffi::OsStr::new("gpg"))
    );
    assert_eq!(
        locations.gate.extension(),
        Some(std::ffi::OsStr::new("gpg"))
    );

    // Check that recipient has no extension
    assert_eq!(locations.recipient.extension(), None);
    assert!(locations.recipient.file_name() == Some(std::ffi::OsStr::new("recipient")));
}

#[test]
fn test_locations_path_structure() {
    let locations = Locations::new("my_vault", "my_account");

    // Verify path hierarchy
    assert!(locations.vault.starts_with(&locations.fmp));
    assert!(locations.account.starts_with(&locations.vault));
    assert!(locations.data.starts_with(&locations.account));
    assert!(locations.recipient.starts_with(&locations.vault));
    assert!(locations.totp.starts_with(&locations.vault));
    assert!(locations.gate.starts_with(&locations.vault));
    assert!(locations.backup.starts_with(&locations.fmp));

    // Check that backup is separate from vault
    assert!(!locations.backup.starts_with(&locations.vault));
}

#[test]
fn test_locations_empty_names() {
    let locations = Locations::new("", "");

    // Should handle empty strings without panicking
    assert!(!locations.fmp.to_string_lossy().is_empty());
    assert!(!locations.vault.to_string_lossy().is_empty());
    assert!(!locations.account.to_string_lossy().is_empty());
}

#[test]
fn test_locations_special_characters() {
    let locations = Locations::new("vault-with_special.chars", "account@with+symbols");

    // Should handle special characters in names
    assert!(
        locations
            .vault
            .to_string_lossy()
            .contains("vault-with_special.chars")
    );
    assert!(
        locations
            .account
            .to_string_lossy()
            .contains("account@with+symbols")
    );
}

#[test]
fn test_locations_unicode_names() {
    let locations = Locations::new("金库", "账户");

    // Should handle Unicode names
    assert!(locations.vault.to_string_lossy().contains("金库"));
    assert!(locations.account.to_string_lossy().contains("账户"));
}

#[test]
fn test_does_vault_exist_nonexistent() {
    let locations = Locations::new("nonexistent_vault_12345", "test");
    let result = locations.does_vault_exist();

    // Should return error for non-existent vault
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_does_account_exist_nonexistent() {
    let locations = Locations::new("test", "nonexistent_account_12345");
    let result = locations.does_account_exist();

    // Should return error for non-existent account
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_locations_consistent_naming() {
    let vault_name = "consistent_vault";
    let account_name = "consistent_account";
    let locations1 = Locations::new(vault_name, account_name);
    let locations2 = Locations::new(vault_name, account_name);

    // Same inputs should produce identical paths
    assert_eq!(locations1.vault, locations2.vault);
    assert_eq!(locations1.account, locations2.account);
    assert_eq!(locations1.data, locations2.data);
    assert_eq!(locations1.recipient, locations2.recipient);
    assert_eq!(locations1.totp, locations2.totp);
    assert_eq!(locations1.gate, locations2.gate);
    assert_eq!(locations1.backup, locations2.backup);
}

#[test]
fn test_locations_different_accounts_same_vault() {
    let vault_name = "shared_vault";
    let locations1 = Locations::new(vault_name, "account1");
    let locations2 = Locations::new(vault_name, "account2");

    // Should share vault-level paths but have different account paths
    assert_eq!(locations1.vault, locations2.vault);
    assert_eq!(locations1.recipient, locations2.recipient);
    assert_eq!(locations1.totp, locations2.totp);
    assert_eq!(locations1.gate, locations2.gate);
    assert_eq!(locations1.backup, locations2.backup);

    // But account-specific paths should differ
    assert_ne!(locations1.account, locations2.account);
    assert_ne!(locations1.data, locations2.data);
}

#[test]
fn test_locations_file_names() {
    let locations = Locations::new("test_vault", "test_account");

    // Check specific file names
    assert_eq!(
        locations.data.file_name(),
        Some(std::ffi::OsStr::new("data.gpg"))
    );
    assert_eq!(
        locations.recipient.file_name(),
        Some(std::ffi::OsStr::new("recipient"))
    );
    assert_eq!(
        locations.totp.file_name(),
        Some(std::ffi::OsStr::new("totp.gpg"))
    );
    assert_eq!(
        locations.gate.file_name(),
        Some(std::ffi::OsStr::new("gate.gpg"))
    );
}

#[test]
fn test_locations_base_directory_fallback() {
    // Test behavior when data_dir() might fail
    // This is hard to test directly, but we can verify the structure is consistent
    let locations = Locations::new("test", "test");

    // Verify all paths are absolute or have a consistent base
    let base = &locations.fmp;
    assert!(locations.vault.starts_with(base));
    assert!(locations.backup.starts_with(base));
}
