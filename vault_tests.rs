/*
Copyright (C) 2025  Luke Wilkinson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

use crate::vault::*;
use std::fs::{create_dir, remove_dir, write};
use tempfile::tempdir;

const VAULT_NAME: &str = "test_vault";
const ACCOUNT_NAME: &str = "test_account";
const NULL: &str = "null";

fn get_valid_recipient() -> String {
    read_to_string("src/tests/recipient.txt")
        .expect("Failed to read valid recipient from file")
        .trim()
        .to_string()
}

#[test]
fn test_initialize_vault() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);

    let result = locations.initialize_vault();

    assert!(result.is_ok());
    assert!(locations.vault.exists());
    assert!(locations.recipient.exists());
}

#[test]
fn test_create_account_directory() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, ACCOUNT_NAME);

    let result = locations.create_account_directory();

    assert!(result.is_ok());
    assert!(locations.account.exists());
}

#[test]
fn test_does_vault_exist_success() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);
    locations.initialize_vault().unwrap();

    let result = Locations::does_vault_exist(&locations);

    assert!(result.is_ok());
}

#[test]
fn test_does_vault_exist_failure() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join("nonexistent_vault")
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);

    let result = Locations::does_vault_exist(&locations);

    assert!(result.is_err());
}

#[test]
fn test_find_account_names() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);
    locations.initialize_vault().unwrap();

    let account1 = Locations::new(&vault_name, "account1");
    account1.create_account_directory().unwrap();

    let account2 = Locations::new(&vault_name, "account2");
    account2.create_account_directory().unwrap();

    let account_names = read_directory(&locations.vault).unwrap();

    assert_eq!(account_names.len(), 2);
    assert!(account_names.contains(&"account1".to_string()));
    assert!(account_names.contains(&"account2".to_string()));
}

#[test]
fn test_encrypt_to_file_and_decrypt_from_file() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(b"test_password".to_vec())),
    };

    store.encrypt_to_file(&userpass).unwrap();

    let decrypted_userpass = store.decrypt_from_file().unwrap();

    assert_eq!(decrypted_userpass.username, "test_user");

    assert_eq!(
        *decrypted_userpass.password.expose_secret(),
        b"test_password".to_vec()
    );
}

#[test]
fn test_missing_recipient_file() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
fn test_corrupted_encrypted_data() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    write(&locations.data, b"corrupted_data").unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
fn test_missing_account_directory() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
fn test_large_number_of_accounts() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);

    locations.initialize_vault().unwrap();

    for i in 0..1000 {
        let account_name = format!("account_{i}");
        let accounts = Locations::new(&vault_name, &account_name);
        accounts.create_account_directory().unwrap();
    }

    let account_names = read_directory(&locations.vault).unwrap();

    assert_eq!(account_names.len(), 1000);
}

#[test]
fn test_large_password() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    let large_password = vec![b'a'; 10_000];
    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(large_password.clone())),
    };

    store.encrypt_to_file(&userpass).unwrap();

    let decrypted_userpass = store.decrypt_from_file().unwrap();

    assert_eq!(*decrypted_userpass.password.expose_secret(), large_password);
}

#[test]
fn test_invalid_username_or_password() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    write(&locations.data, b"invalid_username:invalid_password").unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
#[cfg(unix)]
fn test_file_permissions() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(b"test_password".to_vec())),
    };

    store.encrypt_to_file(&userpass).unwrap();

    let metadata = std::fs::metadata(&locations.data).unwrap();
    let permissions = metadata.permissions();

    assert_eq!(permissions.mode() & 0o777, 0o600);
}

#[test]
fn test_utf8_username_and_password() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    let userpass = UserPass {
        username: "用户名".to_string(),
        password: SecretBox::new(Box::new("密码123".as_bytes().to_vec())),
    };

    store.encrypt_to_file(&userpass).unwrap();

    let decrypted_userpass = store.decrypt_from_file().unwrap();

    assert_eq!(decrypted_userpass.username, "用户名");

    assert_eq!(
        *decrypted_userpass.password.expose_secret(),
        "密码123".as_bytes().to_vec()
    );
}

#[test]
fn test_invalid_recipient() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let invalid_recipient = "invalid@recipient.com";
    write(&locations.recipient, invalid_recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(b"test_password".to_vec())),
    };

    let result = store.encrypt_to_file(&userpass);

    assert!(result.is_err());
}

#[test]
fn test_duplicate_account_creation() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let result = locations.create_account_directory();

    assert!(result.is_ok());
    assert!(locations.account.exists());
}

#[test]
fn test_invalid_data_format() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = &get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();

    write(&locations.data, b"invalid:data:format").unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
fn test_rename_directory() {
    let old_path = PathBuf::from("test_old_dir");
    let new_path = PathBuf::from("test_new_dir");

    // Create the old directory
    create_dir(&old_path).expect("Failed to create test_old_dir");

    // Test renaming the directory
    assert!(rename_directory(&old_path, &new_path).is_ok());
    assert!(!old_path.exists());
    assert!(new_path.exists());

    // Cleanup
    remove_dir(&new_path).expect("Failed to remove test_new_dir");
}

#[test]
fn test_rename_directory_nonexistent() {
    let old_path = PathBuf::from("nonexistent_dir");
    let new_path = PathBuf::from("new_dir");

    // Test renaming a nonexistent directory
    let result = rename_directory(&old_path, &new_path);
    assert!(result.is_err());
}

#[test]
fn test_vault_creation_and_existence_check() {
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join(VAULT_NAME);
    let vault_name = vault_path.to_str().unwrap();
    let locations = Locations::new(vault_name, NULL);

    assert!(locations.initialize_vault().is_ok());
    assert!(Locations::does_vault_exist(&locations).is_ok());

    let nonexist_path = temp_dir.path().join("nonexistent_vault");
    let nonexist_name = nonexist_path.to_str().unwrap();
    let nonexist_locations = Locations::new(nonexist_name, NULL);

    assert!(Locations::does_vault_exist(&nonexist_locations).is_err());
}

#[test]
fn test_decrypt_with_corrupted_data_fails() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();
    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = get_valid_recipient();

    write(&locations.recipient, recipient).unwrap();
    write(&locations.data, b"not a valid gpg file").unwrap();

    let result = store.decrypt_from_file();
    assert!(result.is_err());
}

#[test]
fn test_rename_directory_success_and_failure() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let locations = Locations::new(&vault_name, account_name);

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let old_path = locations.account.clone();
    let new_path = old_path.parent().unwrap().join("renamed_account");

    assert!(rename_directory(&old_path, &new_path).is_ok());
    assert!(new_path.exists());
    assert!(!old_path.exists());

    let fake_path = old_path.parent().unwrap().join("does_not_exist");
    let another_new_path = old_path.parent().unwrap().join("should_not_exist");

    assert!(rename_directory(&fake_path, &another_new_path).is_err());
}

#[test]
fn test_read_directory_with_no_subdirs() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);
    locations.initialize_vault().unwrap();

    let subdirs = read_directory(&locations.vault).unwrap();
    assert!(subdirs.is_empty());
}

#[test]
fn test_read_directory_with_multiple_subdirs() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);

    locations.initialize_vault().unwrap();

    let sub1 = locations.vault.join("acc1");
    let sub2 = locations.vault.join("acc2");

    create_dir(&sub1).unwrap();
    create_dir(&sub2).unwrap();

    let mut subdirs = read_directory(&locations.vault).unwrap();
    subdirs.sort();

    let mut expected = vec!["acc1".to_string(), "acc2".to_string()];
    expected.sort();

    assert_eq!(subdirs, expected);
}

#[test]
fn test_read_directory_with_non_utf8_names() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, NULL);
    locations.initialize_vault().unwrap();

    // Create a directory with invalid UTF-8 (on Unix)
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let non_utf8 = locations
            .vault
            .join(std::ffi::OsStr::from_bytes(b"acc_\xFF"));
        create_dir(&non_utf8).unwrap();
        let result = read_directory(&locations.vault);
        assert!(result.is_err());
    }
    #[cfg(not(unix))]
    {
        // On Windows, skip this test as non-UTF8 names are not supported
        assert!(true);
    }
}

#[test]
fn test_multiple_accounts_isolation() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account1 = "account1";
    let account2 = "account2";

    let mut store1 = Store::new(&vault_name, account1).unwrap();
    let mut store2 = Store::new(&vault_name, account2).unwrap();

    let locations1 = Locations::new(&vault_name, account1);
    let locations2 = Locations::new(&vault_name, account2);

    locations1.initialize_vault().unwrap();
    locations1.create_account_directory().unwrap();
    locations2.create_account_directory().unwrap();

    let recipient = get_valid_recipient();

    write(&locations1.recipient, &recipient).unwrap();
    write(&locations2.recipient, &recipient).unwrap();

    let userpass1 = UserPass {
        username: "user1".to_string(),
        password: SecretBox::new(Box::new(b"pass1".to_vec())),
    };

    let userpass2 = UserPass {
        username: "user2".to_string(),
        password: SecretBox::new(Box::new(b"pass2".to_vec())),
    };

    store1.encrypt_to_file(&userpass1).unwrap();
    store2.encrypt_to_file(&userpass2).unwrap();

    let dec1 = store1.decrypt_from_file().unwrap();
    let dec2 = store2.decrypt_from_file().unwrap();

    assert_eq!(dec1.username, "user1");
    assert_eq!(*dec1.password.expose_secret(), b"pass1".to_vec());
    assert_eq!(dec2.username, "user2");
    assert_eq!(*dec2.password.expose_secret(), b"pass2".to_vec());
}

#[test]
fn test_invalid_utf8_in_username_password() {
    let invalid_utf8 = vec![0xff, 0xfe, 0xfd];
    let username = String::from_utf8(invalid_utf8.clone());
    assert!(username.is_err());
}
