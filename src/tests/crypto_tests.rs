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

use crate::crypto::*;
use gpgme::{Context, Protocol};
use std::fs::read_to_string;

const INVALID_RECIPIENT: &str = "invalid_recipient@invalid.recipient";

const DATA: &[u8; 13] = b"test_password";

fn get_valid_recipient() -> String {
    read_to_string("tests/valid_recipient.txt")
        .expect("Failed to read valid recipient from file")
        .trim()
        .to_string()
}

#[test]
fn test_encrypt_variable_success() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = DATA.to_vec();
    let recipient = &get_valid_recipient();

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `src/tests/valid_recipient.txt` to a valid recipient?",
        );

    assert!(encrypted_data.is_empty());
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
    let recipient = &get_valid_recipient();

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `src/tests/valid_recipient.txt` to a valid recipient?",
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
    let recipient = &get_valid_recipient();

    encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `src/tests/valid_recipient.txt` to a valid recipient?",
        );
    assert!(data.iter().all(|&byte| byte == 0),);
}

#[test]
fn test_encrypt_decrypt_empty_data() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = Vec::new();
    let recipient = &get_valid_recipient();

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `src/tests/valid_recipient.txt` to a valid recipient?",
        );

    let decrypted_data = decrypt_variable(&mut ctx, &encrypted_data).unwrap();

    assert!(decrypted_data.is_empty());
}

#[test]
fn test_encrypt_decrypt_large_data() {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut data = vec![b'a'; 10_000];
    let recipient = &get_valid_recipient();

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `src/tests/valid_recipient.txt` to a valid recipient?",
        );

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
    let recipient = &get_valid_recipient();

    let encrypted_data = encrypt_variable(&mut ctx, &mut data, recipient).expect(
            "Failed to encrypt variable. Have you changed `src/tests/valid_recipient.txt` to a valid recipient?",
        );

    let decrypted_data = decrypt_variable(&mut ctx, &encrypted_data).unwrap();

    assert_eq!(decrypted_data, "密码123".as_bytes().to_vec());
}
