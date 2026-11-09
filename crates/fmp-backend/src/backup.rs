use crate::paths::{
    FileLock, VaultPaths, create_secure_dir, set_secure_file, sync_directory, validate_vault_name,
};

use anyhow::{Context, Result, bail};

use sequoia_openpgp::{Cert, parse::Parse};

use serde::Deserialize;

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
};

use tempfile::Builder as TempDirBuilder;
use uuid::Uuid;

const VAULT_API_VERSION: u32 = 1;
const MAX_BACKUP_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_MANIFEST_SIZE: u64 = 16 * 1024;
const MAX_ACCOUNT_FILE_SIZE: u64 = 8 * 1024 * 1024;

/// Represents the vault manifest used to identify a vault. Not the other manifest - stucts arent sentient they cant write them
#[derive(Deserialize)]
struct VaultManifest {
    api_version: u32,
    id: Uuid,
    created_at: String,
}

/// Manages creation, verification and restoration of vault backups. WOW I SHOULD REALLY MAKE THE DOC LINES BETTER, TAKES LONGWR TO READ THE COMMENT THAN LOOK ATT HE CODE
pub struct BackupManager {
    app_dir: PathBuf,
    vaults_dir: PathBuf,
    backups_dir: PathBuf,
    global_lock: PathBuf,
}

impl BackupManager {
    /// Creates a backup manager using existing app data dir.
    pub fn at(app_dir: impl Into<PathBuf>) -> Self {
        let app_dir = app_dir.into();

        Self {
            vaults_dir: app_dir.join("vaults"),
            backups_dir: app_dir.join("backups"),
            global_lock: app_dir.join(".global.lock"),
            app_dir,
        }
    }

    fn prepare(&self) -> Result<()> {
        create_secure_dir(&self.app_dir)?;
        create_secure_dir(&self.vaults_dir)?;
        create_secure_dir(&self.backups_dir)?;

        Ok(())
    }

    fn paths(&self, name: &str) -> Result<VaultPaths> {
        VaultPaths::new(&self.app_dir, name)
    }

    /// Checks if directory is real... would hope so from the name, good old RRD
    fn require_real_directory(path: &Path) -> Result<()> {
        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("Directory does not exist: {}", path.display()))?;

        if !metadata.file_type().is_dir() {
            bail!("Expected a real directory: {}", path.display());
        }

