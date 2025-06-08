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
use secrecy::ExposeSecret;

use crate::{
    crypto::securely_retrieve_password,
    flags::{
        add_account, change_account_data, change_account_name, create_backup, create_new_vault,
        delete_account_from_vault, delete_vault, install_backup, rename_vault,
    },
    gui::FmpApp,
    password::{generate_password, password_strength_meter},
};

/// Displays the content for the main window of the application when nothing is selected.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn nothing_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.label("Create a new vault:");

    labeled_text_input(ui, "Vault Name:", &mut app.vault_name_create, None);
    labeled_text_input(
        ui,
        "Email:",
        &mut app.recipient,
        Some(
            "What email address should the vault be associated with? (This should be a public key you have imported into GPG). You can create a public key using the command `gpg --full-generate-key`, or import an existing one using `gpg --import <keyfile>`.",
        ),
    );

    if ui.button("Create Vault").clicked() {
        if app.vault_name_create.is_empty() || app.recipient.is_empty() {
            app.output = "Please fill in all fields before adding an account.".to_string();
            return;
        }
        match create_new_vault(app) {
            Ok(_) => {
                app.output = format!(
                    "Vault `{}` created successfully! NOTE: By default, GPG caches your passphrase for 10 minutes. See `https://github.com/lwilk0/Forgot-My-Password/blob/main/GPGCACHE.md`.",
                    app.vault_name_create
                );

                app.vault_name_create.clear();
                app.recipient.clear();

                app.fetch_vault_names();
            }

            Err(e) => {
                app.output = format!(
                    "Failed to create vault `{}`. Error: {}",
                    app.vault_name_create, e
                )
            }
        }
    }

    display_output(app, ui);

    quit_button(app, ui);
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

    labeled_text_input(ui, "Account Name:", &mut app.account_name_create, None);
    labeled_text_input(ui, "Username:", &mut app.userpass.username, None);
    securely_retrieve_password(app, ui, "Account Password:");
    generate_password_slider(app, ui);

    if ui.button("Add Account").clicked() {
        if app.account_name_create.is_empty() {
            app.output = "Please fill in all fields before adding an account.".to_string();
            return;
        }
        match add_account(app) {
            Ok(_) => {
                app.output = format!(
                    "Account `{}` added to vault `{}`.",
                    app.account_name, app.vault_name
                );

                app.clear_account_data();
                app.account_name_create.clear();

                app.fetch_account_names();
            }

            Err(e) => {
                app.output = format!(
                    "Failed to add account `{}` to vault `{}`. Error: {}",
                    app.account_name, app.vault_name, e
                )
            }
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
                app.output = format!("Vault `{}` deleted.", app.vault_name);

                app.vault_name.clear();
                app.fetch_vault_names();
            }
            Err(e) => {
                app.output = format!("Failed to delete vault `{}`. Error: {}", app.vault_name, e)
            }
        }
    }

    ui.separator();

    ui.label("Backup Options:");

    if ui.button("Backup Vault").clicked() {
        match create_backup(app) {
            Ok(_) => app.output = format!("Vault `{}` backed up successfully.", app.vault_name),
            Err(e) => {
                app.output = format!("Failed to back up vault `{}`. Error: {}", app.vault_name, e)
            }
        }
    }

    if ui.button("Restore Vault").clicked() {
        match install_backup(app) {
            Ok(_) => app.output = format!("Vault `{}` restored successfully.", app.vault_name),
            Err(e) => {
                app.output = format!("Failed to restore vault `{}`. Error: {}", app.vault_name, e)
            }
        }
    }

    display_output(app, ui);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.vault_name.clear();
            }

            quit_button(app, ui);
        });
    });
}

