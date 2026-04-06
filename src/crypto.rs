//! This module provides functions for encrypting and decrypting variables using GPGME.
//!
//! Security features include:
//! - Memory locking to prevent sensitive data from being swapped to disk
//! - Secure memory wiping using volatile writes to prevent compiler optimizations
//! - Cross-platform support for memory protection
//! - Automatic cleanup of sensitive data when no longer needed

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

use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::Zeroize;

// Global flags to track security features availability
static MEMORY_LOCKING_AVAILABLE: AtomicBool = AtomicBool::new(true);

/// Locks the memory of the provided data to prevent it from being swapped to disk.
///
/// This function attempts to lock the memory pages containing the sensitive data
/// to prevent them from being swapped to disk, where they might be recovered later.
/// If memory locking fails, it will set a global flag and log a warning, but continue
/// execution.
///
/// # Arguments
/// * "data" - A slice of bytes representing the data to be locked in memory.
///
/// # Security Notes
/// * Memory locking requires appropriate permissions on most systems
/// * On failure, the function will continue but set a global flag
/// * Even with memory locking, data might still be accessible through other means
///   such as hibernation files, core dumps, or direct memory access
pub fn lock_memory(data: &[u8]) {
    if data.is_empty() {
        return;
    }

    // TODO: Add similar to windows
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }

    // Skip if previous attempts have failed
    if !MEMORY_LOCKING_AVAILABLE.load(Ordering::Relaxed) {
        return;
    }

    #[cfg(unix)]
    unsafe {
        #[allow(clippy::ptr_as_ptr)]
        let result = libc::mlock(data.as_ptr() as *const _, data.len());
        if result != 0 {
            // If mlock fails, try mlock2 with MLOCK_ONFAULT on Linux
            #[cfg(target_os = "linux")]
            {
                const MLOCK_ONFAULT: i32 = 1;
                // Use explicit c_void pointer type for clarity
                let mlock2_result = libc::syscall(
                    libc::SYS_mlock2,
                    data.as_ptr() as *const libc::c_void,
                    data.len(),
                    MLOCK_ONFAULT,
                );

                if mlock2_result != 0 {
                    // Both mlock and mlock2 failed, mark memory locking as unavailable
                    MEMORY_LOCKING_AVAILABLE.store(false, Ordering::Relaxed);
                    log::error!(
                        "Warning: Failed to lock memory. Sensitive data may be swapped to disk.",
                    );
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                // On non-Linux Unix systems, if mlock fails, mark as unavailable
                MEMORY_LOCKING_AVAILABLE.store(false, Ordering::Relaxed);
                log::error!(
                    "Warning: Failed to lock memory. Sensitive data may be swapped to disk."
                );
            }
        }
    }

    #[cfg(windows)]
    unsafe {
        use core::ffi::c_void;
        use windows::Win32::System::Memory::VirtualLock;
        #[allow(clippy::ptr_as_ptr)]
        let lock_result = VirtualLock(data.as_ptr() as *const c_void, data.len());

        if !lock_result.as_bool() {
            MEMORY_LOCKING_AVAILABLE.store(false, Ordering::Relaxed);
            log::error!("Warning: Failed to lock memory. Sensitive data may be swapped to disk.");
        }
    }
}

/// Unlocks previously locked memory.
///
/// This function releases the lock on memory pages that were previously locked
/// with `lock_memory`. It should be called before the memory is freed or
/// reallocated to avoid memory leaks.
///
/// # Arguments
/// * "data" - A slice of bytes representing the data to be unlocked.
///
/// # Security Notes
/// * This function should be called before the memory is freed
/// * After unlocking, the memory may be swapped to disk
/// * For maximum security, sensitive data should be overwritten with zeros
///   using `secure_overwrite` before unlocking
pub fn unlock_memory(data: &[u8]) {
    if data.is_empty() {
        return;
    }

    if !MEMORY_LOCKING_AVAILABLE.load(Ordering::Relaxed) {
        return;
    }

    #[cfg(unix)]
    unsafe {
        #[allow(clippy::ptr_as_ptr)]
        let result = libc::munlock(data.as_ptr() as *const _, data.len());
        if result != 0 {
            log::error!("Warning: Failed to unlock memory. This may cause memory leaks.");
        }
    }

    #[cfg(windows)]
    unsafe {
        use core::ffi::c_void;
        use windows::Win32::System::Memory::VirtualUnlock;
        #[allow(clippy::ptr_as_ptr)]
        let unlock_result = VirtualUnlock(data.as_ptr() as *const c_void, data.len());

        if !unlock_result.as_bool() {
            log::error!("Warning: Failed to unlock memory. This may cause memory leaks.");
        }
    }
}

