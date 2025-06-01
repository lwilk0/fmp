//! This module contains the GUI implementation for the Forgot-My-Password application.

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

use eframe::egui;
use secrecy::ExposeSecret;
use zeroize::Zeroize;

use crate::{
    content::*,
    crypto::lock_memory,
    vault::{Locations, UserPass, get_account_details, read_directory}, // Ensure get_account_details accepts vault_name and account_name as parameters
};

/// Runs the Forgot-My-Password GUI application.
///
/// # Returns
/// * `Result<(), eframe::Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If there is an error initializing the GUI, it will return an `eframe::Error`.
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Forgot-My-Password",
        options,
        Box::new(|_cc| Ok(Box::new(FmpApp::default()))),
    )
}

/// The main application state for the Forgot-My-Password GUI.
#[derive(Default)]
pub struct FmpApp {
    pub vault_name: String,
    pub account_name: String,
    pub output: String,
    pub vault_names: Vec<String>,
    pub account_names: Vec<String>,
    pub userpass: UserPass,
    pub recipient: String,
    pub vault_name_create: String,
    pub account_name_create: String,

    pub change_account_info: bool,
    pub change_vault_name: bool,
    pub quit: bool,
}

/// Implementation of methods for the `FmpApp` struct to handle fetching vault and account names.
impl FmpApp {
    pub fn fetch_vault_names(&mut self) {
        if let Ok(locations) = Locations::new("", "") {
            if let Ok(names) = read_directory(&locations.fmp_location.join("vaults")) {
                self.vault_names = names;
            } else {
                self.output = "Failed to fetch vault names.".to_string();
            }
        }
    }

    pub fn fetch_account_names(&mut self) {
        if let Ok(locations) = Locations::new(&self.vault_name, "") {
            if let Ok(names) = read_directory(&locations.vault_location) {
                self.account_names = names;
            } else {
                self.output = "Failed to fetch account names.".to_string();
            }
        }
    }
}

/// Implementation of the `eframe::App` trait for the `FmpApp` struct to handle GUI updates.
impl eframe::App for FmpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.quit {
            self.userpass.password.zeroize();
            std::process::exit(1);
        }

        if self.vault_names.is_empty() {
            self.fetch_vault_names();
        }

        if !self.vault_name.is_empty() {
            self.fetch_account_names();
        }

        egui::SidePanel::left("vault_sidebar").show(ctx, |ui| {
            ui.heading("Vaults");
            for vault in &self.vault_names {
                if ui.button(vault).clicked() {
                    self.vault_name = vault.clone();
                    self.output = format!("Selected vault: {}", vault);
                }
            }
        });

        egui::SidePanel::left("account_sidebar").show(ctx, |ui| {
            ui.heading("Accounts");
            if self.vault_name.is_empty() {
                ui.label("Select a vault.");
            } else {
                for account in &self.account_names {
                    if ui.button(account).clicked() {
                        self.account_name = account.clone();

                        self.userpass =
                            match get_account_details(&self.vault_name, &self.account_name) {
                                Ok(userpass) => userpass,
                                Err(e) => {
                                    self.output = format!("Error fetching account details: {}", e);
                                    return;
                                }
                            };

                        lock_memory(self.userpass.password.expose_secret());
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Forgot-My-Password");

            if self.change_vault_name {
                alter_vault_name(self, ui);
            } else if self.change_account_info {
                alter_account_information(self, ui);
            } else if self.vault_name.is_empty() {
                nothing_selected(self, ui);
            } else if self.account_name.is_empty() {
                vault_selected(self, ui);
            } else {
                account_selected(self, ui);
            }
        });
    }
}
