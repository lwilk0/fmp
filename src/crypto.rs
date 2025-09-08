//! This module provides functions for encrypting and decrypting variables using GPGME.

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

use rand::RngCore;

/// Locks the memory of the provided data to prevent it from being swapped to disk.
///
/// # Arguments
/// * "data" - A slice of bytes representing the data to be locked in memory.
pub fn lock_memory(data: &[u8]) {
    if data.is_empty() {
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
                libc::syscall(libc::SYS_mlock2, data.as_ptr(), data.len(), MLOCK_ONFAULT);
            }
        }
    }

    #[cfg(windows)]
    unsafe {
        use core::ffi::c_void;
        use windows::Win32::System::Memory::VirtualLock;
        #[allow(clippy::ptr_as_ptr)]
        let _ = VirtualLock(data.as_ptr() as *const c_void, data.len());
    }
}

/// Unlocks previously locked memory.
///
/// # Arguments
/// * "data" - A slice of bytes representing the data to be unlocked.
pub fn unlock_memory(data: &[u8]) {
    if data.is_empty() {
        return;
    }

    #[cfg(unix)]
    unsafe {
        #[allow(clippy::ptr_as_ptr)]
        libc::munlock(data.as_ptr() as *const _, data.len());
    }

    #[cfg(windows)]
    unsafe {
        use core::ffi::c_void;
        use windows::Win32::System::Memory::VirtualUnlock;
        #[allow(clippy::ptr_as_ptr)]
        let _ = VirtualUnlock(data.as_ptr() as *const c_void, data.len());
    }
}

/// Securely overwrites memory with random data before zeroization.
///
/// # Arguments
/// * "data" - A mutable slice of bytes to be securely overwritten.
pub fn secure_overwrite(data: &mut [u8]) {
    if data.is_empty() {
        return;
    }

    // First pass: fill with random data
    rand::rng().fill_bytes(data);

    // Second pass: fill with zeros
    data.fill(0);

    // Third pass: fill with 0xFF
    data.fill(0xFF);

    // Final pass: fill with zeros
    data.fill(0);

    // Memory barrier to prevent compiler optimization
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}
