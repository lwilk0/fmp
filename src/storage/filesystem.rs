//! File system utility functions for directory operations.

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
use std::fs::{read_dir, rename};
use std::path::PathBuf;

/// Renames a directory from `old_path` to `new_path`.
///
/// # Arguments
/// * `old_path` - The current path of the directory to be renamed.
/// * `new_path` - The new path for the directory.
///
/// # Returns
/// * `Result<(), Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If the directory at `old_path` does not exist, or if the renaming operation fails, an error is returned.
pub fn rename_directory(old_path: &PathBuf, new_path: &PathBuf) -> Result<(), Error> {
    if old_path.exists() {
        rename(old_path, new_path)?;
    } else {
        return Err(anyhow::anyhow!(
            "The directory `{}` does not exist.",
            old_path.display()
        ));
    }

    Ok(())
}

/// Reads all directories in the specified directory and returns their names as a vector of strings.
///
/// # Arguments
/// * `directory` - The path to the directory to read.
///
/// # Returns
/// * `Result<Vec<String>, Error>` - Returns a vector of directory names on success, or an error on failure.
///
/// # Errors
/// * If reading the directory fails, or if the file type cannot be determined, an error is returned.
pub fn read_directory(directory: &PathBuf) -> Result<Vec<String>, Error> {
    let mut directories = Vec::new();

    for entry in read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let account_name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Failed to convert file name to string."))?;
            directories.push(account_name);
        }
    }

    Ok(directories)
}
