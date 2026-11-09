use crate::paths::FileLock;
use crate::vault_session::UnlockedVaultKeys;
use crate::{account::Account, crypto::VaultDecryptHelper};

use anyhow::{Context, Result, bail};

use sequoia_openpgp::{
    Cert,
    crypto::KeyPair,
    parse::{Parse, stream::DecryptorBuilder},
    policy::StandardPolicy,
    serialize::stream::{Encryptor, LiteralWriter, Message, Signer},
};

use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use uuid::Uuid;
use zeroize::Zeroizing;

const MAX_CIPHERTEXT_SIZE: u64 = 8 * 1024 * 1024;
const MAX_PLAINTEXT_SIZE: usize = 4 * 1024 * 1024;

/// Handles accounts
pub(crate) struct AccountStore {
    vault_path: PathBuf,
}

impl AccountStore {
    pub(crate) fn new(vault_path: impl Into<PathBuf>) -> Self {
        Self {
            vault_path: vault_path.into(),
        }
    }

    fn accounts_dir(&self) -> PathBuf {
        self.vault_path.join("accounts")
    }

    fn account_path(&self, id: Uuid) -> PathBuf {
        self.accounts_dir().join(format!("{id}.pgp"))
    }

    pub(crate) fn create_with_keys(
        &self,
        account: &Account,
        keys: &UnlockedVaultKeys,
    ) -> Result<()> {
        account.validate()?;

        let _lock = FileLock::exclusive(&self.vault_path.join(".vault.lock"))?;

        self.write_account_locked(
            account,
            &keys.public_cert,
            keys.signing_keypair.clone(),
            false,
        )
    }

    pub(crate) fn update_with_keys(
        &self,
        account: &mut Account,
        keys: &UnlockedVaultKeys,
    ) -> Result<()> {
        account.update_modified_time();
        account.validate()?;

        let _lock = FileLock::exclusive(&self.vault_path.join(".vault.lock"))?;

        self.write_account_locked(
            account,
            &keys.public_cert,
            keys.signing_keypair.clone(),
            true,
        )
    }

    /// Encrypts the account and puts it in the vault
    fn write_account_locked(
        &self,
        account: &Account,
        public_cert: &Cert,
        signing_keypair: KeyPair,
        replace: bool,
    ) -> Result<()> {
        let accounts_dir = self.accounts_dir();
        ensure_private_dir(&accounts_dir)?;

        let destination = self.account_path(account.id);

        match fs::symlink_metadata(&destination) {
            Ok(metadata) => {
                if !replace {
                    bail!("Account already exists");
                }
                if !metadata.file_type().is_file() {
                    bail!("Account destination is not a regular file");
                }
            }

            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if replace {
                    bail!("Cannot update an account which does not exist");
                }
            }

            Err(error) => return Err(error.into()),
        }

        let policy = StandardPolicy::new();

        let recipients: Vec<_> = public_cert
            .keys()
            .with_policy(&policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .collect();

        if recipients.is_empty() {
            bail!("No usable storage encryption keys found");
        }

        let mut temp = tempfile::NamedTempFile::new_in(&accounts_dir)?;

        set_private_permissions(temp.as_file())?;

        // TODO: extrapolate this logic?
        {
            let message = Message::new(temp.as_file_mut());
            let message = Encryptor::for_recipients(message, recipients).build()?;
            let message = Signer::new(message, signing_keypair)?
                .add_intended_recipient(public_cert)
                .build()?;

            let literal = LiteralWriter::new(message).build()?;
            let mut limited = LimitedWriter::new(literal, MAX_PLAINTEXT_SIZE);

            serde_json::to_writer(&mut limited, account)?;

            limited.into_inner().finalize()?;
        }

        temp.as_file().sync_all()?;

        if replace {
            temp.persist(&destination).map_err(|error| error.error)?;
        } else {
            temp.persist_noclobber(&destination)
                .map_err(|error| error.error)?;
        }

        #[cfg(unix)]
        File::open(&accounts_dir)?.sync_all()?;

        Ok(())
    }

