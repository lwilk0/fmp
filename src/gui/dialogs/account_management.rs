use crate::gui::dialogs::common::{
    dialog_skeleton, root_window, show_confirmation_dialog, show_error_dialog,
};
use crate::storage::filesystem::rename_account;
use crate::vault::Account;
use adw::prelude::*;
use gpgme::Context;
use gtk4::Box as GtkBox;
use std::{cell::RefCell, rc::Rc};

/// Shows the rename account dialog
pub fn show_rename_account_dialog(
    vault_name: &str,
    account_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();
    let account_name = account_name.to_string();

    let parts = dialog_skeleton(&parent, "Rename Account", "Rename", "suggested-action");
    parts.window.set_default_size(360, -1);

    let name_row = adw::EntryRow::new();
    name_row.set_title("New account name");
    name_row.set_text(&account_name);

    let group = adw::PreferencesGroup::new();
    group.add(&name_row);
    parts.content.append(&group);

    let confirm_for_enter = parts.confirm.clone();
    name_row.connect_entry_activated(move |_| {
        confirm_for_enter.activate();
    });

    let dialog = parts.window.clone();
    let content_area = content_area.clone();
    let name_row = name_row.clone();

    parts.confirm.connect_clicked(move |_| {
        let new_name = name_row.text().trim().to_string();
        if new_name.is_empty() || new_name == account_name {
            return;
        }
        match rename_account(&vault_name, &account_name, &new_name) {
            Ok(()) => {
                dialog.close();
                crate::gui::views::account_view::AccountView::new(
                    &content_area,
                    &vault_name,
                    &new_name,
                    false,
                )
                .create(ctx.clone());
            }
            Err(e) => {
                log::error!("Failed to rename account: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Rename Failed",
                    &format!("Could not rename account: {e}"),
                );
            }
        }
    });

    parts.window.present();
}

/// Shows the add field dialog for adding additional fields to an account
pub fn show_add_field_dialog(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &GtkBox,
    vault_name: &str,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();

    let parts = dialog_skeleton(&parent, "Add Field", "Add", "suggested-action");
    parts.window.set_default_size(360, -1);

    let name_row = adw::EntryRow::new();
    name_row.set_title("Field name");

    let value_row = adw::EntryRow::new();
    value_row.set_title("Field value");

    let group = adw::PreferencesGroup::new();
    group.add(&name_row);
    group.add(&value_row);
    parts.content.append(&group);

    // Enter in name row -> focus value row; Enter in value row -> confirm
    let value_for_name = value_row.clone();
    name_row.connect_entry_activated(move |_| {
        value_for_name.grab_focus();
    });
    let confirm_for_value = parts.confirm.clone();
    value_row.connect_entry_activated(move |_| {
        confirm_for_value.activate();
    });

    let dialog = parts.window.clone();
    let account_rc = account_rc.clone();
    let content_area = content_area.clone();
    let name_row = name_row.clone();
    let value_row = value_row.clone();

    parts.confirm.connect_clicked(move |_| {
        let field_name = name_row.text().trim().to_string();
        let field_value = value_row.text().trim().to_string();

        if field_name.is_empty() || field_value.is_empty() {
            show_error_dialog(
                Some(&parent),
                "Invalid Input",
                "Both field name and value are required.",
            );
            return;
        }

        let mut account = account_rc.borrow_mut();
        if account.additional_fields.contains_key(&field_name) {
            drop(account);
            show_error_dialog(
                Some(&parent),
                "Field Already Exists",
                &format!(
                    "A field named '{field_name}' already exists. Please choose a different name."
                ),
            );
            return;
        }

        account.additional_fields.insert(field_name, field_value);
        account.update_modified_time();
        let account_name = account.name.clone();

        match crate::vault::update_account(&vault_name, &account) {
            Ok(()) => {
                drop(account);
                dialog.close();
                crate::gui::views::account_view::AccountView::new(
                    &content_area,
                    &vault_name,
                    &account_name,
                    true,
                )
                .create(ctx.clone());
            }
            Err(e) => {
                drop(account);
                log::error!("Failed to save account: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Save Error",
                    "Failed to save the new field. Please try again.",
                );
            }
        }
    });

    parts.window.present();
}

