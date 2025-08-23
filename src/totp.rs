//! TOTP (Time-based One-Time Password) support for enabling/disabling per-vault 2FA and verifying codes.

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

use anyhow::Error;
use base32::Alphabet;
use gpgme::{Context, Protocol};
use hmac::{Hmac, Mac};
use rand::Rng;
use rand::rng;
use sha1::Sha1;
use std::collections::HashSet;
use std::fs::{File, create_dir_all, remove_file};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::crypto::lock_memory;
use crate::vault::Locations;
use zeroize::Zeroize;

/// HMAC-SHA1 type alias
pub type HmacSha1 = Hmac<Sha1>;

/// Whether a vault has 2FA enabled (presence of the encrypted TOTP secret file).
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
///
/// # Returns:
/// * A `bool` indicating if TOTP is enabled in the specified vault.
pub fn is_totp_enabled(vault_name: &str) -> bool {
    let locations = Locations::new(vault_name, "");
    locations.totp.exists()
}

/// Whether a vault requires TOTP (marker-based, cannot be bypassed by renaming totp.gpg).
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
///
/// # Returns:
/// * A `bool` indicating if TOTP is to be used.
pub fn is_totp_required(vault_name: &str) -> bool {
    let in_ledgers = ledger_contains(vault_name);
    if in_ledgers {
        return true;
    }

    let locations = Locations::new(vault_name, "");
    if locations.totp.exists() {
        let _ = ledger_add(vault_name);
        return true;
    }

    false
}

/// Enable 2FA for a vault.
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
///
/// # Returns:
/// * `Result<(String, String), Error>` - Returns a Base32-encoded secret and an otp URI on success, and an `Error` of failure.
///
/// # Errors:
/// * Fails when unable to: encrypt and store secret, mark a vault as requiring TOTP, check if a gate exists or add a vault to the ledger.
pub fn enable_totp(vault_name: &str) -> Result<(String, String), Error> {
    let locations = Locations::new(vault_name, "");

    let mut secret = [0u8; 20];
    rng().fill(&mut secret);

    encrypt_and_store_secret(&locations, &secret)?;
    ensure_gate_exists(vault_name)?;
    ledger_add(vault_name)?;

    let secret_b32 = base32::encode(Alphabet::Rfc4648 { padding: false }, &secret);

    let issuer = "FMP";
    let label = format!("{issuer}:{vault_name}");
    let otpauth_uri = format!(
        "otpauth://totp/{}?secret={}&issuer={}&period=30&digits=6&algorithm=SHA1",
        urlencoding::encode(&label),
        secret_b32,
        urlencoding::encode(issuer)
    );

    Ok((secret_b32, otpauth_uri))
}

/// Turns off TOTP for a vault
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
///
/// # Returns:
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an `Error` on failure.
///
/// # Errors:
/// * Fails when unable to: remove the `totp.gpg` file, remove TOTP marker or remove vault from the ledger.
pub fn disable_totp(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    if locations.totp.exists() {
        remove_file(&locations.totp)?;
    }
    ledger_remove(vault_name)?;
    Ok(())
}

/// Verify a user-provided 6-digit TOTP code with a tolerance of ±1 time step(30s).
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
/// * `code` - The user imputed 2FA code.
///
/// # Returns:
/// * `Result<bool, Error>` - Returns a `bool` indicating if the code is valid on success, and an `Error` on failure.
///
/// # Errors:
/// * Fails when unable to calculate the time since the Unix Epoch.
pub fn verify_totp_code(vault_name: &str, code: &str) -> Result<bool, Error> {
    let code: String = code.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if code.len() < 6 || code.len() > 8 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Ok(false);
    }

    let mut secret = decrypt_secret(vault_name)?;

    #[allow(clippy::cast_possible_wrap)]
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("System time error: {}", e))?
        .as_secs() as i64;

    let step = 30i64;
    let digits = 6u32;

    let mut valid = false;
    for skew in [-1i64, 0, 1] {
        #[allow(clippy::cast_sign_loss)]
        let counter = ((now / step) + skew) as u64;
        let hotp_val = hotp(&secret, counter, digits);
        let candidate = format!("{:0width$}", hotp_val, width = digits as usize);
        if candidate == code {
            valid = true;
            break;
        }
    }

    secret.zeroize();
    Ok(valid)
}

