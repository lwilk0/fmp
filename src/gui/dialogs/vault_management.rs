use crate::gui::dialogs::common::show_error_dialog;
use crate::storage::filesystem::{
    create_backup, delete_backup, delete_vault, install_backup, rename_vault,
};
use gpgme::Context;
use std::{cell::RefCell, rc::Rc};

use adw::prelude::*;
use gtk4::{Box as GtkBox, Button, Dialog, Entry, Label, Orientation};

/// Shows the backup vault dialog
pub fn show_backup_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Create Vault Backup"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Create Backup for '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let description = Label::new(Some(
        "This will create a backup of your vault that can be restored later.",
    ));
    description.add_css_class("dim-label");
    description.set_wrap(true);
    description.set_halign(gtk4::Align::Center);
    content_box.append(&description);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let backup_button = Button::new();
    backup_button.set_label("Create Backup");
    backup_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    backup_button.connect_clicked(move |_| match create_backup(&vault_name_clone) {
        Ok(()) => {
            dialog_clone.close();
            crate::gui::views::vault_view::VaultView::new(&content_area_clone, &vault_name_clone)
                .create(ctx.clone());
        }
        Err(e) => {
            log::error!("Failed to create backup: {e}");
            show_error_dialog("Backup Failed", &format!("Could not create backup: {e}"));
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&backup_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the restore vault dialog
pub fn show_restore_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Restore Vault Backup"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 250);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Restore Backup for '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let warning = Label::new(Some(
        "This will replace all current vault data with the backup.\nThis action cannot be undone.",
    ));
    warning.add_css_class("error");
    warning.set_wrap(true);
    warning.set_halign(gtk4::Align::Center);
    content_box.append(&warning);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let restore_button = Button::new();
    restore_button.set_label("Restore Backup");
    restore_button.add_css_class("destructive-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    restore_button.connect_clicked(move |_| {
        match install_backup(&vault_name_clone) {
            Ok(()) => {
                dialog_clone.close();
                // Refresh the vault view
                crate::gui::views::vault_view::VaultView::new(
                    &content_area_clone,
                    &vault_name_clone,
                )
                .create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to restore backup: {e}");
                show_error_dialog("Restore Failed", &format!("Could not restore backup: {e}"));
            }
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&restore_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the delete backup dialog
pub fn show_delete_backup_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Delete Vault Backup"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Delete Backup for '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let warning = Label::new(Some(
        "Are you sure you want to delete the backup?\nThis action cannot be undone.",
    ));
    warning.add_css_class("error");
    warning.set_wrap(true);
    warning.set_halign(gtk4::Align::Center);
    content_box.append(&warning);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let delete_button = Button::new();
    delete_button.set_label("Delete Backup");
    delete_button.add_css_class("destructive-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    delete_button.connect_clicked(move |_| match delete_backup(&vault_name_clone) {
        Ok(()) => {
            dialog_clone.close();
            crate::gui::views::vault_view::VaultView::new(&content_area_clone, &vault_name_clone)
                .create(ctx.clone());
        }
        Err(e) => {
            log::error!("Failed to delete backup: {e}");
            show_error_dialog("Delete Failed", &format!("Could not delete backup: {e}"));
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&delete_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the rename vault dialog
pub fn show_rename_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Rename Vault"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Rename Vault '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let entry_label = Label::new(Some("New vault name:"));
    entry_label.set_halign(gtk4::Align::Start);
    content_box.append(&entry_label);

    let name_entry = Entry::new();
    name_entry.set_text(vault_name);
    name_entry.set_placeholder_text(Some("Enter new vault name"));
    content_box.append(&name_entry);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let rename_button = Button::new();
    rename_button.set_label("Rename");
    rename_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    let name_entry_clone = name_entry.clone();
    rename_button.connect_clicked(move |_| {
        let new_name = name_entry_clone.text().to_string();
        if !new_name.is_empty() && new_name != vault_name_clone {
            match rename_vault(&vault_name_clone, &new_name) {
                Ok(()) => {
                    dialog_clone.close();
                    crate::gui::sidebar::refresh_sidebar_from_content_area(
                        &content_area_clone,
                        ctx.clone(),
                    );
                    crate::gui::views::vault_view::VaultView::new(&content_area_clone, &new_name)
                        .create(ctx.clone());
                }
                Err(e) => {
                    log::error!("Failed to rename vault: {e}");
                    show_error_dialog("Rename Failed", &format!("Could not rename vault: {e}"));
                }
            }
        }
    });

    let rename_button_clone = rename_button.clone();
    name_entry.connect_activate(move |_| {
        rename_button_clone.emit_clicked();
    });

    button_box.append(&cancel_button);
    button_box.append(&rename_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the delete vault dialog
pub fn show_delete_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Delete Vault"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 250);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Delete Vault '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let warning = Label::new(Some(
        "This will permanently delete the vault and all its data.\nThis action cannot be undone.",
    ));
    warning.add_css_class("error");
    warning.set_wrap(true);
    warning.set_halign(gtk4::Align::Center);
    content_box.append(&warning);

    let confirmation_label = Label::new(Some(&format!("Type '{vault_name}' to confirm deletion:")));
    confirmation_label.set_halign(gtk4::Align::Start);
    content_box.append(&confirmation_label);

    let confirmation_entry = Entry::new();
    confirmation_entry.set_placeholder_text(Some("Type vault name here"));
    content_box.append(&confirmation_entry);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let delete_button = Button::new();
    delete_button.set_label("Delete Vault");
    delete_button.add_css_class("destructive-action");
    delete_button.set_sensitive(false);

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let delete_button_clone = delete_button.clone();
    let vault_name_clone = vault_name.to_string();
    confirmation_entry.connect_changed(move |entry| {
        let text = entry.text();
        delete_button_clone.set_sensitive(text == vault_name_clone);
    });

    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    let confirmation_entry_clone = confirmation_entry.clone();
    delete_button.connect_clicked(move |_| {
        let confirmation_text = confirmation_entry_clone.text();
        if confirmation_text == vault_name_clone {
            match delete_vault(&vault_name_clone) {
                Ok(()) => {
                    dialog_clone.close();
                    crate::gui::sidebar::refresh_sidebar_from_content_area(
                        &content_area_clone,
                        ctx.clone(),
                    );
                    crate::gui::views::home_view::HomeView::new(&content_area_clone)
                        .create(ctx.clone())
                }
                Err(e) => {
                    log::error!("Failed to delete vault: {e}");
                    show_error_dialog("Delete Failed", &format!("Could not delete vault: {e}"));
                }
            }
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&delete_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}
