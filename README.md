# FMP (Forgot My Password)

[![Latest Version](https://img.shields.io/crates/v/forgot-my-password.svg?logo=rust)](https://crates.io/crates/forgot-my-password)
[![GPLv3 License](https://img.shields.io/badge/license-GPLv3-red.svg)](https://codeberg.org/lwilko/fmp/-/blob/main/LICENSE?ref_type=heads)

A password manager written in memory-safe Rust.

Forgot My Password (FMP) is a password manager that safely allows you to generate, store, and manage your passwords in encrypted vaults. It uses GPG to protect your sensitive data.

## Features
- **GUI:** Intuitive and fast GUI
- **Create Vaults:** Create encrypted vaults to store your passwords.
- **Modify Accounts:** Add, delete, and rename accounts within a vault.
- **Passwords:** Generate strong passwords and estimate their entropy.
- **Backups:** Backup and restore vaults securely.
- **Modify Account Info:** Update account usernames and passwords.
- **Cross-platform compatibility:** FMP is available on Unix and Windows

## Security
- **Encryption With GPG:** All data is encrypted using GPG. Only users with the correct GPG key can decrypt the vault contents.
- **No Plaintext Passwords on Disk:** All sensitive information is encrypted before being saved.
- **Sensitive Variables Cannot Be Written to Disk:** Sensitive variables are not written to disk in any form except encrypted.
- **Sensitive Variables Are Obfuscated in Memory:** The program uses secure memory handling (Rust’s secrecy crate, memory locking) to prevent secrets from being easily read from RAM.
- **Sensitive Variables Are Cleared from Memory:** The program zeroizes (overwrites) memory holding secrets when they are no longer needed.
- **Memory Locking:** System calls (like mlock) prevent sensitive memory from being swapped to disk.
- **File Permitions:** Strict file permissions are placed on sensitive files.
- **Recipient Verification:** Encryption is tied to a specific GPG recipient.
- **Cross-Platform Secure Handling:** Secure memory and file handling are implemented for both Unix and Windows.

## Installation

1. **Prerequisites**:
   Before installing FMP, make sure the following are installed on your system:
   - [gpgme](https://gpgme.org/)
   - [libgpg-error](https://www.gnupg.org/software/libgpg-error/index.html)
   - [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
   - [rust](https://www.rust-lang.org/tools/install)

  See [INSTALLATION.md](INSTALLATION.md) for OS specific installations.
  
2. **Clone the Repository**:
   ```bash
   git clone https://codeberg.org/lwilko/fmp.git
   cd fmp
   ```

3. **Build and Install FMP**:
   ```bash
   cargo build --release
   cargo install --path .
   ```

## Testing

Run all tests:
```bash
cargo test
```

Run specific tests:
```bash
cargo test --test vault_tests
cargo test --test crypto_tests
```

**Note**: Update the file in `src/tests/recipient.txt` to match a valid recipient in your GPG keyring.

## Troubleshooting

**Problem**: `fmp` command not found after installation.  
**Solution**: Make sure `~/.cargo/bin` is added to your PATH:
```bash
export PATH=$PATH:~/.cargo/bin/
```

**Problem**: GPG key not found in your keyring.  
**Solution**: Make sure the recipient email matches a key in your GPG keyring. Use:
```bash
gpg --list-keys
```

## Contributing

Contributions are welcome! Please follow these steps to contribute:
1. Fork this repository.
2. Create a new branch for your feature or bug-fix.
3. Submit a pull request with a detailed description of your changes.

## License

This project is licensed under the GPLv3 License. See the [LICENSE](LICENSE) file for details.