/// Securely zeroizes sensitive memory.
///
/// Uses the `zeroize` crate to prevent compiler optimizations from
/// eliding the wipe. Multiple overwrite passes are unnecessary and
/// slower; a single guaranteed zeroization is sufficient.
///
/// # Arguments
/// * "data" - A mutable slice of bytes to be securely wiped.
///
/// # Security Notes
/// * This function should be called before freeing memory containing sensitive data
/// * The zeroize crate uses volatile writes to prevent compiler optimizations
/// * For maximum security, call this before `unlock_memory`
/// * This function does not protect against hardware-level attacks or memory
///   inspection via specialized equipment
#[inline]
pub fn secure_overwrite(data: &mut [u8]) {
    if data.is_empty() {
        return;
    }

    // Zeroize uses volatile writes to ensure the compiler does not remove the wipe
    data.zeroize();
}

/// RAII guard that locks memory on creation and securely overwrites + unlocks on drop.  
///  
/// This eliminates manual lock/unlock pairing errors that cause mlock leaks.  
/// The buffer is locked immediately on construction and guaranteed to be  
/// zeroized-then-unlocked on drop, even during panics or early returns.  
///  
/// # Security Notes  
/// * Data is zeroized while still locked (preventing swap exposure during cleanup)  
/// * Drop order is always: secure_overwrite → unlock_memory  
/// * The guard prevents the compiler from optimizing away the zeroization  
/// * Clone is intentionally NOT implemented to prevent untracked copies  
pub struct LockedBuffer {
    data: Vec<u8>,
}

#[allow(dead_code)]
impl LockedBuffer {
    /// Creates a new `LockedBuffer`, immediately locking the data in memory.  
    ///  
    /// # Arguments  
    /// * `data` - The sensitive data to protect. Ownership is transferred to the guard.  
    pub fn new(data: Vec<u8>) -> Self {
        lock_memory(&data);
        Self { data }
    }

    /// Creates a `LockedBuffer` from a `String`, consuming it.  
    ///  
    /// This avoids the need to manually convert strings to bytes and prevents  
    /// the original String from lingering unzeroized.  
    pub fn from_string(s: String) -> Self {
        let bytes = s.into_bytes();
        lock_memory(&bytes);
        Self { data: bytes }
    }

    /// Returns a read-only slice of the locked data.  
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable slice of the locked data.  
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns the length of the locked data.  
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the locked data is empty.  
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Attempts to interpret the locked data as a UTF-8 string slice.  
    ///  
    /// # Returns  
    /// * `Ok(&str)` if the data is valid UTF-8  
    /// * `Err` if the data is not valid UTF-8  
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Provides scoped read access to the data via a closure.  
    ///  
    /// This pattern ensures the caller cannot hold a reference beyond the  
    /// closure's lifetime, reducing the risk of accidental exposure.  
    pub fn with_data<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }

    /// Consumes the guard, zeroizes and unlocks the data, returning nothing.  
    ///  
    /// Use this for explicit early cleanup when you want to release the  
    /// locked memory before the guard would naturally go out of scope.  
    pub fn destroy(mut self) {
        secure_overwrite(&mut self.data);
        unlock_memory(&self.data);
        // Prevent Drop from running again on empty data
        self.data = Vec::new();
    }
}

impl Drop for LockedBuffer {
    fn drop(&mut self) {
        if !self.data.is_empty() {
            // Critical: zeroize WHILE STILL LOCKED, then unlock
            secure_overwrite(&mut self.data);
            unlock_memory(&self.data);
        }
    }
}

impl std::fmt::Debug for LockedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockedBuffer")
            .field("len", &self.data.len())
            .field("data", &"[REDACTED]")
            .finish()
    }
}