    /// Account retrieval via uuid
    pub(crate) fn read_with_keys(&self, id: Uuid, keys: &UnlockedVaultKeys) -> Result<Account> {
        let _lock = FileLock::shared(&self.vault_path.join(".vault.lock"))?;

        self.read_locked(
            id,
            keys.public_cert.clone(),
            keys.decryption_keypairs.clone(),
        )
    }

    /// Decrypt and return the account
    fn read_locked(
        &self,
        id: Uuid,
        public_cert: Cert,
        decryption_keys: Vec<KeyPair>,
    ) -> Result<Account> {
        let path = self.account_path(id);

        let metadata = fs::symlink_metadata(&path)?;

        if !metadata.file_type().is_file() {
            bail!("Account path is not a regular file");
        }

        if metadata.len() > MAX_CIPHERTEXT_SIZE {
            bail!("Account ciphertext exceeds maximum size");
        }

        let ciphertext = fs::read(&path)?;
        let helper = VaultDecryptHelper::new(public_cert, decryption_keys);
        let policy = StandardPolicy::new();

        let mut decryptor = DecryptorBuilder::from_bytes(&ciphertext)?
            .buffer_size(MAX_PLAINTEXT_SIZE + 1)
            .with_policy(&policy, None, helper)?;

        let mut plaintext = Zeroizing::new(Vec::<u8>::new());

        {
            let mut limited_reader = (&mut decryptor).take((MAX_PLAINTEXT_SIZE + 1) as u64);
            limited_reader.read_to_end(&mut plaintext)?;
        }

        if plaintext.len() > MAX_PLAINTEXT_SIZE {
            bail!("Decrypted account exceeds maximum size");
        }

        if !decryptor.message_processed() {
            bail!("Account message has not been fully authenticated");
        }

        let account: Account = serde_json::from_slice(plaintext.as_slice())
            .context("Decrypted account contains invalid JSON")?;

        if account.id != id {
            bail!("Account ID does not match its filename");
        }

        account.validate()?;

        Ok(account)
    }

    /// Deletes an account by its UUID.
    pub(crate) fn delete(&self, id: Uuid) -> Result<()> {
        let _vault_lock = FileLock::exclusive(&self.vault_path.join(".vault.lock"))?;

        let path = self.account_path(id);

        let metadata = fs::symlink_metadata(&path)?;

        if !metadata.file_type().is_file() {
            bail!("Refusing to delete a non-regular account file");
        }

        fs::remove_file(&path)?;

        #[cfg(unix)]
        File::open(self.accounts_dir())?.sync_all()?;

        Ok(())
    }

    /// Lists account ids without decrypting account data.
    pub(crate) fn list_ids(&self) -> Result<Vec<Uuid>> {
        let _vault_lock = FileLock::shared(&self.vault_path.join(".vault.lock"))?;

        let mut ids = Vec::new();

        for entry in fs::read_dir(self.accounts_dir())? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("pgp") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Ok(id) = Uuid::parse_str(stem) {
                ids.push(id);
            }
        }

        ids.sort_unstable();

        Ok(ids)
    }
}

/// Limits the number of plaintext bytes sent to OpenPGP serializer. Prevents unusuallly large ops
struct LimitedWriter<W> {
    inner: W,
    written: usize,
    limit: usize,
}

impl<W> LimitedWriter<W> {
    fn new(inner: W, limit: usize) -> Self {
        Self {
            inner,
            written: 0,
            limit,
        }
    }

    fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for LimitedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.len() > self.limit.saturating_sub(self.written) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Account exceeds maximum plaintext size",
            ));
        }

        let written = self.inner.write(buffer)?;
        self.written += written;

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// TODO: pretty sure this already exists
/// Creates a private directory.
fn ensure_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;

    let metadata = fs::symlink_metadata(path)?;

    if !metadata.file_type().is_dir() {
        bail!("Expected a real directory");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }

    // TODO binbows
    Ok(())
}

