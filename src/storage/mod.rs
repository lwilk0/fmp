//! Storage module providing file system operations and encryption functionality.
//!
//! This module handles all file system operations, path management, and GPG encryption
//! for the FMP application.

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

pub mod filesystem;
pub mod locations;
pub mod store;

pub use filesystem::{read_directory, rename_directory};
pub use locations::Locations;
pub use store::Store;
