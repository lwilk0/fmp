/// All this code is awful, sorry future me. There must be a better way????
use crate::{
    gui::dialogs::common::show_error_dialog, storage::filesystem::rename_account, vault::Account,
};
use adw::prelude::*;
use gpgme::Context;
use gtk4::{Box as GtkBox, Button, Dialog, Entry, Label, Orientation};
use std::{cell::RefCell, rc::Rc};

/// Shows the rename account dialog
pub fn show_rename_account_dialog(
    vault_name: &str,
    account_name: &str,
    content_area: &GtkBox,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Rename Account"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Rename Account '{account_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let entry_label = Label::new(Some("New account name:"));
    entry_label.set_halign(gtk4::Align::Start);
    content_box.append(&entry_label);

    let name_entry = Entry::new();
    name_entry.set_text(account_name);
    name_entry.set_placeholder_text(Some("Enter new account name"));
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
    let account_name_clone = account_name.to_string();
    let name_entry_clone = name_entry.clone();
    rename_button.connect_clicked(move |_| {
        let new_name = name_entry_clone.text().to_string();
        if !new_name.is_empty() && new_name != account_name_clone {
            match rename_account(&vault_name_clone, &account_name_clone, &new_name) {
                Ok(()) => {
                    dialog_clone.close();
                    crate::gui::views::account_view::AccountView::new(
                        &content_area_clone,
                        &vault_name_clone,
                        &new_name,
                        false,
                    )
                    .create(ctx.clone());
                }
                Err(e) => {
                    log::error!("Failed to rename account: {e}");
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

/// Shows the add field dialog for adding additional fields to an account
pub fn show_add_field_dialog(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &GtkBox,
    vault_name: &str,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Add Additional Field"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 300);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title_label = Label::new(Some("Add a new field to this account"));
    title_label.add_css_class("title-4");
    content_box.append(&title_label);

    let name_label = Label::new(Some("Field Name:"));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("dim-label");
    content_box.append(&name_label);

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("e.g., Security Question, PIN, etc."));
    name_entry.set_hexpand(true);
    content_box.append(&name_entry);

    let value_label = Label::new(Some("Field Value:"));
    value_label.set_halign(gtk4::Align::Start);
    value_label.add_css_class("dim-label");
    value_label.set_margin_top(8);
    content_box.append(&value_label);

    let value_entry = Entry::new();
    value_entry.set_placeholder_text(Some("Enter the field value"));
    value_entry.set_hexpand(true);
    content_box.append(&value_entry);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(20);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");

    let add_button = Button::new();
    add_button.set_label("Add Field");
    add_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let account_rc_clone = account_rc.clone();
    let name_entry_clone = name_entry.clone();
    let value_entry_clone = value_entry.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();

    add_button.connect_clicked(move |_| {
        let field_name = name_entry_clone.text().to_string().trim().to_string();
        let field_value = value_entry_clone.text().to_string().trim().to_string();

        if !field_name.is_empty() && !field_value.is_empty() {
            let mut account = account_rc_clone.borrow_mut();
            if account.additional_fields.contains_key(&field_name) {
                drop(account);
                show_error_dialog(
                    "Field Already Exists",
                    &format!(
                        "A field named \"{field_name}\" already exists. Please choose a different name."

                    ),
                );
                return;
            }

            account.additional_fields.insert(field_name, field_value);
            account.update_modified_time();
            let account_name = account.name.clone();

            match crate::vault::update_account(&vault_name_clone, &account) {
                Ok(()) => {
                    drop(account);
                    dialog_clone.close();

                    crate::gui::views::account_view::AccountView::new(
                        &content_area_clone,
                        &vault_name_clone,
                        &account_name,
                        true,
                    ).create(ctx.clone());
                }
                Err(e) => {
                    drop(account);
                    log::error!("Failed to save account: {e}");
                    show_error_dialog(
                        "Save Error",
                        "Failed to save the new field. Please try again.",
                    );
                }
            }
        } else {
            show_error_dialog("Invalid Input", "Both field name and value are required.");
        }
    });

    let add_button_clone = add_button.clone();
    value_entry.connect_activate(move |_| {
        add_button_clone.emit_clicked();
    });

    button_box.append(&cancel_button);
    button_box.append(&add_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

#[allow(clippy::too_many_lines)]
/// Shows the edit field dialog for editing an additional field's name and value
pub fn show_edit_field_dialog(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &GtkBox,
    vault_name: &str,
    field_name: &str,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Edit Field"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 300);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title_label = Label::new(Some("Edit Field"));
    title_label.add_css_class("title-4");
    content_box.append(&title_label);

    let current_value = {
        let account = account_rc.borrow();
        account
            .additional_fields
            .get(field_name)
            .cloned()
            .unwrap_or_default()
    };

    let name_section = GtkBox::new(Orientation::Vertical, 8);
    let name_label = Label::new(Some("Field Name:"));
    name_label.add_css_class("dim-label");
    name_label.set_halign(gtk4::Align::Start);

    let name_entry = Entry::new();
    name_entry.set_text(field_name);
    name_entry.set_hexpand(true);

    name_section.append(&name_label);
    name_section.append(&name_entry);
    content_box.append(&name_section);

    let value_section = GtkBox::new(Orientation::Vertical, 8);
    let value_label = Label::new(Some("Field Value:"));
    value_label.add_css_class("dim-label");
    value_label.set_halign(gtk4::Align::Start);

    let value_entry = Entry::new();
    value_entry.set_text(&current_value);
    value_entry.set_hexpand(true);

    value_section.append(&value_label);
    value_section.append(&value_entry);
    content_box.append(&value_section);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(20);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");

    let save_button = Button::new();
    save_button.set_label("Save");
    save_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let account_rc_clone = account_rc.clone();
    let name_entry_clone = name_entry.clone();
    let value_entry_clone = value_entry.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    let old_field_name = field_name.to_string();

    save_button.connect_clicked(move |_| {
        let new_field_name = name_entry_clone.text().to_string().trim().to_string();
        let new_field_value = value_entry_clone.text().to_string().trim().to_string();

        if !new_field_name.is_empty() && !new_field_value.is_empty() {
            let mut account = account_rc_clone.borrow_mut();

            if new_field_name != old_field_name
                && account.additional_fields.contains_key(&new_field_name)
            {
                drop(account);
                show_error_dialog(
                    "Field Already Exists",
                    &format!(
                        "A field named '{new_field_name}' already exists. Please choose a different name."
                    ),
                );
                return;
            }

             if new_field_name != old_field_name {
                account.additional_fields.remove(&old_field_name);
            }

            account
                .additional_fields
                .insert(new_field_name, new_field_value);

            account.update_modified_time();
            let account_name = account.name.clone();

            match crate::vault::update_account(&vault_name_clone, &account) {
                Ok(()) => {
                    drop(account);
                    dialog_clone.close();

                    crate::gui::views::account_view::AccountView::new(
                        &content_area_clone,
                        &vault_name_clone,
                        &account_name,
                        true, // Keep in edit mode
                    ).create(ctx.clone());
                }
                Err(e) => {
                    drop(account);
                    log::error!("Failed to save account: {e}");
                    show_error_dialog(
                        "Save Error",
                        "Failed to save the field changes. Please try again.",
                    );
                }
            }
        } else {
            show_error_dialog("Invalid Input", "Both field name and value are required.");
        }
    });

    let save_button_clone = save_button.clone();
    name_entry.connect_activate(move |_| {
        save_button_clone.emit_clicked();
    });

    let save_button_clone2 = save_button.clone();
    value_entry.connect_activate(move |_| {
        save_button_clone2.emit_clicked();
    });

    button_box.append(&cancel_button);
    button_box.append(&save_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the delete field confirmation dialog
pub fn show_delete_field_dialog(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &GtkBox,
    vault_name: &str,
    field_name: &str,
    ctx: Rc<RefCell<Context>>,
) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Delete Field"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title_box = GtkBox::new(Orientation::Horizontal, 12);
    title_box.set_halign(gtk4::Align::Center);

    let title_label = Label::new(Some("Delete Field"));
    title_label.add_css_class("title-3");

    title_box.append(&title_label);
    content_box.append(&title_box);

    // Confirmation message
    let message = format!(
        "Are you sure you want to delete the field '{field_name}'?\n\nThis action cannot be undone."
    );
    let message_label = Label::new(Some(&message));
    message_label.set_wrap(true);
    message_label.set_halign(gtk4::Align::Center);
    message_label.set_justify(gtk4::Justification::Center);
    content_box.append(&message_label);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(20);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");

    let delete_button = Button::new();
    delete_button.set_label("Delete");
    delete_button.add_css_class("destructive-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let account_rc_clone = account_rc.clone();
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    let field_name_owned = field_name.to_string();
    delete_button.connect_clicked(move |_| {
        let mut account = account_rc_clone.borrow_mut();
        account.additional_fields.remove(&field_name_owned);
        account.update_modified_time();
        let account_name = account.name.clone();

        match crate::vault::update_account(&vault_name_clone, &account) {
            Ok(()) => {
                drop(account);
                dialog_clone.close();

                // Refresh the account view
                crate::gui::views::account_view::AccountView::new(
                    &content_area_clone,
                    &vault_name_clone,
                    &account_name,
                    true, // Keep in edit mode
                )
                .create(ctx.clone());
            }
            Err(e) => {
                drop(account);
                log::error!("Failed to save account: {e}");
                show_error_dialog(
                    "Save Error",
                    "Failed to delete the field. Please try again.",
                );
            }
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&delete_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}
