use crate::{
    account::Account,
    account_store::AccountStore,
    crypto::{load_vault_certs, unlock_signing_keypair, unlock_storage_keypairs},
    paths::{FileLock, VaultPaths, validate_vault_name},
};
use anyhow::{Context, Result, bail};
use sequoia_openpgp::{Cert, crypto::KeyPair};
use std::{fs, path::PathBuf};
use uuid::Uuid;

/// Keyring storing needed keys for operations
pub(crate) struct UnlockedVaultKeys {
    pub(crate) public_cert: Cert,
    pub(crate) signing_keypair: KeyPair,
    pub(crate) decryption_keypairs: Vec<KeyPair>,
}

/// Basically an unlocked vault.
pub struct VaultSession {
    vault_name: String,
    vault_path: PathBuf,
    unlocked: Option<UnlockedVaultKeys>,
    global_lock: Option<FileLock>,
}

impl VaultSession {
    pub fn unlock(app_dir: impl Into<PathBuf>, vault_name: &str, passphrase: &str) -> Result<Self> {
        validate_vault_name(vault_name)?;

        if passphrase.is_empty() {
            bail!("Vault passphrase cannot be empty");
        }

        let app_dir = app_dir.into();

        let global_lock = FileLock::shared(&app_dir.join(".global.lock"))?;

        let paths = VaultPaths::new(&app_dir, vault_name)?;

        // Refuse symlinks and unexpected filesystem objects.
        let metadata = fs::symlink_metadata(&paths.root).context("Vault does not exist")?;

        if !metadata.file_type().is_dir() {
            bail!("Vault path is not a real directory");
        }

        // Load and validate the certificate pair.
        let (public_cert, secret_cert) = load_vault_certs(&paths.root)?;

        // Unlock the signing key
        let signing_keypair = unlock_signing_keypair(&secret_cert, passphrase)?;

        // Unlock the storage decryption keys
        let decryption_keypairs = unlock_storage_keypairs(&secret_cert, passphrase)?;

        let unlocked = UnlockedVaultKeys {
            public_cert,
            signing_keypair,
            decryption_keypairs,
        };

        Ok(Self {
            vault_name: vault_name.to_string(),
            vault_path: paths.root,
            unlocked: Some(unlocked),
            global_lock: Some(global_lock),
        })
    }

    /// Returns the vault name.
    pub fn name(&self) -> &str {
        &self.vault_name
    }

    /// Returns whether the session currently holds unlocked keys.
    pub fn is_unlocked(&self) -> bool {
        self.unlocked.is_some()
    }

    fn keys(&self) -> Result<&UnlockedVaultKeys> {
        self.unlocked.as_ref().context("Vault session is locked")
    }

    pub fn create_account(&self, account: &Account) -> Result<()> {
        AccountStore::new(self.vault_path.clone()).create_with_keys(account, self.keys()?)
    }

    pub fn read_account(&self, id: Uuid) -> Result<Account> {
        AccountStore::new(self.vault_path.clone()).read_with_keys(id, self.keys()?)
    }

    pub fn update_account(&self, account: &mut Account) -> Result<()> {
        AccountStore::new(self.vault_path.clone()).update_with_keys(account, self.keys()?)
    }

    pub fn delete_account(&self, id: Uuid) -> Result<()> {
        self.keys()?;

        AccountStore::new(self.vault_path.clone()).delete(id)
    }

    /// Lists account UUIDs without decrypting their contents.
    pub fn list_account_ids(&self) -> Result<Vec<Uuid>> {
        self.keys()?;

        AccountStore::new(self.vault_path.clone()).list_ids()
    }

    pub fn lock(&mut self) {
        drop(self.unlocked.take());
        drop(self.global_lock.take());
    }
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;

    use crate::{secret::SecretString, vault::VaultManager};

    use tempfile::tempdir;

    const TEST_PASSPHRASE: &str = "test-only-passphrase-never-use-in-production";

    fn setup() -> (tempfile::TempDir, VaultManager) {
        let dir = tempdir().unwrap();

        let manager = VaultManager::at(dir.path().join("fmp"));

        manager
            .create("Personal", "FMP Test <test@example.org>", TEST_PASSPHRASE)
            .unwrap();

        (dir, manager)
    }

    #[test]
    fn test_session_unlock() {
        let (_dir, manager) = setup();

        let session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        assert!(session.is_unlocked());

        assert_eq!(session.name(), "Personal");
    }

    #[test]
    fn test_wrong_passphrase_rejected() {
        let (_dir, manager) = setup();

        assert!(
            manager
                .unlock_vault("Personal", "incorrect-passphrase")
                .is_err()
        );

        // An empty passphrase must also be rejected.
        assert!(manager.unlock_vault("Personal", "").is_err());
    }

    #[test]
    fn test_session_account_lifecycle() {
        let (_dir, manager) = setup();

        let session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        let mut account = Account::new("Email Account".to_string());

        let id = account.id;

        account.username = SecretString::new("test@example.com".to_string());

        account.password = SecretString::new("INITIAL_PASSWORD".to_string());

        // Create.
        session.create_account(&account).unwrap();

        // List.
        let ids = session.list_account_ids().unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], id);

        // Read.
        let mut recovered = session.read_account(id).unwrap();

        assert_eq!(recovered.password.expose_secret(), "INITIAL_PASSWORD");

        // Update.
        recovered.password.replace("UPDATED_PASSWORD".to_string());

        session.update_account(&mut recovered).unwrap();

        // Read again.
        let updated = session.read_account(id).unwrap();

        assert_eq!(updated.password.expose_secret(), "UPDATED_PASSWORD");

        // Delete.
        session.delete_account(id).unwrap();

        assert!(session.list_account_ids().unwrap().is_empty());
    }

    #[test]
    fn test_explicit_lock() {
        let (_dir, manager) = setup();

        let mut session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        // Create an account before locking.
        let account = Account::new("Before lock".to_string());
        let existing_id = account.id;

        session.create_account(&account).unwrap();

        assert!(session.is_unlocked());

        // Explicitly lock the vault.
        session.lock();

        assert!(!session.is_unlocked());

        // Account creation must fail after locking.
        assert!(
            session
                .create_account(&Account::new("After lock".to_string()))
                .is_err()
        );

        // Account update must fail.
        let mut account_for_update = Account::new("Update after lock".to_string());

        account_for_update.id = existing_id;

        assert!(session.update_account(&mut account_for_update).is_err());

        // Reading must fail.
        assert!(session.read_account(existing_id).is_err());

        // Deletion must fail.
        assert!(session.delete_account(existing_id).is_err());

        // Listing must fail.
        assert!(session.list_account_ids().is_err());
    }

    #[test]
    fn test_lock_released_after_session_lock() {
        let (_dir, manager) = setup();

        let mut session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        // Vault management must be possible again after
        // explicitly closing the session.
        session.lock();

        manager.rename("Personal", "Renamed").unwrap();

        assert!(manager.exists("Renamed").unwrap());
        assert!(!manager.exists("Personal").unwrap());
    }

    #[test]
    fn test_backup_while_session_open() {
        let (_dir, manager) = setup();

        let session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        // A backup should not require the session to close.
        manager.create_backup("Personal").unwrap();

        assert!(manager.backup_exists("Personal").unwrap());

        assert!(session.is_unlocked());
    }
}
