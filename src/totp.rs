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

use crate::vault::Locations;

/// HMAC-SHA1 type alias
pub type HmacSha1 = Hmac<Sha1>;

/// Whether a vault has 2FA enabled (presence of the encrypted TOTP secret file).
pub fn is_totp_enabled(vault_name: &str) -> bool {
    let locations = Locations::new(vault_name, "");
    locations.totp.exists()
}

/// Whether a vault requires TOTP (marker-based, cannot be bypassed by renaming totp.gpg).
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

/// Enable 2FA for a vault: generate a new secret, store it encrypted, and return
/// (`base32_secret`, `otpauth_uri`) for enrolling in Authy/Aegis/etc.
pub fn enable_totp(vault_name: &str) -> Result<(String, String), Error> {
    let locations = Locations::new(vault_name, "");

    let mut secret = [0u8; 20];
    rng().fill(&mut secret);

    encrypt_and_store_secret(&locations, &secret)?;
    require_totp(vault_name)?;
    ensure_gate_exists(vault_name)?;
    ledger_add(vault_name)?;

    let secret_b32 = base32::encode(Alphabet::RFC4648 { padding: false }, &secret);

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

pub fn disable_totp(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    if locations.totp.exists() {
        remove_file(&locations.totp)?;
    }
    unrequire_totp(vault_name)?;
    ledger_remove(vault_name)?;
    Ok(())
}

/// Verify a user-provided 6-digit TOTP code with a tolerance of ±1 time step.
pub fn verify_totp_code(vault_name: &str, code: &str) -> Result<bool, Error> {
    let code = code.trim();
    if code.len() < 6 || code.len() > 8 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Ok(false);
    }

    let secret = decrypt_secret(vault_name)?;

    #[allow(clippy::cast_possible_wrap)]
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("System time error: {}", e))?
        .as_secs() as i64;

    let step = 30i64;
    let digits = 6u32;

    for skew in [-1i64, 0, 1] {
        #[allow(clippy::cast_sign_loss)]
        let counter = ((now / step) + skew) as u64;
        let hotp_val = hotp(&secret, counter, digits);
        let candidate = format!("{:0width$}", hotp_val, width = digits as usize);
        if candidate == code {
            return Ok(true);
        }
    }

    Ok(false)
}

/// RFC 4226 HOTP calculation using HMAC-SHA1.
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

fn encrypt_and_store_secret(locations: &Locations, secret: &[u8]) -> Result<(), Error> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;

    let recipient = std::fs::read_to_string(&locations.recipient)?;
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

    Ok(out)
}

/// Create a tiny encrypted gate file to trigger GPG passphrase prompt early.
pub fn ensure_gate_exists(vault_name: &str) -> Result<(), Error> {
    let locations = Locations::new(vault_name, "");
    if locations.gate.exists() {
        return Ok(());
    }

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
    let recipient = std::fs::read_to_string(&locations.recipient)?;
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

/// Compute the path to the external TOTP-required marker in the user config directory.
fn required_marker_path(vault_name: &str) -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("fmp").join("totp_required").join(vault_name)
}

/// Mark a vault as requiring TOTP by creating a marker file in the config directory.
pub fn require_totp(vault_name: &str) -> Result<(), Error> {
    let marker = required_marker_path(vault_name);
    if let Some(dir) = marker.parent() {
        create_dir_all(dir)?;
    }
    if !marker.exists() {
        File::create(&marker)?;
    }
    Ok(())
}

/// Remove the TOTP requirement marker.
pub fn unrequire_totp(vault_name: &str) -> Result<(), Error> {
    let marker = required_marker_path(vault_name);
    if marker.exists() {
        remove_file(&marker)?;
    }
    Ok(())
}

/// Simple newline-based ledgers in both config and data dirs for redundancy
fn ledger_path_config() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("fmp").join("totp_ledger")
}
fn ledger_path_data() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("fmp").join("totp_ledger")
}

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

fn ledger_union() -> HashSet<String> {
    let mut set = load_ledger_at(&ledger_path_config());
    set.extend(load_ledger_at(&ledger_path_data()));
    set
}

fn ledger_contains(vault: &str) -> bool {
    ledger_union().contains(vault)
}

fn ledger_add(vault: &str) -> Result<(), Error> {
    for p in [ledger_path_config(), ledger_path_data()] {
        let mut set = load_ledger_at(&p);
        set.insert(vault.to_string());
        save_ledger_at(&p, &set)?;
    }
    Ok(())
}

fn ledger_remove(vault: &str) -> Result<(), Error> {
    for p in [ledger_path_config(), ledger_path_data()] {
        let mut set = load_ledger_at(&p);
        set.remove(vault);
        save_ledger_at(&p, &set)?;
    }
    Ok(())
}
