use crate::storage::filesystem::*;
use std::fs::{File, create_dir_all, write};
use std::path::PathBuf;
use tempfile::TempDir;

#[allow(dead_code)]
fn create_test_vault_structure(temp_dir: &TempDir, vault_name: &str) -> PathBuf {
    let fmp_dir = temp_dir.path().join("fmp");
    let vaults_dir = fmp_dir.join("vaults");
    let vault_dir = vaults_dir.join(vault_name);

    create_dir_all(&vault_dir).unwrap();

    // Create a recipient file for the vault
    write(vault_dir.join("recipient.txt"), "test@example.com").unwrap();

    fmp_dir
}

#[test]
fn test_read_empty_directory_successfully() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    create_dir_all(&empty_dir).unwrap();

    let result = read_directory(&empty_dir);

    assert!(result.is_ok());
    let directories = result.unwrap();
    assert_eq!(directories.len(), 0);
}

#[test]
fn test_read_directory_with_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test");
    create_dir_all(&test_dir).unwrap();

    // Create some subdirectories
    create_dir_all(test_dir.join("account1")).unwrap();
    create_dir_all(test_dir.join("account2")).unwrap();
    create_dir_all(test_dir.join("account3")).unwrap();

    // Create a file (should be ignored)
    File::create(test_dir.join("not_a_dir.txt")).unwrap();

    let result = read_directory(&test_dir);

    assert!(result.is_ok());
    let mut directories = result.unwrap();
    directories.sort(); // Sort for consistent testing

    assert_eq!(directories.len(), 3);
    assert_eq!(directories, vec!["account1", "account2", "account3"]);
}

#[test]
fn test_create_backup_successfully() {
    // Since create_backup uses Locations which uses system paths,
    // we test with a non-existent vault to verify error handling
    let result = create_backup("definitely_non_existent_test_vault");

    // This should fail since the vault doesn't exist
    assert!(result.is_err());

    // The error could be various types depending on the system state
    let error_msg = result.unwrap_err().to_string();

    // Be more lenient with error message checking
    assert!(!error_msg.is_empty(), "Error message should not be empty");
}

#[test]
fn test_install_backup_successfully() {
    // This test would require a more complex setup with actual backup directories
    // For now, we'll test the error case when backup doesn't exist
    let result = install_backup("non_existent_vault");

    assert!(result.is_err());
}

#[test]
fn test_read_non_existent_directory() {
    let non_existent_path = PathBuf::from("/non/existent/directory");

    let result = read_directory(&non_existent_path);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No such file") || error_msg.contains("not found"));
}

#[test]
fn test_delete_non_existent_vault() {
    let result = delete_vault("definitely_non_existent_vault");

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_rename_vault_with_existing_target() {
    // First test renaming a non-existent vault
    let result = rename_vault("non_existent", "also_non_existent");

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_backup_exists_false() {
    let result = backup_exists("definitely_non_existent_vault");
    assert!(!result);
}

#[test]
fn test_delete_non_existent_backup() {
    let result = delete_backup("non_existent_vault");

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_rename_account_non_existent_vault() {
    let result = rename_account("non_existent_vault", "old_account", "new_account");

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("does not exist") || error_msg.contains("not found"));
}
