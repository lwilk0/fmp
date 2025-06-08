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

use crate::{
    content::*,
    vault::{Locations, UserPass, get_account_details, read_directory},
};
use eframe::egui;
use log::error;
use secrecy::SecretBox;
use zeroize::Zeroize;

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
    pub password_length: u8,

    pub change_account_info: bool,
    pub change_vault_name: bool,
    pub quit: bool,
    pub show_password: bool,
    pub show_welcome: String,
}

/// Implementation of methods for the `FmpApp` struct to handle fetching vault and account names.
impl FmpApp {
    /// Find all the vault names.
    pub fn fetch_vault_names(&mut self) {
        if let Ok(locations) = Locations::new("", "") {
            if let Ok(names) = read_directory(&locations.fmp_location.join("vaults")) {
                self.vault_names = names;
            } else {
                self.output = "Failed to fetch vault names.".to_string();
            }
        }
    }

    /// Find all the account names in a vault.
    pub fn fetch_account_names(&mut self) {
        if let Ok(locations) = Locations::new(&self.vault_name, "") {
            if let Ok(names) = read_directory(&locations.vault_location) {
                self.account_names = names;
            } else {
                self.output = "Failed to fetch account names.".to_string();
            }
        }
    }

    /// Clear the data in userpass
    pub fn clear_account_data(&mut self) {
        self.userpass.username.clear();
        self.userpass.password = SecretBox::new(Box::new(vec![]));
    }

    /// Check if this is FMPs first run.
    pub fn check_first_run(&mut self) {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("fmp_ran");

        if config_path.exists() {
            self.show_welcome = String::from("false");
        } else {
            let _ = std::fs::write(&config_path, "shown");
            self.show_welcome = String::from("true");
        }
    }
}

/// Implementation of the `eframe::App` trait for the `FmpApp` struct to handle GUI updates.
impl eframe::App for FmpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_welcome.is_empty() {
            self.check_first_run();
        }

        if self.vault_names.is_empty() {
            self.fetch_vault_names();
        }

        if !self.vault_name.is_empty() && self.account_names.is_empty() {
            self.fetch_account_names();
        }

        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Vaults");
                for vault in &self.vault_names {
                    if ui.button(vault).clicked() {
                        self.vault_name = vault.clone();
                        self.account_names.clear();
                    }
                }

                ui.separator();

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
                                        error!("Failed to fetch account details. Error: {}", e);
                                        return;
                                    }
                                };
                        }
                    }
                }
            });
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

        if self.quit {
            egui::Window::new("Confirm Exit")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to quit?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            self.userpass.password.zeroize();
                            std::process::exit(0);
                        }
                        if ui.button("No").clicked() {
                            self.quit = false;
                        }
                    });
                });
        }

        if self.show_welcome == "true" {
            egui::Window::new("Welcome to Forgot-My-Password!")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Welcome!");
                    ui.label("Thank you for installing Forgot-My-Password.\n\nThis is a secure password manager. Get started by creating your first vault and adding an account to it.");
                    if ui.button("Get Started").clicked() {
                        self.show_welcome = String::from("false");
                    }
                });
        }
    }
}
