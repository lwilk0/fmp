use anyhow::{Context, Result, bail};

use fs2::FileExt;

use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

/// Validates a vault name, ensuring it will work will all OSes
pub fn validate_vault_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 128
        || name.trim() != name // No spaces
        || name.starts_with('.')
        || name == "."
        || name == ".."
        || name.ends_with('.')
        || name.ends_with(' ')
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.chars().any(char::is_control)
    {
        bail!("Invalid vault name");
    }

    // Reject Windows reserved device names
    let part = name.split('.').next().unwrap_or(name).to_ascii_uppercase();

    let reserved = matches!(part.as_str(), "CON" | "PRN" | "AUX" | "NUL");

    let numbered_reserved = ["COM", "LPT"].iter().any(|prefix| {
        part.strip_prefix(prefix)
            .is_some_and(|suffix| suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9'))
    });

    if reserved || numbered_reserved {
        bail!("Vault name is a reserved filesystem name");
    }

    Ok(())
}

/// Creates a directory with restrictive permissions
pub fn create_secure_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

        let mut builder = fs::DirBuilder::new();

        builder.recursive(true);
        builder.mode(0o700);

        match builder.create(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to create directory: {}", path.display()));
            }
        }

        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("Failed to inspect directory: {}", path.display()))?;

        if !metadata.file_type().is_dir() {
            bail!("Expected a real directory: {}", path.display());
        }

        // Ensure an existing directory also has correct permissions.
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("Failed to set directory permissions: {}", path.display()))?;
    }

    #[cfg(not(unix))]
    {
        match fs::create_dir_all(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }

        let metadata = fs::symlink_metadata(path)?;

        if !metadata.file_type().is_dir() {
            bail!("Expected a real directory: {}", path.display());
        }
    }

    Ok(())
}

/// Restricts a file to its owner on Unix.
pub fn set_secure_file(file: &File) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }

    // TODO binbows

    Ok(())
}

/// Synchronizes a directory after filesystem modifications
pub fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;

    #[cfg(not(unix))]
    let _ = path; // TODO binbows

    Ok(())
}

/// Atomically writes data to a file.
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    use std::io::Write;

    let parent = path
        .parent()
        .context("Destination has no parent directory")?;

    create_secure_dir(parent)?;

    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    set_secure_file(temp.as_file())?;

    temp.write_all(contents)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|error| error.error)?;

    sync_directory(parent)?;

    Ok(())
}

/// RAII guard to prevent breakage of a vault from operations
///
/// Three different locks for different vault actions:
/// - .global.lock - Global, across all vaults for operations outwith a singular one
/// - .vault.lock - Self explanitory - for a single vault
/// - .backup.lock - For backup operations
pub struct FileLock {
    file: File,
}

impl FileLock {
    // small helpers fns for code clarity
    pub fn exclusive(path: &Path) -> Result<Self> {
        Self::acquire(path, true)
    }
    pub fn shared(path: &Path) -> Result<Self> {
        Self::acquire(path, false)
    }

    /// Acquire a lock on the vault
    /// Shared - read access - can coexit with other share locks
    /// Exclusive - write access - no other locks can exist
    fn acquire(path: &Path, exclusive: bool) -> Result<Self> {
        // Validate parent dir
        let parent = path.parent().context("Lock path has no parent directory")?;
        let parent_metadata = fs::symlink_metadata(parent)?;
        if !parent_metadata.file_type().is_dir() {
            bail!("Lock parent is not a real directory");
        }

        // Reject a symlink or non-regular lock file.
        match fs::symlink_metadata(path) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                bail!("Lock path is not a regular file");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        // TODO binbows

        let file = options.open(path)?;
        set_secure_file(&file)?;

        if exclusive {
            FileExt::lock_exclusive(&file).context("Failed to acquire exclusive lock")?;
        } else {
            FileExt::lock_shared(&file).context("Failed to acquire shared lock")?;
        }

        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// Paths belonging to a single vault.
#[derive(Debug, Clone)]
pub struct VaultPaths {
    pub root: PathBuf,
    pub accounts: PathBuf,
    pub recipient: PathBuf,
    pub backup: PathBuf,
}

impl VaultPaths {
    pub fn new(app_dir: &Path, vault_name: &str) -> Result<Self> {
        validate_vault_name(vault_name)?;

        let root = app_dir.join("vaults").join(vault_name);

        Ok(Self {
            accounts: root.join("accounts"),
            recipient: root.join("recipient"),
            backup: app_dir.join("backups").join(vault_name),
            root,
        })
    }
}

// Most tests LLM generated, I would LOVE to write them manually but im.... busy
#[cfg(test)]
mod secure_dir_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_missing_directory() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("fmp").join("vaults");

        create_secure_dir(&path).unwrap();

        assert!(path.is_dir());
    }

    #[test]
    fn test_existing_directory_accepted() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("fmp");

        create_secure_dir(&path).unwrap();
        create_secure_dir(&path).unwrap();

        assert!(path.is_dir());
    }

    #[test]
    fn test_existing_file_rejected() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("not-a-directory");

        fs::write(&path, "test").unwrap();

        assert!(create_secure_dir(&path).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_final_symlink_rejected() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();

        let real_dir = temp.path().join("real");
        let link = temp.path().join("link");

        fs::create_dir(&real_dir).unwrap();
        symlink(&real_dir, &link).unwrap();

        assert!(create_secure_dir(&link).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_directory_permissions_are_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempdir().unwrap();
        let path = temp.path().join("fmp");

        create_secure_dir(&path).unwrap();

        let metadata = fs::metadata(&path).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
    }
}
