# FMP (Forgot My Password)

[![Latest Version](https://img.shields.io/crates/v/forgot-my-password.svg?logo=rust)](https://crates.io/crates/forgot-my-password)
[![GPLv3 License](https://img.shields.io/badge/license-GPLv3-red.svg)](https://codeberg.org/lwilko/fmp/-/blob/main/LICENSE?ref_type=heads)

A password manager written in memory-safe Rust.

## Note on usage of AI:
AI was not used for any sensitive backend components. It was somewhat used for GUI stuff but no issues are found. Some documentation was written by AI, but was rewording my writing - I have dyslexia and my writing is sometimes odd. All AI items have been check and edited to the same extent as my own. All AI code is being reworked in the GUI now I have a better understanding of the GTK library.

## Table of Contents
- [Requirements](#requirements)
- [Quickstart](#quickstart)
- [Usage](#usage)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

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
