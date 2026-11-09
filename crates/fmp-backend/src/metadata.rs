use crate::paths::atomic_write;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const MAX_METADATA_SIZE: u64 = 1024 * 1024;
const MAX_RECENT_VAULTS: usize = 10;

/// Non-sensitive application metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VaultMetadata {
    #[serde(default)]
    pub usage_counts: BTreeMap<String, u64>,

    #[serde(default)]
    pub recent_vaults: Vec<String>,
}

impl VaultMetadata {
    /// Increments the usage count for a vault.
    pub fn increment_usage(&mut self, name: &str) {
        let count = self.usage_counts.entry(name.to_string()).or_insert(0);

        *count = count.saturating_add(1);
    }

    /// Places a vault atop the recent vaults vector
    pub fn record_recent(&mut self, name: &str) {
        self.recent_vaults.retain(|existing| existing != name);
        self.recent_vaults.insert(0, name.to_string());
        self.recent_vaults.truncate(MAX_RECENT_VAULTS);
    }

    /// Records a successful vault opening.
    pub fn record_opened(&mut self, name: &str) {
        self.increment_usage(name);
        self.record_recent(name);
    }

    /// Removes a vault from statistics and recent history.
    pub fn remove_vault(&mut self, name: &str) {
        self.usage_counts.remove(name);

        self.recent_vaults.retain(|existing| existing != name);
    }

    /// Updates references when a vault is renamed.
    pub fn rename_vault(&mut self, old_name: &str, new_name: &str) {
        if let Some(count) = self.usage_counts.remove(old_name) {
            self.usage_counts
                .entry(new_name.to_string())
                .or_insert(count);
        }

        let mut updated = Vec::new();

        for name in &self.recent_vaults {
            let replacement = if name == old_name {
                new_name
            } else {
                name.as_str()
            };

            if !updated.iter().any(|item| item == replacement) {
                updated.push(replacement.to_string());
            }
        }

        updated.truncate(MAX_RECENT_VAULTS);

        self.recent_vaults = updated;
    }

    pub fn recent_vaults(&self, limit: usize) -> Vec<String> {
        self.recent_vaults.iter().take(limit).cloned().collect()
    }

    /// Returns the most-used vault.
    /// Ties are resolved alphabetically
    pub fn most_used_vault(&self) -> Option<String> {
        let mut best: Option<(&String, u64)> = None;

        for (name, count) in &self.usage_counts {
            if *count == 0 {
                continue;
            }

            match best {
                None => best = Some((name, *count)),

                Some((_, current_count)) if *count > current_count => {
                    best = Some((name, *count));
                }

                _ => {}
            }
        }

        best.map(|(name, _)| name.clone())
    }

    pub fn usage_count(&self, name: &str) -> u64 {
        self.usage_counts.get(name).copied().unwrap_or(0)
    }
}

/// Reads and writes the metadata.
pub(crate) struct MetadataStore {
    path: PathBuf,
}

impl MetadataStore {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            path: app_dir.join("metadata.json"),
        }
    }

    pub fn load(&self) -> Result<VaultMetadata> {
        match fs::symlink_metadata(&self.path) {
            Ok(metadata) => {
                if !metadata.file_type().is_file() {
                    bail!("Metadata path is not a regular file");
                }

                if metadata.len() > MAX_METADATA_SIZE {
                    bail!("Metadata file exceeds maximum size");
                }

                let data = fs::read(&self.path)?;

                let parsed =
                    serde_json::from_slice(&data).context("Failed to parse vault metadata")?;

                Ok(parsed)
            }

            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(VaultMetadata::default())
            }

            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, metadata: &VaultMetadata) -> Result<()> {
        let serialized = serde_json::to_vec(metadata)?;

        if serialized.len() as u64 > MAX_METADATA_SIZE {
            bail!("Metadata exceeds maximum allowed size");
        }

        atomic_write(&self.path, &serialized)
    }
}
