//! Forgot-My-Password(FMP) - A simple password vault application.

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

use eframe::Error;
use env_logger::{Builder, Env};
use std::io::Write;

mod crypto;
mod flags;
mod password;
mod vault;

#[path = "Gui/content.rs"]
mod content;
#[path = "Gui/gui.rs"]
mod gui;

use crate::gui::run_gui;

fn main() -> Result<(), Error> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
        .init();

    match run_gui() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error running GUI: {}", e);
            Err(e)
        }
    }
}