/// RFC 4226 HOTP calculation using HMAC-SHA1.
///
/// # Arguments:
/// * `secret` - A `&[u8]` slice containing the decrypted `totp.gpg` data.
/// * `counter` - The amount of time steps since the unix epoch, skewed.
/// * `digits` - The length of the TOTP code.
///
/// # Returns:
/// * Numeric OTP for authentication
fn hotp(secret: &[u8], counter: u64, digits: u32) -> u32 {
    let mut counter_bytes = [0u8; 8];
    counter_bytes.copy_from_slice(&counter.to_be_bytes());

    let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0x0f) as usize;
    let bin_code = ((u32::from(result[offset]) & 0x7f) << 24)
        | ((u32::from(result[offset + 1]) & 0xff) << 16)
        | ((u32::from(result[offset + 2]) & 0xff) << 8)
        | (u32::from(result[offset + 3]) & 0xff);

    let modulo = 10u32.pow(digits);
    bin_code % modulo
}

/// Encrypts a secret to `totp.gpg`
///
/// # Arguments:
/// * locations - The `Locations` for the current vault.
/// * secret = The `&[u8]` slice to encrypt
///
/// # Returns:
/// * Result<(), Error> - Returns `Ok(())` on success, or an `Error` on failure.
///
/// # Errors:
/// * Fails when unable to get key for the specified recipient.
fn encrypt_and_store_secret(locations: &Locations, secret: &[u8]) -> Result<(), Error> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    let recipient = std::fs::read_to_string(&locations.recipient)?
        .trim()
        .to_string();
    let recipient_key = ctx.get_key(&recipient).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find recipient `{}` for encryption. Error: {}",
            recipient,
            e
        )
    })?;

    let mut output = Vec::new();
    ctx.encrypt([&recipient_key], secret, &mut output)
        .map_err(|e| anyhow::anyhow!("Failed to encrypt TOTP secret. Error: {}", e))?;

    let mut file = File::create(&locations.totp)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }

    file.write_all(&output)?;
    Ok(())
}

/// Decrypts `totp.gpg`
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
///
/// # Returns:
/// * `Result<Vec<u8>, Error>` - Returns a `Vec<u8>` containing the unencrypted data on success, and an `Error` on failure.
///
/// # Errors:
/// * Fails when unable to: find the `totp.gpg` file for the specified vault, open the `totp.gpg` file or get the gpgme `Context`.
fn decrypt_secret(vault_name: &str) -> Result<Vec<u8>, Error> {
    let locations = Locations::new(vault_name, "");
    if !locations.totp.exists() {
        return Err(anyhow::anyhow!(
            "2FA is not enabled for vault `{}`.",
            vault_name
        ));
    }

    let mut encrypted = Vec::new();
    let mut file = File::open(&locations.totp)?;
    file.read_to_end(&mut encrypted)?;

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let mut out = Vec::new();
    ctx.decrypt(&encrypted, &mut out)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt TOTP secret. Error: {}", e))?;

    lock_memory(&out);
    encrypted.zeroize();

    Ok(out)
}

/// Create a tiny encrypted gate file to trigger GPG passphrase prompt early.
///
/// # Arguments:
/// * `vault_name` - The name of the vault.
///
/// # Returns:
/// * `Result<(), Error>` - `OK(())` on success and an `Error` on failure
///
/// # Errors:
/// * Fails when unable to: get gpgme `Context`, read the `recipient.txt` file, get the key for the recipient, encrypt the `gate.gpg` file, open the `gate.gpg` file, change the `gate.gpg` files permissions or write data to `gate.gpg`.
pub fn ensure_gate_exists(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    if locations.gate.exists() {
        return Ok(());
    }

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let recipient = std::fs::read_to_string(&locations.recipient)?
        .trim()
        .to_string();
    let recipient_key = ctx.get_key(&recipient).map_err(|e| {
        anyhow::anyhow!(
            "Failed to find recipient `{}` for encryption. Error: {}",
            recipient,
            e
        )
    })?;

    let data = b"gate";
    let mut output = Vec::new();
    ctx.encrypt([&recipient_key], &data[..], &mut output)
        .map_err(|e| anyhow::anyhow!("Failed to encrypt gate file. Error: {}", e))?;

    let mut file = File::create(&locations.gate)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(&output)?;
    Ok(())
}
/// Returns the path to the TOTP ledger in the user config directory.
///
/// # Returns:
/// * `PathBuf` - Path to the config-based ledger text file.
fn ledger_path_config() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("fmp").join("totp_ledger")
}

