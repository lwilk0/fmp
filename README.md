# FMP (Forgot My Password)

A command-line password manager written in Rust.

Forgot My Password (FMP) is a secure password manager that allows you to create, store, and manage your passwords in encrypted vaults. It uses GPG to ensure that your sensitive data is protected.

![GPLv3 License](https://img.shields.io/badge/license-GPLv3-red.svg)

## Features

- Create encrypted vaults to store your passwords.
- Add, delete, and rename accounts within a vault.
- Generate strong passwords and calculate their entropy.
- Backup and restore vaults securely.
- Change account usernames and passwords.
- Cross-platform support with secure memory handling.

## Installation

1. **Prerequisites**:
   Before installing FMP, make sure the following are installed on your system:
   - [gpgme](https://gpgme.org/) (or [gpg4win](https://www.gpg4win.org/) on Windows)
   - [libgpg-error](https://www.gnupg.org/software/libgpg-error/index.html)
   - [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
   - [rust](https://www.rust-lang.org/tools/install)

2. **Clone the Repository**:
   ```bash
   git clone https://github.com/lwilk0/Forgot-My-Password.git
   cd Forgot-My-Password
   ```

3. **Build and Install FMP**:
   ```bash
   cargo build --release
   cargo install --path .
   ```

4. **Test if FMP is Installed**:
   Run the following command to create a vault:
   ```bash
   fmp -c
   ```

## Usage

### Common Commands

- **Create a New Vault**:
  ```bash
  fmp -c
  ```

- **Add an Account**:
  ```bash
  fmp -a
  ```

- **Generate a Password**:
  ```bash
  fmp -g
  ```

- **Calculate Password Entropy**:
  ```bash
  fmp -e
  ```

- **Backup a Vault**:
  ```bash
  fmp -b
  ```

- **Restore a Backup**:
  ```bash
  fmp -i
  ```

- **Delete a Vault**:
  ```bash
  fmp -D
  ```

For a full list of commands, run:
```bash
fmp --help
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
**Solution**: Ensure `~/.cargo/bin` is added to your PATH:
```bash
export PATH=$PATH:~/.cargo/bin/
```

**Problem**: GPG key not found.  
**Solution**: Ensure the recipient email matches a key in your GPG keyring. Use:
```bash
gpg --list-keys
```

## Contributing

Contributions are welcome! Please follow these steps:
1. Fork this repository.
2. Create a new branch for your feature or bugfix.
3. Submit a pull request with a detailed description of your changes.

## License

This project is licensed under the GPLv3 License. See the [LICENSE](LICENSE) file for details.
