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
use crate::{
    crypto::securely_retrieve_password,
    flags::{
        add_account, change_account_data, change_account_name, create_backup, create_new_vault,
        delete_account_from_vault, delete_vault, install_backup, rename_vault,
    },
    gui::FmpApp,
    password::{generate_password, password_strength_meter},
};
use core::mem;
use secrecy::ExposeSecret;
use std::time::Duration;
use zeroize::Zeroize;

/// Displays the content for the main window of the application when nothing is selected.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state.
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
pub fn nothing_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Create a New Vault");
        });

        ui.separator();
        ui.add_space(8.0);

        labeled_text_input(ui, "Vault Name:", &mut app.vault_name_create, None);
        ui.add_space(4.0);
        labeled_text_input(
            ui,
            "Email:",
            &mut app.recipient,
            Some(
                "What email address should the vault be associated with? (This should be a public key you have imported into GPG). You can create a public key using the command \"gpg --full-generate-key\", or import an existing one using \"gpg --import <keyfile>\".",
            ),
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Create Vault").clicked() {
                if app.vault_name_create.is_empty() || app.recipient.is_empty() {
                    app.output = Some(Err(
                        "Please fill in all fields before adding an account.".to_string()
                    ));
                    return;
                }
                match create_new_vault(app) {
                    Ok(_) => {
                        app.output = Some(Ok(format!(
                            "Vault \"{}\" created successfully! NOTE: By default, GPG caches your password for 10 minutes. See \"https://github.com/lwilk0/fmp/blob/main/GPGCACHE.md\".",
                            app.vault_name_create
                        )));

                        app.vault_name_create.clear();
                        app.recipient.clear();

                        app.fetch_vault_names();
                    }

                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to create vault \"{}\". Error: {}",
                            app.vault_name_create, e
                        )))
                    }
                }
            }
        });
    });

    ui.add_space(16.0);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        quit_button(app, ui);
    });
}

/// Displays the content for the main window of the application when a vault is selected.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state., pa
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
pub fn vault_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.add_space(12.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Add a New Account");
        });

        ui.separator();
        ui.add_space(8.0);
        labeled_text_input(ui, "Account Name:", &mut app.account_name_create, None);
        ui.add_space(4.0);
        labeled_text_input(ui, "Account Username:", &mut app.userpass.username, None);
        ui.add_space(4.0);
        securely_retrieve_password(app, ui, "Account Password:", false);
        ui.add_space(4.0);
        if ui.button("Generate Password").clicked() {
            app.random_password = true;
        }
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Add Account").clicked() {
                if app.account_name_create.is_empty() {
                    app.output = Some(Err(
                        "Please fill in all fields before adding an account.".to_string()
                    ));
                    return;
                }
                match add_account(app) {
                    Ok(_) => {
                        app.output = Some(Ok(format!(
                            "Account \"{}\" added to vault \"{}\".",
                            app.account_name, app.vault_name
                        )));
                        app.clear_account_data();
                        app.account_name_create.clear();
                        app.fetch_account_names();
                    }
                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to add account \"{}\" to vault \"{}\". Error: {}",
                            app.account_name, app.vault_name, e
                        )));
                    }
                }
            }
        });
    });

    ui.add_space(16.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Vault Options");
        });

        ui.separator();

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Change Vault Name").clicked() {
                app.change_vault_name = true;
            }

            if ui
                .button(egui::RichText::new("Delete Vault").color(egui::Color32::LIGHT_RED))
                .clicked()
            {
                match delete_vault(app) {
                    Ok(()) => {
                        app.output = Some(Ok(format!("Vault \"{}\" deleted.", app.vault_name)));
                        app.vault_name.clear();
                        app.fetch_vault_names();
                    }
                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to delete vault \"{}\". Error: {}",
                            app.vault_name, e
                        )));
                    }
                }
            }
        });
    });

    ui.add_space(16.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Backup Options");
        });

        ui.separator();

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Backup Vault").clicked() {
                match create_backup(app) {
                    Ok(_) => {
                        app.output = Some(Ok(format!(
                            "Vault \"{}\" backed up successfully.",
                            app.vault_name
                        )));
                    }
                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to back up vault \"{}\". Error: {}",
                            app.vault_name, e
                        )));
                    }
                }
            }

            if ui.button("Restore Vault").clicked() {
                match install_backup(app) {
                    Ok(_) => {
                        app.output = Some(Ok(format!(
                            "Vault \"{}\" restored successfully.",
                            app.vault_name
                        )));
                    }
                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to restore vault \"{}\". Error: {}",
                            app.vault_name, e
                        )));
                    }
                }
            }
        });
    });

    ui.add_space(16.0);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.vault_name.clear();
                app.show_password_retrieve = false;
            }
            quit_button(app, ui);
        });
    });
}

