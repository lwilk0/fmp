use crate::{
    crypto::RecipientBuilder,
    metadata::{MetadataStore, VaultMetadata},
    paths::{
        FileLock, VaultPaths, atomic_write, create_secure_dir, sync_directory, validate_vault_name,
    },
};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tempfile::Builder as TempDirBuilder;
use uuid::Uuid;

/// Current vault api version.
const VAULT_API_VERSION: u32 = 1;

/// Public information about the vault
#[derive(Debug, Serialize, Deserialize)]
struct VaultManifest {
    api_version: u32,
    id: Uuid,
    created_at: String,
}

/// Creates a Manager for a vault, contains every function needed to manage it
pub struct VaultManager {
    app_dir: PathBuf,
    vaults_dir: PathBuf,
    backups_dir: PathBuf,
    lock_path: PathBuf,
    metadata: MetadataStore,
}

impl VaultManager {
    /// Creates a manager in the data dir
    pub fn new() -> Result<Self> {
        let data_dir =
            dirs::data_dir().context("Unable to determine application data directory")?;

        Ok(Self::at(data_dir.join("fmp")))
    }

    /// Creates a manager using a supplied application directory - mainly for testing
    pub fn at(app_dir: impl Into<PathBuf>) -> Self {
        let app_dir = app_dir.into();

        Self {
            vaults_dir: app_dir.join("vaults"),
            backups_dir: app_dir.join("backups"),
            lock_path: app_dir.join(".global.lock"),
            metadata: MetadataStore::new(&app_dir),
            app_dir,
        }
    }

    /// Ensures all the basic dirs exist
    fn prepare_layout(&self) -> Result<()> {
        create_secure_dir(&self.app_dir)?;
        create_secure_dir(&self.vaults_dir)?;
        create_secure_dir(&self.backups_dir)?;

        Ok(())
    }

    fn lock_exclusive(&self) -> Result<FileLock> {
        FileLock::exclusive(&self.lock_path)
    }

    fn lock_shared(&self) -> Result<FileLock> {
        FileLock::shared(&self.lock_path)
    }

    /// Returns the paths belonging to a vault.
    pub fn paths(&self, name: &str) -> Result<VaultPaths> {
        VaultPaths::new(&self.app_dir, name)
    }