/// Restricts a file to its owner on Unix.
fn set_private_permissions(file: &File) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }

    // TODO binbows

    Ok(())
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        crypto::{
            RecipientBuilder, load_vault_certs, unlock_signing_keypair, unlock_storage_keypairs,
        },
        secret::SecretString,
        vault_session::UnlockedVaultKeys,
    };

    use sequoia_openpgp::{Cert, crypto::KeyPair};
    use tempfile::tempdir;

    const TEST_PASSPHRASE: &str = "test-only-passphrase-do-not-use-in-production";

    /// Creates a test vault and obtains its unlocked key material.
    ///
    /// These tests exercise AccountStore's internal keyed API.
    /// Passphrase verification is tested separately by VaultSession.
    fn setup_store(path: &Path) -> (AccountStore, UnlockedVaultKeys) {
        RecipientBuilder::new("FMP Test <test@example.org>", path)
            .write(TEST_PASSPHRASE)
            .unwrap();

        let (public_cert, secret_cert) = load_vault_certs(path).unwrap();

        let keys = UnlockedVaultKeys {
            public_cert,
            signing_keypair: unlock_signing_keypair(&secret_cert, TEST_PASSPHRASE).unwrap(),
            decryption_keypairs: unlock_storage_keypairs(&secret_cert, TEST_PASSPHRASE).unwrap(),
        };

        (AccountStore::new(path), keys)
    }

    fn test_account() -> Account {
        let mut account = Account::new("Example Account".to_string());

        account.website = SecretString::new("https://example.com".to_string());

        account.username = SecretString::new("example@example.com".to_string());

        account.password = SecretString::new("VERY_SECRET_TEST_PASSWORD".to_string());

        account.notes = SecretString::new("Test account notes".to_string());

        account.otpauth = Some(SecretString::new(
            "otpauth://totp/Test?secret=EXAMPLE".to_string(),
        ));

        account.additional_fields.insert(
            "Recovery Code".to_string(),
            SecretString::new("123456789".to_string()),
        );

        account
    }

    #[test]
    fn test_account_round_trip() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let account = test_account();
        let id = account.id;

        store.create_with_keys(&account, &keys).unwrap();

        let account_path = dir.path().join("accounts").join(format!("{id}.pgp"));

        assert!(account_path.is_file());

        let ciphertext = fs::read(&account_path).unwrap();

        assert!(!ciphertext.is_empty());
        assert!(!ciphertext.starts_with(b"{"));

        assert!(
            !ciphertext
                .windows(b"VERY_SECRET_TEST_PASSWORD".len())
                .any(|window| window == b"VERY_SECRET_TEST_PASSWORD"),
            "Password leaked into the ciphertext file"
        );

        let recovered = store.read_with_keys(id, &keys).unwrap();

        assert_eq!(recovered.name.expose_secret(), "Example Account");

        assert_eq!(recovered.website.expose_secret(), "https://example.com");

        assert_eq!(recovered.username.expose_secret(), "example@example.com");

        assert_eq!(
            recovered.password.expose_secret(),
            "VERY_SECRET_TEST_PASSWORD"
        );

        assert_eq!(recovered.notes.expose_secret(), "Test account notes");

        assert_eq!(
            recovered.otpauth.as_ref().unwrap().expose_secret(),
            "otpauth://totp/Test?secret=EXAMPLE"
        );

        assert_eq!(
            recovered
                .additional_fields
                .get("Recovery Code")
                .unwrap()
                .expose_secret(),
            "123456789"
        );
    }

    #[test]
    fn test_wrong_vault_keys_rejected() {
        let vault_a = tempdir().unwrap();
        let vault_b = tempdir().unwrap();

        let (store_a, keys_a) = setup_store(vault_a.path());
        let (_store_b, keys_b) = setup_store(vault_b.path());

        let account = test_account();
        let id = account.id;

        store_a.create_with_keys(&account, &keys_a).unwrap();

        assert!(
            store_a.read_with_keys(id, &keys_b).is_err(),
            "A different vault's decryption keys must not decrypt this account"
        );
    }

    #[test]
    fn test_duplicate_account_creation_rejected() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let account = test_account();

        store.create_with_keys(&account, &keys).unwrap();

        assert!(store.create_with_keys(&account, &keys).is_err());
    }

    #[test]
    fn test_account_update() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let mut account = test_account();
        let id = account.id;

        store.create_with_keys(&account, &keys).unwrap();

        account
            .password
            .replace("UPDATED_TEST_PASSWORD".to_string());

        store.update_with_keys(&mut account, &keys).unwrap();

        let recovered = store.read_with_keys(id, &keys).unwrap();

        assert_eq!(recovered.password.expose_secret(), "UPDATED_TEST_PASSWORD");

        assert_eq!(recovered.id, id);
    }

    #[test]
    fn test_update_missing_account_rejected() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let mut account = test_account();

        assert!(store.update_with_keys(&mut account, &keys).is_err());

        assert!(store.list_ids().unwrap().is_empty());
    }

    #[test]
    fn test_tampered_account_rejected() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let account = test_account();
        let id = account.id;

        store.create_with_keys(&account, &keys).unwrap();

        let path = dir.path().join("accounts").join(format!("{id}.pgp"));

        let mut data = fs::read(&path).unwrap();

        assert!(!data.is_empty());

        let middle = data.len() / 2;
        data[middle] ^= 0x01;

        fs::write(&path, data).unwrap();

        assert!(
            store.read_with_keys(id, &keys).is_err(),
            "Tampered account must not be accepted"
        );
    }

    #[test]
    fn test_list_account_ids() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let account_a = test_account();
        let account_b = test_account();

        store.create_with_keys(&account_a, &keys).unwrap();
        store.create_with_keys(&account_b, &keys).unwrap();

        let ids = store.list_ids().unwrap();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&account_a.id));
        assert!(ids.contains(&account_b.id));
    }

    #[test]
    fn test_delete_account() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let account = test_account();
        let id = account.id;

        store.create_with_keys(&account, &keys).unwrap();

        store.delete(id).unwrap();

        assert!(store.read_with_keys(id, &keys).is_err());
    }

    fn write_signed_by_another_vault(
        path: &Path,
        account: &Account,
        recipient_cert: &Cert,
        signing_keypair: KeyPair,
    ) {
        let policy = StandardPolicy::new();

        let recipients: Vec<_> = recipient_cert
            .keys()
            .with_policy(&policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .collect();

        let mut ciphertext = Vec::new();

        {
            let message = Message::new(&mut ciphertext);

            let message = Encryptor::for_recipients(message, recipients)
                .build()
                .unwrap();

            let message = Signer::new(message, signing_keypair)
                .unwrap()
                .add_intended_recipient(recipient_cert)
                .build()
                .unwrap();

            let mut literal = LiteralWriter::new(message).build().unwrap();

            serde_json::to_writer(&mut literal, account).unwrap();

            literal.finalize().unwrap();
        }

        fs::write(path, ciphertext).unwrap();
    }

    #[test]
    fn test_account_signed_by_another_vault_rejected() {
        let vault_a = tempdir().unwrap();
        let vault_b = tempdir().unwrap();

        let (store_a, keys_a) = setup_store(vault_a.path());
        let (_store_b, keys_b) = setup_store(vault_b.path());

        let account = test_account();
        let id = account.id;

        store_a.create_with_keys(&account, &keys_a).unwrap();

        let destination = vault_a.path().join("accounts").join(format!("{id}.pgp"));

        write_signed_by_another_vault(
            &destination,
            &account,
            &keys_a.public_cert,
            keys_b.signing_keypair.clone(),
        );

        assert!(
            store_a.read_with_keys(id, &keys_a).is_err(),
            "An account signed by another vault must be rejected"
        );
    }

    #[test]
    fn test_unsigned_account_rejected() {
        let dir = tempdir().unwrap();
        let (store, keys) = setup_store(dir.path());

        let account = test_account();
        let id = account.id;

        store.create_with_keys(&account, &keys).unwrap();

        let policy = StandardPolicy::new();

        let recipients: Vec<_> = keys
            .public_cert
            .keys()
            .with_policy(&policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .collect();

        let mut ciphertext = Vec::new();

        {
            let message = Message::new(&mut ciphertext);

            let message = Encryptor::for_recipients(message, recipients)
                .build()
                .unwrap();

            // Deliberately omit the signing layer.
            let mut literal = LiteralWriter::new(message).build().unwrap();

            serde_json::to_writer(&mut literal, &account).unwrap();

            literal.finalize().unwrap();
        }

        let destination = dir.path().join("accounts").join(format!("{id}.pgp"));

        fs::write(&destination, ciphertext).unwrap();

        assert!(
            store.read_with_keys(id, &keys).is_err(),
            "Unsigned accounts must not be accepted"
        );
    }
}
