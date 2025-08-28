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

/// Locks the memory of the provided data to prevent it from being swapped to disk.
///
/// # Arguments
/// * "data" - A slice of bytes representing the data to be locked in memory.
pub fn lock_memory(data: &[u8]) {
    #[cfg(unix)]
    unsafe {
        #[allow(clippy::ptr_as_ptr)]
        libc::mlock(data.as_ptr() as *const _, data.len());
    }

    #[cfg(windows)]
    unsafe {
        use core::ffi::c_void;
        use windows::Win32::System::Memory::VirtualLock;
        #[allow(clippy::ptr_as_ptr)]
        let _ = VirtualLock(data.as_ptr() as *const c_void, data.len());
    }
}
