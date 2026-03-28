# FMP (Forgot My Password)

[![Latest Version](https://img.shields.io/crates/v/forgot-my-password.svg?logo=rust)](https://crates.io/crates/forgot-my-password)
[![GPLv3 License](https://img.shields.io/badge/license-GPLv3-red.svg)](https://codeberg.org/lwilko/fmp/-/blob/main/LICENSE?ref_type=heads)

A password manager written in memory-safe Rust.

Forgot My Password (FMP) lets you generate, store, and manage passwords in encrypted vaults. It uses GPG to protect your sensitive data and provides a fast, intuitive GUI.

## Table of Contents
- [Features](#features)
- [Security](#security)
- [Requirements](#requirements)
- [Quickstart](#quickstart)
- [Usage](#usage)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

## Features
- **GUI:** Intuitive and fast GUI.
- **Create Vaults:** Create encrypted vaults to store your passwords.
- **Modify Accounts:** Add, delete, and rename accounts within a vault.
- **Passwords:** Generate strong passwords and estimate their entropy.
- **Backups:** Backup and restore vaults securely.
- **Modify Account Info:** Update account usernames and passwords.
- **Cross-platform compatibility:** FMP is available on Unix and Windows.

## Security
- **Encryption with GPG:** All data is encrypted using GPG. Only users with the correct GPG key can decrypt vault contents.
- **No plaintext passwords on disk:** Sensitive information is always encrypted.
- **Sensitive variables cannot be written to disk:** Secrets are never written while unencrypted.
- **Sensitive variables are obfuscated in memory:** Secure memory handling (Rust’s `secrecy` crate, memory locking) helps prevent secrets from being scraped from RAM.
- **Sensitive variables are cleared from memory:** Memory holding secrets is zeroized when no longer needed.
- **Memory locking:** System calls (like `mlock`) prevent sensitive memory from being swapped to disk.
- **File permissions:** Strict file permissions on sensitive files.
- **Recipient verification:** Encryption is tied to a specific GPG recipient.
- **Cross-platform secure handling:** Secure memory and file handling for both Unix and Windows.

## Requirements
- **Rust toolchain and Cargo**
- **GPGME** and **libgpg-error**
- **GTK4** and **libadwaita** (for the GUI)

See [distribution-specific instructions](https://codeberg.org/lwilko/fmp/wiki/Distro-Specific+Install.-) for OS-specific setup instructions.

## Quickstart
1. **Clone the repository**
   ```bash
   git clone https://codeberg.org/lwilko/fmp.git
   cd fmp
   ```
2. **Build and install**

   Makefile:
   ```bash
      make install-user

      #OR

      sudo make install-system
   ```
   Manual:
   ```bash
   cargo build --release
   cargo install --path .
   export PATH=$PATH:~/.cargo/bin/
   ```

## Usage
- Launch the app with `fmp` (or `cargo run --release` during development).
- Create a new vault or open an existing one.
- Add and manage accounts, generate passwords, and make backups via the GUI.

## Testing
Run all tests:
```bash
cargo test
```

Run specific tests:
```bash
# Filter by module or test name (unit tests live under src/)
cargo test vault_operations_tests
cargo test crypto_tests

# Run a single test with full path
cargo test tests::crypto_tests::test_secure_overwrite_data
```

Note:
- Update `src/tests/recipient.txt` to match a valid recipient in your GPG keyring.

## Troubleshooting
- **`fmp` command not found after installation**
  - Ensure `~/.cargo/bin` is in your PATH:
    ```bash
    export PATH=$PATH:~/.cargo/bin/
    ```
- **GPG key not found in your keyring**
  - Ensure the recipient email matches a key in your keyring:
    ```bash
    gpg --list-keys
    ```

## Contributing
Contributions are welcome! Please:
1. Fork this repository.
2. Create a new branch for your feature or bug fix.
3. Submit a pull request with a clear description of your changes.

## License
This project is licensed under the GPLv3 License. See the [LICENSE](LICENSE) file for details.