        Ok(())
    }

    /// Creates a complete backup of a vault.
    pub fn create(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare()?;

        let _global_lock = FileLock::shared(&self.global_lock)?;
        let _backup_lock = FileLock::exclusive(&self.backups_dir.join(".backup.lock"))?;

        let paths = self.paths(name)?;

        Self::require_real_directory(&paths.root)?;

        let _vault_lock = FileLock::shared(&paths.root.join(".vault.lock"))?;

        let temp = TempDirBuilder::new()
            .prefix(".fmp-backup-")
            .tempdir_in(&self.backups_dir)
            .context("Failed to create backup temp directory")?;

        let staged_snapshot = temp.path().join("snapshot");

        let mut copied_bytes = 0;

        copy_tree_contents(&paths.root, &staged_snapshot, true, &mut copied_bytes)
            .context("Failed to copy vault into backup temp area")?;

        // Validate the copied vault before installation.
        validate_vault_tree(&staged_snapshot).context("Staged backup validation failed")?;

        let destination = &paths.backup;

        let destination_exists = directory_exists(destination)?;

        let previous_backup = if destination_exists {
            let old = self
                .backups_dir
                .join(format!(".fmp-previous-backup-{}", Uuid::new_v4()));

            fs::rename(destination, &old).context("Failed to stage the previous backup")?;

            Some(old)
        } else {
            None
        };

        // Install the completed snapshot.
        if let Err(error) = fs::rename(&staged_snapshot, destination)
            && let Some(previous) = &previous_backup
        {
            if fs::rename(previous, destination).is_err() {
                bail!(
                    "Backup installation failed: {error}; \
                         previous backup remains at {}",
                    previous.display()
                );
            }

            return Err(error).context("Failed to install new backup");
        }

        sync_directory(&self.backups_dir)?;

        // Remove prior backup
        if let Some(previous) = previous_backup
            && let Err(error) = fs::remove_dir_all(&previous)
        {
            bail!(
                "New backup was installed, but previous backup \
                     cleanup failed at {}: {}",
                previous.display(),
                error
            );
        }

        sync_directory(&self.backups_dir)?;

        Ok(())
    }

    /// Restores a vault from its backup
    pub fn install(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare()?;

        let _global_lock = FileLock::exclusive(&self.global_lock)?;

        let paths = self.paths(name)?;

        Self::require_real_directory(&paths.backup)?;

        let backup_id = validate_vault_tree(&paths.backup).context("Backup validation failed")?;

        let destination_exists = match fs::symlink_metadata(&paths.root) {
            Ok(metadata) => {
                if !metadata.file_type().is_dir() {
                    bail!("Existing vault path is not a real directory");
                }

                true
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };

        // Acquire exclusive access to the existing vault before taking it down
        let vault_lock = if destination_exists {
            Some(FileLock::exclusive(&paths.root.join(".vault.lock"))?)
        } else {
            None
        };

        if destination_exists {
            let existing_id =
                validate_vault_tree(&paths.root).context("Existing vault validation failed")?;

            if existing_id != backup_id {
                bail!(
                    "Backup belongs to a different vault identity. \
                     Refusing to overwrite the existing vault."
                );
            }
        }

        // Create the restored vault in the target filesystem.
        let temp = TempDirBuilder::new()
            .prefix(".fmp-restore-")
            .tempdir_in(&self.vaults_dir)
            .context("Failed to create restoration temp directory")?;

        let temp_vault = temp.path().join("restored");

        let mut copied_bytes = 0;

        copy_tree_contents(&paths.backup, &temp_vault, true, &mut copied_bytes)
            .context("Failed to copy backup into restoration temp area")?;

        // Give the restored vault its own fresh lock file.
        create_lock_file(&temp_vault.join(".vault.lock"))?;
        validate_vault_tree(&temp_vault).context("Restored vault verification failed")?;
        sync_directory(&temp_vault)?;

        // Trash the existing vault rather than deleting it.
        let previous_vault = if destination_exists {
            let previous = self
                .vaults_dir
                .join(format!(".fmp-previous-vault-{}", Uuid::new_v4()));

            fs::rename(&paths.root, &previous).context("Failed to stage the existing vault")?;

            Some(previous)
        } else {
            None
        };

        // Move the restored vault.
        if let Err(error) = fs::rename(&temp_vault, &paths.root)
            && let Some(previous) = &previous_vault
        {
            if fs::rename(previous, &paths.root).is_err() {
                bail!(
                    "Restoration failed: {error}; \
                    previous vault remains at {}",
                    previous.display()
                );
            }

            return Err(error).context("Failed to move restored vault");
        }

        sync_directory(&self.vaults_dir)?;

        // The old lock file is attached to the trashed directory. Should be dropped before removing that directory.
        drop(vault_lock);

        if let Some(previous) = previous_vault
            && let Err(error) = fs::remove_dir_all(&previous)
        {
            bail!(
                "Vault was restored successfully, but previous \
                     vault cleanup failed at {}: {}",
                previous.display(),
                error
            );
        }

        sync_directory(&self.vaults_dir)?;

        Ok(())
    }

    /// Checks whether a backup exists.
    pub fn exists(&self, name: &str) -> Result<bool> {
        validate_vault_name(name)?;

        self.prepare()?;

        let _lock = FileLock::shared(&self.global_lock)?;

        let paths = self.paths(name)?;

        match fs::symlink_metadata(&paths.backup) {
            Ok(metadata) => {
                if !metadata.file_type().is_dir() {
                    bail!("Backup path is not a real directory");
                }

                Ok(true)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    /// Deletes a backup without deleting its original vault.
    pub fn delete(&self, name: &str) -> Result<()> {
        validate_vault_name(name)?;

        self.prepare()?;

        let _global_lock = FileLock::shared(&self.global_lock)?;

        let _backup_lock = FileLock::exclusive(&self.backups_dir.join(".backup.lock"))?;

        let paths = self.paths(name)?;

        Self::require_real_directory(&paths.backup)?;
        fs::remove_dir_all(&paths.backup).context("Failed to delete vault backup")?;
        sync_directory(&self.backups_dir)?;

        Ok(())
    }
}

/// You'll never guess what this does
fn directory_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(true),
        Ok(_) => {
            bail!("Expected a real directory: {}", path.display())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

/// Creates a new lock file.
fn create_lock_file(path: &Path) -> Result<()> {
    let mut options = OpenOptions::new();

    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let file = options.open(path)?;
    set_secure_file(&file)?;
    file.sync_all()?;

    Ok(())
}

/// Copies a directory tree without following symlinks.
fn copy_tree_contents(
    source: &Path,
    destination: &Path,
    root: bool,
    total_bytes: &mut u64,
) -> Result<()> {
    require_source_directory(source)?;
    create_secure_dir(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;

        let file_name = entry.file_name();

        if root && file_name == ".vault.lock" {
            continue;
        }

        let source_path = entry.path();
        let destination_path = destination.join(&file_name);

        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            bail!("Backup refuses symbolic links: {}", source_path.display());
        }
        if file_type.is_dir() {
            create_secure_dir(&destination_path)?;

            copy_tree_contents(&source_path, &destination_path, false, total_bytes)?;
        } else if file_type.is_file() {
            copy_regular_file(&source_path, &destination_path, total_bytes)?;
        } else {
            bail!(
                "Backup refuses unsupported filesystem entries: {}",
                source_path.display()
            );
        }
    }

    sync_directory(destination)?;

    Ok(())
}

fn require_source_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;

    if !metadata.file_type().is_dir() {
        bail!("Source is not a real directory");
    }

    Ok(())
}

/// Copies and verifies one regular file.
fn copy_regular_file(source: &Path, destination: &Path, total_bytes: &mut u64) -> Result<()> {
    let original_metadata = fs::symlink_metadata(source)?;

    if !original_metadata.file_type().is_file() {
        bail!("Source file is not a regular file");
    }

    let input = File::open(source)?;

    let opened_metadata = input.metadata()?;

    if !opened_metadata.is_file() || opened_metadata.len() != original_metadata.len() {
        bail!("Source file changed during backup");
    }

    // tf is an iNode, apple doing too much these days
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if opened_metadata.dev() != original_metadata.dev()
            || opened_metadata.ino() != original_metadata.ino()
        {
            bail!("Source file was replaced during backup");
        }
    }

    let expected_size = opened_metadata.len();

    *total_bytes = (*total_bytes)
        .checked_add(expected_size)
        .context("Backup size overflow")?;

    if *total_bytes > MAX_BACKUP_BYTES {
        bail!("Backup exceeds maximum allowed size");
    }

    let mut options = OpenOptions::new();

    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut output = options.open(destination)?;

    set_secure_file(&output)?;

    let copied = io::copy(
        &mut input.take(expected_size.saturating_add(1)),
        &mut output,
    )?;

    if copied != expected_size {
        bail!("Source file size changed during backup");
    }

    output.sync_all()?;

    // Compare the actual source and copied destination byte for byte.
    if !files_equal(source, destination)? {
        bail!("Backup verification failed for {}", source.display());
    }

    Ok(())
}

fn read_full_chunk(reader: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;

    while filled < buffer.len() {
        let count = reader.read(&mut buffer[filled..])?;

        if count == 0 {
            break;
        }

        filled += count;
    }

    Ok(filled)
}

fn files_equal(first: &Path, second: &Path) -> Result<bool> {
    let mut first_file = File::open(first)?;
    let mut second_file = File::open(second)?;

    let mut first_buffer = [0u8; 65536];
    let mut second_buffer = [0u8; 65536];

    loop {
        let first_read = read_full_chunk(&mut first_file, &mut first_buffer)?;
        let second_read = read_full_chunk(&mut second_file, &mut second_buffer)?;

        if first_read != second_read {
            return Ok(false);
        }

        if first_buffer[..first_read] != second_buffer[..second_read] {
            return Ok(false);
        }

        if first_read == 0 {
            return Ok(true);
        }
    }
}

/// Validates a vault's structure
fn validate_vault_tree(root: &Path) -> Result<Option<Uuid>> {
    require_source_directory(root)?;

    let manifest_path = root.join("vault.json");

    let manifest_id = match fs::symlink_metadata(&manifest_path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                bail!("Vault manifest is not a regular file");
            }
            if metadata.len() > MAX_MANIFEST_SIZE {
                bail!("Vault manifest exceeds maximum size");
            }
            let data = fs::read(&manifest_path)?;
            let manifest: VaultManifest =
                serde_json::from_slice(&data).context("Invalid vault manifest")?;
            if manifest.api_version != VAULT_API_VERSION {
                bail!("Unsupported vault api version");
            }
            if manifest.created_at.trim().is_empty() {
                bail!("Vault creation timestamp is missing");
            }

            Some(manifest.id)
        }

        Err(error) => return Err(error.into()),
    };

    let accounts_dir = root.join("accounts");
    let recipient_dir = root.join("recipient");

    require_source_directory(&accounts_dir)?;
    require_source_directory(&recipient_dir)?;

    file_ok(&recipient_dir.join("recipient.asc"), MAX_MANIFEST_SIZE)?;
    file_ok(&recipient_dir.join("recipient.key"), MAX_MANIFEST_SIZE)?;
    file_ok(&recipient_dir.join("recipient.rev"), MAX_MANIFEST_SIZE)?;

    // Validate the public/private certificate pair.
    let public_cert = Cert::from_file(recipient_dir.join("recipient.asc"))
        .context("Invalid public certificate in vault")?;
    let secret_cert = Cert::from_file(recipient_dir.join("recipient.key"))
        .context("Invalid private certificate in vault")?;
    if public_cert.is_tsk() || !secret_cert.is_tsk() {
        bail!("Incorrect public/private certificate contents");
    }
    if public_cert.fingerprint() != secret_cert.fingerprint() {
        bail!("Public/private certificate mismatch");
    }
    if !secret_cert
        .keys()
        .secret()
        .all(|key| key.key().secret().is_encrypted())
    {
        bail!("Vault contains unprotected private key material");
    }

    // Require a revocation certificate.
    let revocation_path = recipient_dir.join("recipient.rev");
    let revocation_metadata = fs::symlink_metadata(&revocation_path)?;
    if !revocation_metadata.file_type().is_file() || revocation_metadata.len() == 0 {
        bail!("Invalid or missing revocation certificate");
    }

    // Validate filenames and file types in the accounts directory.
    for entry in fs::read_dir(&accounts_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_symlink() || !file_type.is_file() {
            bail!("Invalid account filesystem entry: {}", path.display());
        }

        // Validate account identity only for recognised account files
        if path.extension().and_then(|ext| ext.to_str()) == Some("pgp") {
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .context("Invalid account filename")?;

            Uuid::parse_str(stem).context("Invalid account UUID filename")?;

            if entry.metadata()?.len() > MAX_ACCOUNT_FILE_SIZE {
                bail!("Account ciphertext exceeds maximum size");
            }
        }
    }

    Ok(manifest_id)
}