/// Displays the content for the main window of the application when an account is selected.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state.
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
pub fn account_selected(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.add_space(12.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Account Details");
        });

        ui.separator();
        ui.add_space(8.0);

        ui.label(egui::RichText::new(format!(
            "Account: {}",
            app.account_name
        )));

        ui.add_space(8.0);

        ui.label(egui::RichText::new(format!(
            "Username: {}",
            app.userpass.username
        )));

        let password = String::from_utf8_lossy(app.userpass.password.expose_secret());

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if app.show_password_account {
                ui.label(egui::RichText::new(format!("Password: {password}")));
            } else {
                ui.label(egui::RichText::new("Password: ********"));
            }

            if ui.button("Copy").clicked() {
                app.toasts
                    .success("Password copied to clipboard.")
                    .duration(Some(Duration::from_secs(2)));

                app.toasts
                    .warning("Clipboard may be read by other apps.")
                    .duration(Some(Duration::from_secs(3)));

                ui.ctx().copy_text(password.to_string());
            }

            app.show_password_account = show_password_button(
                app.show_password_account,
                ui,
                if app.show_password_account {
                    "Hide"
                } else {
                    "Show"
                },
            );
        });

        if app.show_password_account {
            password_strength_meter(ui, &password); // TODO: cache
        }
    });

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        if ui.button("Change Information").clicked() {
            app.account_name_create = app.account_name.clone();
            app.change_account_info = true;
        }

        if ui
            .button(egui::RichText::new("Delete Account").color(egui::Color32::LIGHT_RED))
            .clicked()
        {
            match delete_account_from_vault(app) {
                Ok(_) => {
                    app.output = Some(Ok(format!(
                        "Account \"{}\" deleted from vault \"{}\".",
                        app.account_name, app.vault_name
                    )));

                    app.account_name.clear();
                    app.clear_account_data();
                    app.fetch_account_names();
                }

                Err(e) => {
                    app.output = Some(Err(format!(
                        "Failed to delete account \"{}\" from vault {}. Error: {}",
                        app.account_name, app.vault_name, e
                    )))
                }
            }
        }
    });

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.account_name.clear();
                app.clear_account_data();
                app.show_password_account = false;
            }

            quit_button(app, ui);
        });
    });
}

/// Displays the content for altering account information.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state.
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
pub fn alter_account_information(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.add_space(12.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Edit Account Information");
        });

        ui.separator();
        ui.add_space(8.0);
        labeled_text_input(ui, "Account Name:", &mut app.account_name_create, None);
        ui.add_space(4.0);
        labeled_text_input(ui, "Username:", &mut app.userpass.username, None);
        ui.add_space(4.0);
        securely_retrieve_password(app, ui, "New Password:", false);
        ui.add_space(4.0);
        if ui.button("Generate Password").clicked() {
            app.random_password = true;
        }
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui.button("Change Information").clicked() {
                if app.account_name_create.is_empty() {
                    app.output = Some(Err(
                        "Please fill in all fields before changing an accounts information."
                            .to_string(),
                    ));
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
                                    app.output = Some(Err(format!(
                                        "Failed to change account name. Error: {e}"
                                    )));
                                    return;
                                }
                            }
                        }

                        app.output = Some(Ok(format!(
                            "Account \"{}\" updated successfully in vault \"{}\".",
                            app.account_name, app.vault_name
                        )));

                        app.change_account_info = false;
                        app.fetch_account_names();
                    }
                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to change account name \"{}\". Error: {}",
                            app.account_name, e
                        )));
                    }
                }
            }
            if ui.button("Cancel").clicked() {
                app.change_account_info = false;
                app.show_password_retrieve = false;
            }
        });
    });

    ui.add_space(16.0);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.change_account_info = false;
                app.show_password_retrieve = false;
            }

            quit_button(app, ui);
        });
    });
}

