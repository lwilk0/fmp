use crate::vault::*;
use std::fs::write;
use tempfile::tempdir;

// TODO: change to a valid recipient for testing
const VALID_RECIPIENT: &str = "wilkinsonluke@proton.me";

const VAULT_NAME: &str = "test_vault";
const ACCOUNT_NAME: &str = "test_account";

#[test]
fn test_initialize_vault() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, "null").unwrap();

    let result = locations.initialize_vault();

    assert!(result.is_ok());
    assert!(locations.vault_location.exists());
    assert!(locations.recipient_location.exists());
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

    let locations = Locations::new(&vault_name, ACCOUNT_NAME).unwrap();

    let result = locations.create_account_directory();

    assert!(result.is_ok());
    assert!(locations.account_location.exists());
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

    let locations = Locations::new(&vault_name, "null").unwrap();
    locations.initialize_vault().unwrap();

    let result = Locations::does_vault_exist(&vault_name);

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

    let result = Locations::does_vault_exist(&vault_name);

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

    let locations = Locations::new(&vault_name, "null").unwrap();
    locations.initialize_vault().unwrap();

    let account1 = Locations::new(&vault_name, "account1").unwrap();
    account1.create_account_directory().unwrap();

    let account2 = Locations::new(&vault_name, "account2").unwrap();
    account2.create_account_directory().unwrap();

    let account_names = locations.find_account_names().unwrap();

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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;
    write(&locations.recipient_location, recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(&mut store.ctx, &mut b"test_password".to_vec(), recipient).expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    let decrypted_userpass = store.decrypt_from_file().unwrap();

    assert_eq!(decrypted_userpass.username, "test_user");

    let decrypted_password = decrypt_variable(
        &mut store.ctx,
        decrypted_userpass.password.expose_secret().as_slice(),
    )
    .unwrap();

    assert_eq!(decrypted_password, b"test_password".to_vec());
}

#[test]
fn test_print_vault_entries() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(&mut store.ctx, &mut b"test_password".to_vec(), recipient).expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    let result = crate::vault::print_vault_entries(&vault_name);

    assert!(result.is_ok());
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

    let locations = Locations::new(&vault_name, account_name).unwrap();

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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    write(&locations.data_location, b"corrupted_data").unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
fn test_print_empty_vault() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let locations = Locations::new(&vault_name, "null").unwrap();

    locations.initialize_vault().unwrap();

    let result = crate::vault::print_vault_entries(&vault_name);

    assert!(result.is_ok());
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

    let locations = Locations::new(&vault_name, "null").unwrap();

    locations.initialize_vault().unwrap();

    for i in 0..1000 {
        let account_name = format!("account_{}", i);
        let account_locations = Locations::new(&vault_name, &account_name).unwrap();
        account_locations.create_account_directory().unwrap();
    }

    let account_names = locations.find_account_names().unwrap();

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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    let large_password = vec![b'a'; 10_000];
    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(&mut store.ctx, &mut large_password.clone(), recipient).expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    let decrypted_userpass = store.decrypt_from_file().unwrap();

    let decrypted_password = decrypt_variable(
        &mut store.ctx,
        decrypted_userpass.password.expose_secret().as_slice(),
    )
    .unwrap();

    assert_eq!(decrypted_password, large_password);
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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    write(
        &locations.data_location,
        b"invalid_username:invalid_password",
    )
    .unwrap();

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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(&mut store.ctx, &mut b"test_password".to_vec(), recipient).expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    let metadata = std::fs::metadata(&locations.data_location).unwrap();
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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    let userpass = UserPass {
        username: "用户名".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(
                &mut store.ctx,
                &mut "密码123".as_bytes().to_vec(),
                recipient,
            )
            .expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    let decrypted_userpass = store.decrypt_from_file().unwrap();

    assert_eq!(decrypted_userpass.username, "用户名");

    let decrypted_password = decrypt_variable(
        &mut store.ctx,
        decrypted_userpass.password.expose_secret().as_slice(),
    )
    .unwrap();

    assert_eq!(decrypted_password, "密码123".as_bytes().to_vec());
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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let invalid_recipient = "invalid@recipient.com";
    write(&locations.recipient_location, invalid_recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(
                &mut store.ctx,
                &mut b"test_password".to_vec(),
                invalid_recipient,
            )
            .unwrap_or_else(|_| vec![]),
        )),
    };

    let result = store.encrypt_to_file(userpass);

    assert!(result.is_err());
}

#[test]
fn test_empty_account_name() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let result = Locations::new(&vault_name, "");

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
    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let result = locations.create_account_directory();

    assert!(result.is_ok());
    assert!(locations.account_location.exists());
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

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    write(&locations.data_location, b"invalid:data:format").unwrap();

    let result = store.decrypt_from_file();

    assert!(result.is_err());
}

#[test]
fn test_change_account_username() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    let userpass = UserPass {
        username: "old_username".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(&mut store.ctx, &mut b"test_password".to_vec(), recipient).expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    // Change the username
    store.change_account_username("new_username").unwrap();

    let updated_userpass = store.decrypt_from_file().unwrap();

    assert_eq!(updated_userpass.username, "new_username");
}

#[test]
fn test_change_account_password() {
    let temp_dir = tempdir().unwrap();
    let vault_name = temp_dir
        .path()
        .join(VAULT_NAME)
        .to_str()
        .unwrap()
        .to_string();

    let account_name = ACCOUNT_NAME;
    let mut store = Store::new(&vault_name, account_name).unwrap();

    let locations = Locations::new(&vault_name, account_name).unwrap();

    locations.initialize_vault().unwrap();
    locations.create_account_directory().unwrap();

    let recipient = VALID_RECIPIENT;

    write(&locations.recipient_location, recipient).unwrap();

    let userpass = UserPass {
        username: "test_user".to_string(),
        password: SecretBox::new(Box::new(
            encrypt_variable(&mut store.ctx, &mut b"old_password".to_vec(), recipient).expect("Failed to encrypt password. Have you changed VALID_RECIPIENT to a valid recipient in `src/tests/vault_tests.rs`?"),
        )),
    };

    store.encrypt_to_file(userpass).unwrap();

    // Change the password
    let new_password = SecretBox::new(Box::new(
        encrypt_variable(&mut store.ctx, &mut b"new_password".to_vec(), recipient).unwrap(),
    ));
    store.change_account_password(new_password).unwrap();

    let updated_userpass = store.decrypt_from_file().unwrap();

    let decrypted_password = decrypt_variable(
        &mut store.ctx,
        updated_userpass.password.expose_secret().as_slice(),
    )
    .unwrap();

    assert_eq!(decrypted_password, b"new_password".to_vec());
}
