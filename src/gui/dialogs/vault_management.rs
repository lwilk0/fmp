use crate::gui::content;
use crate::gui::dialogs::common::{dialog_skeleton, root_window, show_error_dialog};
use crate::storage::filesystem::{
    create_backup, delete_backup, delete_vault, install_backup, rename_vault,
};
use adw::prelude::*;
use gpgme::Context;
use gtk4::Box as GtkBox;
use std::{cell::RefCell, rc::Rc};

/// Shows the backup vault dialog
pub fn show_backup_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();

    let dialog = adw::MessageDialog::new(
        Some(&parent),
        Some(&format!("Create Backup for '{vault_name}'?")),
        Some("This will create a backup of your vault that can be restored later."),
    );

    dialog.add_response("cancel", "Cancel");
    dialog.set_close_response("cancel");
    dialog.set_default_response(Some("cancel"));

    dialog.add_response("confirm", "Create Backup");
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Suggested);

    let content_area = content_area.clone();

    dialog.connect_response(Some("confirm"), move |_, _| {
        match create_backup(&vault_name) {
            Ok(()) => {
                crate::gui::views::vault_view::VaultView::new(&content_area, &vault_name)
                    .create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to create backup: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Backup Failed",
                    &format!("Could not create backup: {e}"),
                );
            }
        }
    });

    dialog.present();
}

/// Shows the restore vault dialog
pub fn show_restore_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();

    let dialog = adw::MessageDialog::new(
        Some(&parent),
        Some(&format!("Restore Backup for '{vault_name}'?")),
        Some(
            "This will replace all current vault data with the backup. This action cannot be undone.",
        ),
    );

    dialog.add_response("cancel", "Cancel");
    dialog.set_close_response("cancel");
    dialog.set_default_response(Some("cancel"));

    dialog.add_response("confirm", "Restore Backup");
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);

    let content_area = content_area.clone();

    dialog.connect_response(Some("confirm"), move |_, _| {
        match install_backup(&vault_name) {
            Ok(()) => {
                crate::gui::views::vault_view::VaultView::new(&content_area, &vault_name)
                    .create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to restore backup: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Restore Failed",
                    &format!("Could not restore backup: {e}"),
                );
            }
        }
    });

    dialog.present();
}

/// Shows the delete backup dialog
pub fn show_delete_backup_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();

    let dialog = adw::MessageDialog::new(
        Some(&parent),
        Some(&format!("Delete Backup for '{vault_name}'?")),
        Some("Are you sure you want to delete the backup? This action cannot be undone."),
    );

    dialog.add_response("cancel", "Cancel");
    dialog.set_close_response("cancel");
    dialog.set_default_response(Some("cancel"));

    dialog.add_response("confirm", "Delete Backup");
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);

    let content_area = content_area.clone();

    dialog.connect_response(Some("confirm"), move |_, _| {
        match delete_backup(&vault_name) {
            Ok(()) => {
                crate::gui::views::vault_view::VaultView::new(&content_area, &vault_name)
                    .create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to delete backup: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Delete Failed",
                    &format!("Could not delete backup: {e}"),
                );
            }
        }
    });

    dialog.present();
}

/// Shows the rename vault dialog
pub fn show_rename_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();

    let parts = dialog_skeleton(&parent, "Rename Vault", "Rename", "suggested-action");
    parts.window.set_default_size(360, -1); // width only; height is natural

    let name_row = adw::EntryRow::new();
    name_row.set_title("New vault name");
    name_row.set_text(&vault_name);

    let group = adw::PreferencesGroup::new();
    group.add(&name_row);
    parts.content.append(&group);

    // Enter in the row triggers the confirm button
    let confirm_for_enter = parts.confirm.clone();
    name_row.connect_entry_activated(move |_| {
        confirm_for_enter.activate();
    });

    let dialog = parts.window.clone();
    let content_area = content_area.clone();
    let name_row = name_row.clone();

    parts.confirm.connect_clicked(move |_| {
        let new_name = name_row.text().trim().to_string();
        if new_name.is_empty() || new_name == vault_name {
            return;
        }
        match rename_vault(&vault_name, &new_name) {
            Ok(()) => {
                dialog.close();
                crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area, ctx.clone());
                crate::gui::views::vault_view::VaultView::new(&content_area, &new_name)
                    .create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to rename vault: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Rename Failed",
                    &format!("Could not rename vault: {e}"),
                );
            }
        }
    });

    parts.window.present();
}

/// Shows the delete vault dialog
pub fn show_delete_vault_dialog(
    vault_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();

    let parts = dialog_skeleton(
        &parent,
        "Delete Vault",
        "Delete Vault",
        "destructive-action",
    );
    parts.window.set_default_size(360, -1);
    parts.confirm.set_sensitive(false);

    let confirm_row = adw::EntryRow::new();
    confirm_row.set_title(&format!("Type '{vault_name}' to confirm"));

    let group = adw::PreferencesGroup::new();
    group.set_description(Some(
        "This will permanently delete the vault and all its data. This action cannot be undone.",
    ));
    group.add(&confirm_row);
    parts.content.append(&group);

    // Confirm button only enables when the typed name matches
    let confirm_btn = parts.confirm.clone();
    let vault_for_gate = vault_name.clone();
    confirm_row.connect_changed(move |row| {
        confirm_btn.set_sensitive(row.text().as_str() == vault_for_gate);
    });

    let dialog = parts.window.clone();
    let content_area = content_area.clone();

    parts.confirm.connect_clicked(move |_| {
        if confirm_row.text().as_str() != vault_name {
            return;
        }
        match delete_vault(&vault_name) {
            Ok(()) => {
                dialog.close();
                crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area, ctx.clone());
                crate::gui::views::home_view::HomeView::new(&content_area).create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to delete vault: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Delete Failed",
                    &format!("Could not delete vault: {e}"),
                );
            }
        }
    });

    parts.window.present();
}