fn file_ok(path: &Path, maximum_size: u64) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;

    if !metadata.file_type().is_file() {
        bail!("Expected a regular file: {}", path.display());
    }

    if metadata.len() > maximum_size {
        bail!("File exceeds maximum permitted size");
    }

    Ok(())
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod tests {
    use super::*;

    use crate::{account::Account, secret::SecretString, vault::VaultManager};

    use tempfile::tempdir;

    const TEST_PASSPHRASE: &str = "test-only-passphrase-never-use-in-production";

    fn setup() -> (tempfile::TempDir, VaultManager) {
        let dir = tempdir().unwrap();

        let app_dir = dir.path().join("fmp");

        let manager = VaultManager::at(app_dir);

        (dir, manager)
    }

    fn create_test_vault(manager: &VaultManager) {
        manager
            .create("Personal", "FMP Test <test@example.org>", TEST_PASSPHRASE)
            .unwrap_or_else(|err| {
                panic!("Failed to create test vault:\n{err:#}");
            });
    }

    fn create_test_account() -> Account {
        let mut account = Account::new("Example Account".to_string());

        account.username = SecretString::new("example@example.com".to_string());

        account.password = SecretString::new("ORIGINAL_PASSWORD".to_string());

        account
    }

    #[test]
    fn test_backup_creation() {
        let (_dir, manager) = setup();

        create_test_vault(&manager);

        let paths = manager.paths("Personal").unwrap();

        assert!(!manager.backup_exists("Personal").unwrap());

        manager.create_backup("Personal").unwrap();

        assert!(manager.backup_exists("Personal").unwrap());

        assert!(paths.backup.join("vault.json").is_file());

        assert!(paths.backup.join("recipient/recipient.asc").is_file());

        assert!(paths.backup.join("recipient/recipient.key").is_file());

        assert!(paths.backup.join("recipient/recipient.rev").is_file());

        assert!(paths.backup.join("accounts").is_dir());
    }

    #[test]
    fn test_backup_restoration() {
        let (_dir, manager) = setup();

        create_test_vault(&manager);

        let mut session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        let mut account = create_test_account();
        let id = account.id;

        // Save the initial account through the session-only API.
        session.create_account(&account).unwrap();

        // A backup is allowed while a session is open.
        manager.create_backup("Personal").unwrap();

        // Modify and verify the live account through the same session.
        account.password.replace("MODIFIED_PASSWORD".to_string());

        session.update_account(&mut account).unwrap();

        let current = session.read_account(id).unwrap();

        assert_eq!(current.password.expose_secret(), "MODIFIED_PASSWORD");

        // Restore requires an exclusive global lock, so close the session first.
        session.lock();

        manager.install_backup("Personal").unwrap();

        // Open a fresh session against the restored vault.
        let restored_session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        let restored = restored_session.read_account(id).unwrap();

        assert_eq!(restored.password.expose_secret(), "ORIGINAL_PASSWORD");

        assert_eq!(restored.username.expose_secret(), "example@example.com");

        assert_eq!(restored.id, id);
    }

    #[test]
    fn test_restore_missing_vault() {
        let (dir, manager) = setup();

        create_test_vault(&manager);

        let paths = manager.paths("Personal").unwrap();

        let mut session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        let account = create_test_account();
        let id = account.id;

        session.create_account(&account).unwrap();

        // Back up the vault, then close the session before simulating loss.
        manager.create_backup("Personal").unwrap();

        session.lock();

        let displaced = dir.path().join("displaced-vault");

        fs::rename(&paths.root, &displaced).unwrap();

        assert!(!manager.exists("Personal").unwrap());

        manager.install_backup("Personal").unwrap();

        assert!(manager.exists("Personal").unwrap());

        let restored_session = manager.unlock_vault("Personal", TEST_PASSPHRASE).unwrap();

        let restored = restored_session.read_account(id).unwrap();

        assert_eq!(restored.password.expose_secret(), "ORIGINAL_PASSWORD");
    }

    #[cfg(unix)]
    #[test]
    fn test_replacement_preserves_previous_backup_on_failure() {
        let (_dir, manager) = setup();

        create_test_vault(&manager);

        let paths = manager.paths("Personal").unwrap();

        manager.create_backup("Personal").unwrap();

        let original_manifest = fs::read(paths.backup.join("vault.json")).unwrap();

        // Inject an unexpected symlink into the live vault.
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/passwd", paths.root.join("unexpected-link")).unwrap();

        // Creating a backup should fail.
        assert!(manager.create_backup("Personal").is_err());

        // The previous backup must still be intact.
        let preserved_manifest = fs::read(paths.backup.join("vault.json")).unwrap();

        assert_eq!(original_manifest, preserved_manifest);
    }

    #[test]
    fn test_delete_backup_preserves_original_vault() {
        let (_dir, manager) = setup();

        create_test_vault(&manager);

        manager.create_backup("Personal").unwrap();

        assert!(manager.backup_exists("Personal").unwrap());

        manager.delete_backup("Personal").unwrap();

        assert!(!manager.backup_exists("Personal").unwrap());

        assert!(manager.exists("Personal").unwrap());
    }

    #[test]
    fn test_restore_rejects_different_vault_identity() {
        let (dir, manager) = setup();

        create_test_vault(&manager);

        let original_paths = manager.paths("Personal").unwrap();

        manager.create_backup("Personal").unwrap();

        // Move the backup aside.
        let saved_backup = dir.path().join("saved-backup");

        fs::rename(&original_paths.backup, &saved_backup).unwrap();

        // Delete and recreate the vault under the same name.
        manager.delete("Personal").unwrap();

        create_test_vault(&manager);

        let replacement_paths = manager.paths("Personal").unwrap();

        // Put the old backup under the new vault name.
        fs::rename(&saved_backup, &replacement_paths.backup).unwrap();

        // The backup UUID must not match the newly created vault.
        assert!(manager.install_backup("Personal").is_err());

        assert!(manager.exists("Personal").unwrap());
    }
}