/// Displays the content for the main window of the application when an account is selected.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn account_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.label(format!("Account: {}", app.account_name));
    ui.label(format!("Username: {}", app.userpass.username));

    ui.horizontal(|ui| {
        if app.show_password {
            let password = String::from_utf8_lossy(app.userpass.password.expose_secret());

            ui.label(format!("Password: {}", password));

            app.show_password = show_password_button(app.show_password, ui, "Hide");

            password_strength_meter(ui, &password); // TODO: cache
        } else {
            ui.label("Password:");
            app.show_password = show_password_button(app.show_password, ui, "Show");
        }
    });

    if ui.button("Change Information").clicked() {
        app.account_name_create = app.account_name.clone();
        app.change_account_info = true;
    }

    if ui.button("Delete Account").clicked() {
        match delete_account_from_vault(app) {
            Ok(_) => {
                app.output = format!(
                    "Account `{}` deleted from vault `{}`.",
                    app.account_name, app.vault_name
                );

                app.account_name.clear();
                app.clear_account_data();
                app.fetch_account_names();
            }

            Err(e) => {
                app.output = format!(
                    "Failed to delete account `{}` from vault {}. Error: {}",
                    app.account_name, app.vault_name, e
                )
            }
        }
    }

    display_output(app, ui);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.account_name.clear();
            }

            quit_button(app, ui);
        });
    });
}

/// Displays the content for altering account information.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn alter_account_information(app: &mut FmpApp, ui: &mut egui::Ui) {
    labeled_text_input(ui, "Account Name:", &mut app.account_name_create, None);
    labeled_text_input(ui, "Username:", &mut app.userpass.username, None);
    securely_retrieve_password(app, ui, "New Password:");
    generate_password_slider(app, ui);

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
                            app.output = format!("Failed to change account name. Error: {}", e);
                            return;
                        }
                    }
                }

                app.output = format!(
                    "Account `{}` updated successfully in vault `{}`.",
                    app.account_name, app.vault_name
                );

                app.change_account_info = false;
                app.fetch_account_names();
            }

            Err(e) => {
                app.output = format!(
                    "Failed to change account name `{}`. Error: {}",
                    app.account_name, e
                );
            }
        }
    }

    display_output(app, ui);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.change_account_info = false;
            }

            quit_button(app, ui);
        });
    });
}

/// Displays the content for altering the vault name.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn alter_vault_name(app: &mut FmpApp, ui: &mut egui::Ui) {
    labeled_text_input(ui, "New Vault Name:", &mut app.vault_name_create, None);

    if ui.button("Rename Vault").clicked() {
        if app.vault_name_create.is_empty() {
            app.output = "Please fill in all fields before changing vault name.".to_string();
            return;
        }

        match rename_vault(app) {
            Ok(_) => {
                app.output = format!("Vault renamed to `{}`.", app.vault_name_create);
                app.vault_name = app.vault_name_create.clone();
                app.vault_name_create.clear();
                app.fetch_vault_names();
                app.change_vault_name = false;
            }
            Err(e) => {
                app.output = format!(
                    "Failed to rename vault `{}`. Error: {}",
                    app.vault_name_create, e
                );
            }
        }
    }

    display_output(app, ui);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.change_vault_name = false;
            }

            quit_button(app, ui);
        });
    });
}

/// Creates a quit button
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn quit_button(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        if ui.button("Quit").clicked() {
            app.quit = true;
        }
    });
}

/// Creates labeled text input field.
///
/// # Arguments
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
/// * `label` - The label for the text input field.
/// * `value` - A mutable reference to the value of the text input field.
/// * `hover` - An optional string to display as a hover text for the text input field.
fn labeled_text_input(ui: &mut egui::Ui, label: &str, value: &mut String, hover: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(label);

        let text_edit = ui.text_edit_singleline(value);

        if let Some(hover_text) = hover {
            text_edit.on_hover_text(hover_text);
        }
    });
}

/// Displays the output of the application.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
fn display_output(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.separator();
    ui.label(&app.output);
}

/// Displays a show/hide password button
///
/// # Arguments
/// * `state` - The current state of the password(True - Show, False - Hide)
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
/// * `label` - The label for the text input field.
///
/// # Returns
/// * A `bool` of the new state.
fn show_password_button(state: bool, ui: &mut egui::Ui, label: &str) -> bool {
    if ui.button(label).clicked() {
        return !state;
    }
    state
}

fn generate_password_slider(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Generate Password").clicked() {
            generate_password(app);
        }
        ui.add(egui::Slider::new(&mut app.password_length, 8..=128).text("chars"));
    });
}