/// Returns the path to the TOTP ledger in the user data directory.
///
/// # Arguments:
/// * None
///
/// # Returns:
/// * `PathBuf` - Path to the data-based ledger text file.
fn ledger_path_data() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("fmp").join("totp_ledger")
}

/// Loads a ledger file containing vault names (one per line).
///
/// # Arguments:
/// * `path` - `&PathBuf`, filesystem path to read from.
///
/// # Returns:
/// * `HashSet<String>` - Set of vault names read from the file (if present).
///
/// # Errors:
/// * Returns empty set if the file or its contents are invalid, but does NOT propagate IO errors.
fn load_ledger_at(path: &PathBuf) -> HashSet<String> {
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(text) = String::from_utf8(bytes) {
            return text
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    HashSet::new()
}

/// Saves the given set of vault names to a ledger file, one per line, sorted.
///
/// # Arguments:
/// * `path` - `&PathBuf`, filesystem path to write to.
/// * `set` - `&HashSet<String>`, set of vault names.
///
/// # Returns:
/// * `Result<(), Error>`
///
/// # Errors:
/// * Returns an error if the file cannot be written or its parent cannot be created.
fn save_ledger_at(path: &PathBuf, set: &HashSet<String>) -> Result<(), Error> {
    if let Some(dir) = path.parent() {
        create_dir_all(dir)?;
    }
    let mut lines: Vec<&str> = set.iter().map(std::string::String::as_str).collect();
    lines.sort_unstable();
    let data = lines.join("\n");
    std::fs::write(path, data)?;
    Ok(())
}

/// Returns the union set of all ledger entries from config and data directories.
///
/// # Returns:
/// * `HashSet<String>` - Set of all vault names required in any ledger found.
fn ledger_union() -> HashSet<String> {
    let mut set = load_ledger_at(&ledger_path_config());
    set.extend(load_ledger_at(&ledger_path_data()));
    set
}

/// Checks if a given vault name is present in any TOTP ledger.
///
/// # Arguments:
/// * `vault` - `&str`, vault name.
///
/// # Returns:
/// * `bool` - True if found.
fn ledger_contains(vault: &str) -> bool {
    ledger_union().contains(vault)
}

/// Adds a vault name to the ledgers in both config and data directories.
///
/// # Arguments:
/// * `vault` - `&str`, vault name to add.
///
/// # Returns:
/// * `Result<(), Error>` - Ok on success, Err for IO errors.
///
/// # Errors:
/// * Returns an error if a ledger file or directory cannot be written/created.
fn ledger_add(vault: &str) -> Result<(), Error> {
    for p in [ledger_path_config(), ledger_path_data()] {
        let mut set = load_ledger_at(&p);
        set.insert(vault.to_string());
        save_ledger_at(&p, &set)?;
    }
    Ok(())
}

/// Removes a vault name from the ledgers in both config and data directories.
///
/// # Arguments:
/// * `vault` - `&str`, vault name to remove.
///
/// # Returns:
/// * `Result<(), Error>` - Ok on success, Err for IO errors.
///
/// # Errors:
/// * Returns an error if a ledger file or directory cannot be written/created.
fn ledger_remove(vault: &str) -> Result<(), Error> {
    for p in [ledger_path_config(), ledger_path_data()] {
        let mut set = load_ledger_at(&p);
        set.remove(vault);
        save_ledger_at(&p, &set)?;
    }
    Ok(())
}
