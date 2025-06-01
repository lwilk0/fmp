//! This module provides the content for the main window of the application.

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
use secrecy::{ExposeSecret, SecretBox};

use crate::{
    crypto::securely_retrieve_password,
    flags::{
        add_account, change_account_data, change_account_name, create_backup, create_new_vault,
        delete_account_from_vault, delete_vault, install_backup, rename_vault,
    },
    gui::FmpApp,
    password::calculate_entropy,
};

/// Displays the content for the main window of the application when nothing is selected.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn nothing_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.label("Create a new vault:");

    ui.horizontal(|ui| {
        ui.label("Vault Name:");
        ui.text_edit_singleline(&mut app.vault_name_create);
    });

    ui.horizontal(|ui| {
        ui.label("Email:");
        ui.text_edit_singleline(&mut app.recipient).on_hover_text(
            "What email address should the vault be associated with? (This should be a public key you have imported into GPG). You can create a public key using the command `gpg --full-generate-key`, or import an existing one using `gpg --import <keyfile>`.",
        );
    });

    if ui.button("Create Vault").clicked() {
        if app.vault_name_create.is_empty() || app.recipient.is_empty() {
            app.output = "Please fill in all fields before adding an account.".to_string();
            return;
        }
        match create_new_vault(app) {
            Ok(_) => {
                app.output = format!("Vault '{}' created successfully!", app.vault_name_create);

                app.vault_name_create.clear();
                app.recipient.clear();

                app.fetch_vault_names();
            }
            Err(e) => app.output = format!("Error: {}", e),
        }
    }

    ui.separator();
    ui.label(&app.output);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        if ui.button("Quit").clicked() {
            app.quit = true;
        }
    });
}

/// Displays the content for the main window of the application when a vault is selected.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn vault_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.label(format!("Vault: {}", app.vault_name));

    ui.separator();

    ui.label("Add a new account:");

    ui.horizontal(|ui| {
        ui.label("Account Name:");
        ui.text_edit_singleline(&mut app.account_name_create);
    });

    ui.horizontal(|ui| {
        ui.label("Account Username:");
        ui.text_edit_singleline(&mut app.userpass.username);
    });

    securely_retrieve_password(app, ui, "Account Password:");

    if ui.button("Add Account").clicked() {
        if app.account_name_create.is_empty() {
            app.output = "Please fill in all fields before adding an account.".to_string();
            return;
        }
        match add_account(app) {
            Ok(_) => {
                app.output = format!(
                    "Account '{}' added to vault '{}'.",
                    app.account_name, app.vault_name
                );

                app.userpass.username.clear();
                app.userpass.password = SecretBox::new(Box::new(vec![]));
                app.fetch_account_names();
            }

            Err(e) => app.output = format!("Error: {}", e),
        }
    }

    ui.separator();

    ui.label("Vault Options:");

    if ui.button("Change Vault Name").clicked() {
        app.change_vault_name = true;
    }

    if ui.button("Delete Vault").clicked() {
        match delete_vault(app) {
            Ok(()) => {
                app.output = format!("Vault '{}' deleted.", app.vault_name);

                app.vault_name.clear();
                app.fetch_vault_names();
            }
            Err(e) => app.output = format!("Error: {}", e),
        }
    }

    ui.separator();

    ui.label("Backup Options:");

    if ui.button("Backup Vault").clicked() {
        match create_backup(app) {
            Ok(_) => app.output = format!("Vault '{}' backed up successfully.", app.vault_name),
            Err(e) => app.output = format!("Error backing up vault: {}", e),
        }
    }

    if ui.button("Restore Vault").clicked() {
        match install_backup(app) {
            Ok(_) => app.output = format!("Vault '{}' restored successfully.", app.vault_name),
            Err(e) => app.output = format!("Error restoring vault: {}", e),
        }
    }

    ui.separator();
    ui.label(&app.output);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.vault_name.clear();
            }
            if ui.button("Quit").clicked() {
                app.quit = true;
            }
        });
    });
}

/// Displays the content for the main window of the application when an account is selected.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn account_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.label(format!("{}:", app.account_name));
    ui.label(format!("Username: {}", app.userpass.username));

    let password = String::from_utf8_lossy(app.userpass.password.expose_secret());
    let (entropy, rating) = calculate_entropy(&password);

    ui.horizontal(|ui| {
        ui.label(format!(
            "Password: {}",
            password // TODO: Should be copy button
        ));

        ui.label(format!("Entropy: {:.2} bits, Rating: {}", entropy, rating)); // TODO: cache
    });

    if ui.button("Change Information").clicked() {
        app.account_name_create = app.account_name.clone();
        app.change_account_info = true;
    }

    if ui.button("Delete Account").clicked() {
        match delete_account_from_vault(app) {
            Ok(_) => {
                app.output = format!(
                    "Account '{}' deleted from vault '{}'.",
                    app.account_name, app.vault_name
                );

                app.account_name.clear();
                app.userpass.username.clear();
                app.userpass.password = SecretBox::new(Box::new(vec![]));

                app.fetch_account_names();
            }
            Err(e) => app.output = format!("Error: {}", e),
        }
    }

    ui.separator();
    ui.label(&app.output);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.account_name.clear();
            }
            if ui.button("Quit").clicked() {
                app.quit = true;
            }
        });
    });
}

/// Displays the content for altering account information.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn alter_account_information(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("New Account Name:");
        ui.text_edit_singleline(&mut app.account_name_create);
    });

    ui.horizontal(|ui| {
        ui.label("New Username:");
        ui.text_edit_singleline(&mut app.userpass.username);
    });

    securely_retrieve_password(app, ui, "New Password:");

    if ui.button("Change Information").clicked() {
        if app.account_name_create.is_empty() {
            app.output =
                "Please fill in all fields before changing an accounts information.".to_string();
            return;
        }
        match change_account_data(app) {
            Ok(_) => {
                if app.account_name != app.account_name_create {
                    match change_account_name(app) {
                        Ok(_) => {
                            app.account_name = app.account_name_create.clone();
                            app.account_name_create.clear();
                        }
                        Err(e) => {
                            app.output = format!("Error changing account name: {}", e);
                            return;
                        }
                    }
                }

                app.output = format!(
                    "Account '{}' updated successfully in vault '{}'.",
                    app.account_name, app.vault_name
                );

                app.change_account_info = false;
                app.fetch_account_names();
            }
            Err(e) => {
                app.output = format!("Error: {}", e);
            }
        }
    }

    ui.separator();
    ui.label(&app.output);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.change_account_info = false;
            }
            if ui.button("Quit").clicked() {
                app.quit = true;
            }
        });
    });
}

/// Displays the content for altering the vault name.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn alter_vault_name(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("New Vault Name:");
        ui.text_edit_singleline(&mut app.vault_name_create);
    });

    if ui.button("Rename Vault").clicked() {
        if app.vault_name_create.is_empty() {
            app.output = "Please fill in all fields before changing vault name.".to_string();
            return;
        }

        match rename_vault(app) {
            Ok(_) => {
                app.output = format!("Vault renamed to '{}'.", app.vault_name_create);

                app.vault_name = app.vault_name_create.clone();

                app.vault_name_create.clear();

                app.fetch_vault_names();

                app.change_vault_name = false;
            }
            Err(e) => {
                app.output = format!("Error: {}", e);
            }
        }
    }

    ui.separator();
    ui.label(&app.output);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.change_vault_name = false;
            }
            if ui.button("Quit").clicked() {
                app.quit = true;
            }
        });
    });
}