/// Shows the edit field dialog for editing an additional field's name and value
#[allow(clippy::too_many_lines)]
pub fn show_edit_field_dialog(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &GtkBox,
    vault_name: &str,
    field_name: &str,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");
    let vault_name = vault_name.to_string();
    let field_name = field_name.to_string();

    let current_value = {
        let account = account_rc.borrow();
        account
            .additional_fields
            .get(&field_name)
            .cloned()
            .unwrap_or_default()
    };

    let parts = dialog_skeleton(&parent, "Edit Field", "Save", "suggested-action");
    parts.window.set_default_size(360, -1);

    let name_row = adw::EntryRow::new();
    name_row.set_title("Field name");
    name_row.set_text(&field_name);

    let value_row = adw::EntryRow::new();
    value_row.set_title("Field value");
    value_row.set_text(&current_value);

    let group = adw::PreferencesGroup::new();
    group.add(&name_row);
    group.add(&value_row);
    parts.content.append(&group);

    let value_for_name = value_row.clone();
    name_row.connect_entry_activated(move |_| {
        value_for_name.grab_focus();
    });
    let confirm_for_value = parts.confirm.clone();
    value_row.connect_entry_activated(move |_| {
        confirm_for_value.activate();
    });

    let dialog = parts.window.clone();
    let account_rc = account_rc.clone();
    let content_area = content_area.clone();
    let old_field_name = field_name.clone();
    let name_row = name_row.clone();
    let value_row = value_row.clone();

    parts.confirm.connect_clicked(move |_| {
        let new_field_name = name_row.text().trim().to_string();
        let new_field_value = value_row.text().trim().to_string();

        if new_field_name.is_empty() || new_field_value.is_empty() {
            show_error_dialog(
                Some(&parent),
                "Invalid Input",
                "Both field name and value are required.",
            );
            return;
        }

        let mut account = account_rc.borrow_mut();
        if new_field_name != old_field_name
            && account.additional_fields.contains_key(&new_field_name)
        {
            drop(account);
            show_error_dialog(
                Some(&parent),
                "Field Already Exists",
                &format!("A field named '{new_field_name}' already exists. Please choose a different name."),
            );
            return;
        }

        if new_field_name != old_field_name {
            account.additional_fields.remove(&old_field_name);
        }
        account.additional_fields.insert(new_field_name, new_field_value);
        account.update_modified_time();
        let account_name = account.name.clone();

        match crate::vault::update_account(&vault_name, &account) {
            Ok(()) => {
                drop(account);
                dialog.close();
                crate::gui::views::account_view::AccountView::new(
                    &content_area,
                    &vault_name,
                    &account_name,
                    true, // keep in edit mode
                )
                .create(ctx.clone());
            }
            Err(e) => {
                drop(account);
                log::error!("Failed to save account: {e}");
                show_error_dialog(
                    Some(&parent),
                    "Save Error",
                    "Failed to save the field changes. Please try again.",
                );
            }
        }
    });

    parts.window.present();
}

/// Shows the delete field confirmation dialog
pub fn show_delete_field_dialog(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &GtkBox,
    vault_name: &str,
    field_name: &str,
    ctx: Rc<RefCell<Context>>,
) {
    let parent = root_window(content_area).expect("content_area must live inside a window");

    // Convert borrowed params into owned handles so the 'static closure can keep them
    let account_rc = account_rc.clone();
    let content_area = content_area.clone();
    let vault_name = vault_name.to_string();
    let field_name = field_name.to_string();

    show_confirmation_dialog(
        "Delete Field",
        &format!(
            "Are you sure you want to delete the field '{field_name}'? This action cannot be undone."
        ),
        "Delete",
        Some(&parent.clone()),
        move || {
            let mut account = account_rc.borrow_mut();
            account.additional_fields.remove(&field_name);
            account.update_modified_time();
            let account_name = account.name.clone();

            match crate::vault::update_account(&vault_name, &account) {
                Ok(()) => {
                    drop(account);
                    crate::gui::views::account_view::AccountView::new(
                        &content_area,
                        &vault_name,
                        &account_name,
                        true, // keep in edit mode
                    )
                    .create(ctx.clone());
                }
                Err(e) => {
                    drop(account);
                    log::error!("Failed to save account: {e}");
                    show_error_dialog(
                        Some(&parent),
                        "Save Error",
                        "Failed to delete the field. Please try again.",
                    );
                }
            }
        },
    );
}
