use crate::crypto::*;
use gpgme::{Context, Protocol};

// TODO: Replace with a valid recipient in your keyring
const VALID_RECIPIENT: &str = "wilkinsonluke@proton.me";
const INVALID_RECIPIENT: &str = "invalid_recipient@invalid.recipient";

const DATA: &'static [u8; 13] = b"test_password";

#[test]
fn test_encrypt_variable_success() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = DATA.to_vec();
    let recipient = VALID_RECIPIENT;

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `VALID_RECIPIENT` to a valid recipient in `src/tests/crypto_tests.rs`?",
        );

    assert!(encrypted_data.len() > 0);
}

#[test]
fn test_encrypt_variable_invalid_recipient() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = DATA.to_vec();
    let recipient = INVALID_RECIPIENT;

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient);
    assert!(encrypted_data.is_err());
}

#[test]
fn test_decrypt_variable_success() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = DATA.to_vec();
    let recipient = VALID_RECIPIENT;

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `VALID_RECIPIENT` to a valid recipient in `src/tests/crypto_tests.rs`?",
        );
    let decrypted_data = decrypt_variable(&mut ctx, &encrypted_data);

    assert!(decrypted_data.is_ok());
    assert_eq!(decrypted_data.unwrap(), DATA.to_vec());
}

#[test]
fn test_decrypt_variable_invalid_data() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let invalid_data = DATA;

    let decrypted_data = decrypt_variable(&mut ctx, invalid_data);
    assert!(decrypted_data.is_err());
}

#[test]
fn test_memory_safety() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = DATA.to_vec();
    let recipient = VALID_RECIPIENT;

    encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `VALID_RECIPIENT` to a valid recipient in `src/tests/crypto_tests.rs`?",
        );
    assert!(data.iter().all(|&byte| byte == 0),);
}

#[test]
fn test_encrypt_decrypt_empty_data() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = Vec::new();
    let recipient = VALID_RECIPIENT;

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).unwrap();
    let decrypted_data = decrypt_variable(&mut ctx, &encrypted_data).unwrap();

    assert!(decrypted_data.is_empty());
}

#[test]
fn test_encrypt_decrypt_large_data() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = vec![b'a'; 10_000];
    let recipient = VALID_RECIPIENT;

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).unwrap();
    let decrypted_data = decrypt_variable(&mut ctx, &encrypted_data).unwrap();

    assert_eq!(decrypted_data, vec![b'a'; 10_000]);
}

#[test]
fn test_decrypt_variable_corrupted_data() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let corrupted_data = vec![0, 1, 2, 3, 4, 5];

    let decrypted_data = decrypt_variable(&mut ctx, &corrupted_data);
    assert!(decrypted_data.is_err());
}

#[test]
fn test_encrypt_decrypt_utf8_password() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = "密码123".as_bytes().to_vec();
    let recipient = VALID_RECIPIENT;

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).unwrap();
    let decrypted_data = decrypt_variable(&mut ctx, &encrypted_data).unwrap();

    assert_eq!(decrypted_data, "密码123".as_bytes().to_vec());
}