/// Displays the content for altering the vault name.
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state.
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
pub fn alter_vault_name(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.add_space(12.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Change Vault Name");
        });

        ui.separator();
        ui.add_space(8.0);

        labeled_text_input(ui, "New Vault Name:", &mut app.vault_name_create, None);

        ui.horizontal(|ui| {
            if ui.button("Rename Vault").clicked() {
                if app.vault_name_create.is_empty() {
                    app.output = Some(Err(
                        "Please fill in all fields before changing vault name.".to_string()
                    ));
                    return;
                }

                match rename_vault(app) {
                    Ok(_) => {
                        app.output = Some(Ok(format!(
                            "Vault renamed to \"{}\".",
                            app.vault_name_create
                        )));
                        app.vault_name = app.vault_name_create.clone();
                        app.vault_name_create.clear();
                        app.fetch_vault_names();
                        app.change_vault_name = false;
                    }
                    Err(e) => {
                        app.output = Some(Err(format!(
                            "Failed to rename vault \"{}\". Error: {}",
                            app.vault_name_create, e
                        )));
                    }
                }
            }
            if ui.button("Cancel").clicked() {
                app.change_vault_name = false;
            }
        });
    });

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.change_vault_name = false;
            }

            quit_button(app, ui);
        });
    });
}

pub fn random_password(app: &mut FmpApp, ui: &mut egui::Ui) {
    ui.add_space(12.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Password Generator");
        });

        ui.separator();
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("Pool");
            ui.separator();

            egui::Grid::new("pool-options")
                .num_columns(3)
                .spacing([16.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.toggle_value(&mut app.selections[0], "Lower case characters");
                    ui.toggle_value(&mut app.selections[2], "Numbers ");
                    labeled_text_input(
                        ui,
                        "Consider Characters:",
                        &mut app.consider_characters,
                        None,
                    );
                    ui.end_row();

                    ui.toggle_value(&mut app.selections[1], "Upper case characters");
                    ui.toggle_value(&mut app.selections[3], "Symbols   ");
                    labeled_text_input(ui, "Ignore Characters:", &mut app.ignore_characters, None);
                    ui.end_row();

                    ui.toggle_value(&mut app.selections[5], "Accented characters   ");
                    ui.toggle_value(&mut app.selections[4], "Spaces      ");
                    ui.horizontal(|ui| {
                        ui.label("Length:");
                        ui.add(egui::Slider::new(&mut app.password_length, 8..=128));
                    });
                    ui.end_row();
                })
        });

        ui.add_space(8.0);

        if ui.button("Generate Password").clicked() {
            generate_password(app);
        }

        ui.add_space(6.0);
        ui.separator();

        securely_retrieve_password(app, ui, "Generated Password:", true);

        ui.horizontal(|ui| {
            if ui.button("Use").clicked() {
                app.userpass.password = mem::replace(
                    &mut app.generated_password,
                    secrecy::SecretBox::new(Box::new(Vec::<u8>::new())),
                );

                app.generated_password.zeroize();
                app.random_password = false;
            }
            if ui.button("Cancel").clicked() {
                app.generated_password.zeroize();
                app.random_password = false;
                app.show_password_retrieve = false;
            }
        });
    });

    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                app.generated_password.zeroize();
                app.random_password = false;
                app.show_password_retrieve = false;
            }

            quit_button(app, ui);
        });
    });
}

/// Creates a quit button
///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state.
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
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
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
/// * "label" - The label for the text input field.
/// * "value" - A mutable reference to the value of the text input field.
/// * "hover" - An optional string to display as a hover text for the text input field.
fn labeled_text_input(ui: &mut egui::Ui, label: &str, value: &mut String, hover: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(label);

        let text_edit = ui.text_edit_singleline(value);

        if let Some(hover_text) = hover {
            text_edit.on_hover_text(hover_text);
        }
    });
}

/// Displays a show/hide password button
///
/// # Arguments
/// * "state" - The current state of the password(True - Show, False - Hide)
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
/// * "label" - The label for the text input field.
///
/// # Returns
/// * A "bool" of the new state.
pub fn show_password_button(state: bool, ui: &mut egui::Ui, label: &str) -> bool {
    if ui.button(label).clicked() {
        return !state;
    }
    state
}