    /// Checks whether a path exists without following symlinks.
    fn path_exists(path: &Path) -> Result<bool> {
        match fs::symlink_metadata(path) {
            Ok(_) => Ok(true),

            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),

            Err(error) => Err(error.into()),
        }
    }

    /// Verifies that a vault is an actual directory.
    fn verify_vault_directory(&self, name: &str) -> Result<PathBuf> {
        let paths = self.paths(name)?;

        let metadata = fs::symlink_metadata(&paths.root)
            .with_context(|| format!("Vault `{name}` does not exist"))?;

        if !metadata.file_type().is_dir() {
            bail!("Vault path is not a real directory");
        }

        Ok(paths.root)
    }

    fn verify_backup_directory(&self, name: &str) -> Result<bool> {
        let paths = self.paths(name)?;

        match fs::symlink_metadata(&paths.backup) {
            Ok(metadata) => {
                if !metadata.file_type().is_dir() {
                    bail!("Backup path is not a real directory");
                }

                Ok(true)
            }

            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),

            Err(error) => Err(error.into()),
        }
    }

    /// Creates a new vault and generates its OpenPGP certificate.
    pub fn create(&self, name: &str, recipient: &str, passphrase: &str) -> Result<()> {
        validate_vault_name(name)?;
        self.prepare_layout()?;

        let _lock = self.lock_exclusive()?;

        let destination = self.vaults_dir.join(name);

        if Self::path_exists(&destination)? {
            bail!("Vault `{name}` already exists");
        }

        // Build the new vault in a temporary directory.
        let temp_dir = TempDirBuilder::new()
            .prefix(".fmp-vault-create-")
            .tempdir_in(&self.app_dir)
            .context("Failed to create vault temporary directory")?;

        create_secure_dir(temp_dir.path())?;

        // Create the account directory.
        let accounts_dir = temp_dir.path().join("accounts");
        create_secure_dir(&accounts_dir)?;

        // Generate public/private certificates and revocation data.
        let recipient_builder =
            RecipientBuilder::new(recipient.to_string(), temp_dir.path().to_path_buf());

        recipient_builder
            .write(passphrase)
            .context("Failed to generate vault recipient")?;

        // Write the versioned manifest.
        let manifest = VaultManifest {
            api_version: VAULT_API_VERSION,
            id: Uuid::new_v4(),
            created_at: Utc::now().to_rfc3339(),
        };

        let serialized_manifest = serde_json::to_vec(&manifest)?;

        atomic_write(&temp_dir.path().join("vault.json"), &serialized_manifest)?;

        sync_directory(temp_dir.path())?;

        // Check again immediately before move.
        if Self::path_exists(&destination)? {
            bail!("Vault destination appeared during creation");
        }

        // Move the completed vault.
        fs::rename(temp_dir.path(), &destination).context("Failed to move completed vault")?;

        sync_directory(&self.vaults_dir)?;
        sync_directory(&self.app_dir)?;

        Ok(())
    }

    /// Returns all vault directory names.
    pub fn list(&self) -> Result<Vec<String>> {
        self.prepare_layout()?;

        let _lock = self.lock_shared()?;

        let mut vaults = Vec::new();

        for entry in fs::read_dir(&self.vaults_dir)? {
            let entry = entry?;

            if !entry.file_type()?.is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if validate_vault_name(&name).is_err() {
                continue;
            }

            vaults.push(name);
        }

        vaults.sort_unstable();

        Ok(vaults)
    }

    /// Checks whether a vault exists.
    pub fn exists(&self, name: &str) -> Result<bool> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        let _lock = self.lock_shared()?;

        let path = self.vaults_dir.join(name);

        match fs::symlink_metadata(&path) {
            Ok(metadata) => Ok(metadata.file_type().is_dir()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    /// Renames a vault and moves its backup if one exists.
    pub fn rename(&self, old_name: &str, new_name: &str) -> Result<()> {
        validate_vault_name(old_name)?;
        validate_vault_name(new_name)?;

        self.prepare_layout()?;

        let _lock = self.lock_exclusive()?;

        let old_vault = self.verify_vault_directory(old_name)?;

        let vault_lock = FileLock::exclusive(&old_vault.join(".vault.lock"))?;

        if old_name == new_name {
            return Ok(());
        }

        let old_paths = self.paths(old_name)?;
        let new_paths = self.paths(new_name)?;

        if Self::path_exists(&new_paths.root)? {
            bail!("Vault `{new_name}` already exists");
        }

        let has_backup = self.verify_backup_directory(old_name)?;

        if Self::path_exists(&new_paths.backup)? {
            bail!("A backup already exists under the new vault name");
        }

        let old_metadata = self.metadata.load()?;
        let mut new_metadata = old_metadata.clone();

        new_metadata.rename_vault(old_name, new_name);

        // Rename the vault directory.
        fs::rename(&old_vault, &new_paths.root).context("Failed to rename vault directory")?;

        // Move vault backup.
        if has_backup && let Err(error) = fs::rename(&old_paths.backup, &new_paths.backup) {
            // Restore the original vault location.
            let rollback = fs::rename(&new_paths.root, &old_paths.root);

            if rollback.is_err() {
                bail!(
                    "Backup rename failed: {error}; \
                    vault rollback also failed"
                );
            }

            return Err(error).context("Failed to rename vault backup");
        }

        // Update metadata.
        if let Err(error) = self.metadata.save(&new_metadata) {
            let backup_rollback = if has_backup {
                fs::rename(&new_paths.backup, &old_paths.backup)
            } else {
                Ok(())
            };

            let vault_rollback = fs::rename(&new_paths.root, &old_paths.root);

            // Restore metadata even if the save operation failed after its replacement had already occurred.
            let metadata_rollback = self.metadata.save(&old_metadata);

            if backup_rollback.is_err() || vault_rollback.is_err() || metadata_rollback.is_err() {
                bail!(
                    "Metadata update failed: {error}; \
                     one or more rollback operations failed"
                );
            }

            return Err(error).context("Failed to update vault metadata");
        }

        drop(vault_lock);

        sync_directory(&self.vaults_dir)?;

        if has_backup {
            sync_directory(&self.backups_dir)?;
        }

        Ok(())
    }

    /// Deletes a vault and its backup. Rollback may not be needed but, eh
    pub fn delete(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        let _lock = self.lock_exclusive()?;

        let vault_path = self.verify_vault_directory(name)?;
        let vault_lock = FileLock::exclusive(&vault_path.join(".vault.lock"))?;
        let paths = self.paths(name)?;
        let has_backup = self.verify_backup_directory(name)?;
        let old_metadata = self.metadata.load()?;
        let mut new_metadata = old_metadata.clone();
        new_metadata.remove_vault(name);

        // Create a trash location.
        let trash_dir = self.app_dir.join(".trash");
        // Secure the trash!!!
        create_secure_dir(&trash_dir)?;
        let trash_path = trash_dir.join(Uuid::new_v4().to_string());

        fs::create_dir(&trash_path)?;
        create_secure_dir(&trash_path)?;

        let staged_vault = trash_path.join("vault");
        let staged_backup = trash_path.join("backup");

        // Move the vault into trash.
        fs::rename(&vault_path, &staged_vault).context("Failed to stage vault deletion")?;

        // Move the backup too, if it exists.
        if has_backup && let Err(error) = fs::rename(&paths.backup, &staged_backup) {
            let rollback = fs::rename(&staged_vault, &paths.root);

            if rollback.is_err() {
                bail!(
                    "Backup c failed: {error}; \
                         vault rollback failed. \
                         Data remains in {:?}",
                    trash_path
                );
            }

            return Err(error).context("Failed to stage backup deletion");
        }

        // Update metadata.
        if let Err(error) = self.metadata.save(&new_metadata) {
            let backup_rollback = if has_backup {
                fs::rename(&staged_backup, &paths.backup)
            } else {
                Ok(())
            };

            let vault_rollback = fs::rename(&staged_vault, &paths.root);
            let metadata_rollback = self.metadata.save(&old_metadata);
            if backup_rollback.is_err() || vault_rollback.is_err() || metadata_rollback.is_err() {
                bail!(
                    "Metadata update failed: {error}; \
                     rollback incomplete. \
                     Recovery data may remain at {:?}",
                    trash_path
                );
            }

            drop(vault_lock);

            let _ = fs::remove_dir(&trash_path);

            return Err(error).context("Failed to update deletion metadata");
        }

        // Remove trash after pass
        if let Err(error) = fs::remove_dir_all(&trash_path) {
            bail!(
                "Vault removed from active storage, but \
                 quarantine cleanup failed at {:?}: {}",
                trash_path,
                error
            );
        }

        sync_directory(&self.vaults_dir)?;
        sync_directory(&self.backups_dir)?;
        sync_directory(&trash_dir)?;

        Ok(())
    }

    /// Records a successful vault opening
    pub fn record_opened(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        let _lock = self.lock_exclusive()?;

        self.verify_vault_directory(name)?;

        let mut metadata = self.metadata.load()?;
        metadata.record_opened(name);
        self.metadata.save(&metadata)?;

        Ok(())
    }

    /// Increments the usage count
    pub fn increment_vault_usage(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        let _lock = self.lock_exclusive()?;

        self.verify_vault_directory(name)?;
        let mut metadata = self.metadata.load()?;
        metadata.increment_usage(name);
        self.metadata.save(&metadata)?;

        Ok(())
    }

    /// Records a vault as recently accessed.
    pub fn record_recent_vault(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        let _lock = self.lock_exclusive()?;

        self.verify_vault_directory(name)?;
        let mut metadata = self.metadata.load()?;
        metadata.record_recent(name);
        self.metadata.save(&metadata)?;

        Ok(())
    }

    /// Returns recently accessed vaults, newest first.
    pub fn get_recent_vaults(&self, limit: usize) -> Result<Vec<String>> {
        self.prepare_layout()?;

        let _lock = self.lock_shared()?;

        let metadata = self.metadata.load()?;

        Ok(metadata.recent_vaults(limit))
    }

    /// Returns the most-used vault, if one has been recorded.
    pub fn get_most_used_vault(&self) -> Result<Option<String>> {
        self.prepare_layout()?;

        let _lock = self.lock_shared()?;

        let metadata = self.metadata.load()?;

        Ok(metadata.most_used_vault())
    }

    /// Returns a vault's recorded usage count.
    pub fn get_vault_usage(&self, name: &str) -> Result<u64> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        let _lock = self.lock_shared()?;

        let metadata: VaultMetadata = self.metadata.load()?;

        Ok(metadata.usage_count(name))
    }

    /// Creates a backup of a vault.
    pub fn create_backup(&self, name: &str) -> Result<()> {
        crate::backup::BackupManager::at(self.app_dir.clone()).create(name)
    }

    /// Restores a vault from its backup.
    pub fn install_backup(&self, name: &str) -> Result<()> {
        crate::backup::BackupManager::at(self.app_dir.clone()).install(name)
    }

    /// Checks whether a backup exists.
    pub fn backup_exists(&self, name: &str) -> Result<bool> {
        crate::backup::BackupManager::at(self.app_dir.clone()).exists(name)
    }

    /// Deletes a backup without deleting the original vault.
    pub fn delete_backup(&self, name: &str) -> Result<()> {
        crate::backup::BackupManager::at(self.app_dir.clone()).delete(name)
    }

    /// Opens and unlocks an existing vault.
    pub fn unlock_vault(
        &self,
        name: &str,
        passphrase: &str,
    ) -> Result<crate::vault_session::VaultSession> {
        validate_vault_name(name)?;

        self.prepare_layout()?;

        crate::vault_session::VaultSession::unlock(self.app_dir.clone(), name, passphrase)
    }
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;

    use crate::paths::create_secure_dir;

    use tempfile::tempdir;

    const TEST_PASSPHRASE: &str = "test-only-passphrase-never-use-in-production";

    fn setup() -> (tempfile::TempDir, VaultManager) {
        let dir = tempdir().unwrap();

        let manager = VaultManager::at(dir.path().join("fmp"));

        (dir, manager)
    }

    fn create_test_vault(manager: &VaultManager, name: &str) {
        manager
            .create(name, "FMP Test <test@example.org>", TEST_PASSPHRASE)
            .unwrap_or_else(|err| {
                panic!("Failed to create test vault:\n{err:#}");
            });
    }

    #[test]
    fn test_create_and_list_vaults() {
        let (_dir, manager) = setup();

        assert!(!manager.exists("Personal").unwrap());

        create_test_vault(&manager, "Personal");

        assert!(manager.exists("Personal").unwrap());

        assert_eq!(manager.list().unwrap(), vec!["Personal".to_string()]);

        let paths = manager.paths("Personal").unwrap();

        assert!(paths.root.is_dir());
        assert!(paths.accounts.is_dir());
        assert!(paths.recipient.is_dir());

        assert!(paths.recipient.join("recipient.asc").exists());
        assert!(paths.recipient.join("recipient.key").exists());
        assert!(paths.recipient.join("recipient.rev").exists());

        assert!(paths.root.join("vault.json").exists());
    }

    #[test]
    fn test_duplicate_vault_rejected() {
        let (_dir, manager) = setup();

        create_test_vault(&manager, "Personal");

        assert!(
            manager
                .create("Personal", "Another Recipient", TEST_PASSPHRASE,)
                .is_err()
        );

        assert_eq!(manager.list().unwrap().len(), 1);
    }

    #[test]
    fn test_invalid_vault_names_rejected() {
        let (_dir, manager) = setup();

        let invalid_names = [
            "",
            ".",
            "..",
            "../outside",
            "/tmp/malicious",
            "vault/name",
            "vault\\name",
            "CON",
            "NUL",
            "name.",
            " name",
            "name ",
        ];

        for name in invalid_names {
            assert!(
                manager
                    .create(name, "Test Recipient", TEST_PASSPHRASE,)
                    .is_err(),
                "Invalid vault name was accepted: {name:?}"
            );
        }

        assert!(manager.list().unwrap().is_empty());
    }

    #[test]
    fn test_vault_rename_moves_backup_and_metadata() {
        let (_dir, manager) = setup();

        create_test_vault(&manager, "OldName");

        manager.record_opened("OldName").unwrap();
        manager.record_opened("OldName").unwrap();

        let old_paths = manager.paths("OldName").unwrap();

        // Create a representative backup directory.
        create_secure_dir(&old_paths.backup).unwrap();

        fs::write(old_paths.backup.join("backup-marker"), b"test backup").unwrap();

        manager.rename("OldName", "NewName").unwrap();

        assert!(!manager.exists("OldName").unwrap());
        assert!(manager.exists("NewName").unwrap());

        let new_paths = manager.paths("NewName").unwrap();

        assert!(new_paths.backup.is_dir());

        assert_eq!(
            fs::read(new_paths.backup.join("backup-marker")).unwrap(),
            b"test backup"
        );

        assert_eq!(
            manager.get_recent_vaults(10).unwrap(),
            vec!["NewName".to_string()]
        );

        assert_eq!(manager.get_vault_usage("NewName").unwrap(), 2);

        assert_eq!(manager.get_vault_usage("OldName").unwrap(), 0);

        assert_eq!(
            manager.get_most_used_vault().unwrap(),
            Some("NewName".to_string())
        );
    }

    #[test]
    fn test_rename_to_existing_vault_rejected() {
        let (_dir, manager) = setup();

        create_test_vault(&manager, "First");
        create_test_vault(&manager, "Second");

        assert!(manager.rename("First", "Second").is_err());

        assert!(manager.exists("First").unwrap());
        assert!(manager.exists("Second").unwrap());
    }

    #[test]
    fn test_vault_delete_removes_backup_and_metadata() {
        let (_dir, manager) = setup();

        create_test_vault(&manager, "Personal");

        manager.record_opened("Personal").unwrap();

        let paths = manager.paths("Personal").unwrap();

        create_secure_dir(&paths.backup).unwrap();

        fs::write(paths.backup.join("backup-marker"), b"test backup").unwrap();

        assert!(manager.backup_exists("Personal").unwrap());

        manager.delete("Personal").unwrap();

        assert!(!manager.exists("Personal").unwrap());

        assert!(!manager.backup_exists("Personal").unwrap());

        assert!(manager.get_recent_vaults(10).unwrap().is_empty());

        assert_eq!(manager.get_vault_usage("Personal").unwrap(), 0);

        assert_eq!(manager.get_most_used_vault().unwrap(), None);
    }

    #[test]
    fn test_delete_backup_preserves_vault() {
        let (_dir, manager) = setup();

        create_test_vault(&manager, "Personal");

        let paths = manager.paths("Personal").unwrap();

        create_secure_dir(&paths.backup).unwrap();

        assert!(manager.backup_exists("Personal").unwrap());

        manager.delete_backup("Personal").unwrap();

        assert!(!manager.backup_exists("Personal").unwrap());

        assert!(manager.exists("Personal").unwrap());
    }

    #[test]
    fn test_recent_vault_limit() {
        let (_dir, manager) = setup();

        for index in 0..12 {
            let name = format!("Vault{index}");

            create_test_vault(&manager, &name);

            manager.record_recent_vault(&name).unwrap();
        }

        let recent = manager.get_recent_vaults(10).unwrap();

        assert_eq!(recent.len(), 10);

        assert_eq!(recent[0], "Vault11");
        assert_eq!(recent[9], "Vault2");
    }

    #[test]
    fn test_reopening_vault_updates_usage() {
        let (_dir, manager) = setup();

        create_test_vault(&manager, "Personal");

        manager.record_opened("Personal").unwrap();
        manager.record_opened("Personal").unwrap();
        manager.record_opened("Personal").unwrap();

        assert_eq!(manager.get_vault_usage("Personal").unwrap(), 3);
    }

    #[test]
    fn test_missing_metadata_starts_empty() {
        let (_dir, manager) = setup();

        assert_eq!(manager.get_vault_usage("Personal").unwrap(), 0);

        assert_eq!(manager.get_most_used_vault().unwrap(), None);

        assert!(manager.get_recent_vaults(10).unwrap().is_empty());
    }
}
