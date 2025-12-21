use crate::{
    gui::{
        content::{CreateBox, clear_content, create_editable_field_row, create_field_row},
        dialogs::{
            show_add_field_dialog, show_confirmation_dialog, show_delete_field_dialog,
            show_edit_field_dialog, show_password_generator_dialog, show_rename_account_dialog,
        },
        views::home_view::HomeView,
    },
    vault::{Account, delete_account, get_full_account_details, update_account},
};
use adw::{ButtonContent, prelude::*};
use gtk4::{
    Align, Box, Button, Entry, Label, Orientation, PolicyType, ScrolledWindow, Separator,
    pango::EllipsizeMode,
};
use std::{cell::RefCell, rc::Rc};
pub struct AccountView<'a> {
    content_area: &'a Box,
    vault_name: String,
    account_name: String,
    edit_mode: bool,
}

impl<'a> AccountView<'a> {
    pub fn new(
        content_area: &'a Box,
        vault_name: &str,
        account_name: &str,
        edit_mode: bool,
    ) -> Self {
        Self {
            content_area,
            vault_name: vault_name.to_string(),
            account_name: account_name.to_string(),
            edit_mode,
        }
    }

    pub fn create(&self) {
        clear_content(self.content_area);

        let account_data = match get_full_account_details(&self.vault_name, &self.account_name) {
            Ok(account) => account,
            Err(_) => Account {
                name: self.account_name.clone(),
                ..Default::default()
            },
        };

        let account_rc = Rc::new(RefCell::new(account_data));

        let main_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 24))
            .margins(24, 24, 24, 24)
            .build();

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(true);

        main_box.append(&self.header_section(&account_rc));
        main_box.append(&self.details_section(&account_rc));
        main_box.append(&self.password_section(&account_rc));
        main_box.append(&self.additional_fields_section(&account_rc));
        main_box.append(&self.notes_section(&account_rc));

        if self.edit_mode {
            main_box.append(&self.actions_section(&account_rc));
        }

        scrolled_window.set_child(Some(&main_box));
        self.content_area.append(&scrolled_window);
    }

    fn header_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let header_box = Box::new(Orientation::Horizontal, 16);
        header_box.set_halign(gtk4::Align::Fill);

        let info_box = Box::new(Orientation::Vertical, 4);
        info_box.set_hexpand(true);

        let account = account_rc.borrow();

        // Primary title: prefer stored account name, fallback to provided name
        let display_name = if account.name.trim().is_empty() {
            self.account_name.to_string()
        } else {
            account.name.clone()
        };

        let title = Label::new(Some(&display_name));
        title.add_css_class("title-1");
        title.set_halign(gtk4::Align::Start);
        title.set_ellipsize(EllipsizeMode::End);
        title.set_max_width_chars(50);
        title.set_tooltip_text(Some(&display_name));
        title.set_wrap(true);
        title.set_lines(2);

        let acct_type = if account.account_type.trim().is_empty() {
            "Account".to_string()
        } else {
            account.account_type.clone()
        };

        let test = self.vault_name.clone();
        let subtitle_text = format!("{acct_type} • in {test}");

        let subtitle = Label::new(Some(&subtitle_text));
        subtitle.add_css_class("dim-label");
        subtitle.add_css_class("caption");
        subtitle.set_halign(gtk4::Align::Start);
        subtitle.set_ellipsize(EllipsizeMode::End);
        subtitle.set_max_width_chars(55);
        subtitle.set_tooltip_text(Some(&subtitle_text));

        info_box.append(&title);
        info_box.append(&subtitle);

        let actions_box = Box::new(Orientation::Horizontal, 8);

        if !self.edit_mode {
            let edit_button = Button::new();
            edit_button.set_label("Edit");
            edit_button.set_tooltip_text(Some("Edit account"));
            edit_button.add_css_class("suggested-action");

            // Connect edit functionality
            let content_area_clone = self.content_area.clone();
            let vault_name_clone = self.vault_name.to_string();
            let account_name_clone = self.account_name.to_string();
            edit_button.connect_clicked(move |_| {
                AccountView::new(
                    &content_area_clone,
                    &vault_name_clone,
                    &account_name_clone,
                    true,
                )
                .create();
            });

            let rename_button = Button::new();
            rename_button.set_label("Rename");
            rename_button.set_tooltip_text(Some("Rename account"));
            rename_button.add_css_class("suggested-action");

            let delete_button = Button::new();
            delete_button.set_label("Delete");
            delete_button.set_tooltip_text(Some("Delete account"));
            delete_button.add_css_class("destructive-action");

            // Connect delete functionality with confirmation dialog
            let content_area_delete = self.content_area.clone();
            let vault_name_delete = self.vault_name.to_string();
            let account_name_delete = self.account_name.to_string();
            delete_button.connect_clicked(move |_| {
            let message = format!(
                "Are you sure you want to delete the account '{account_name_delete}'?\n\nThis action cannot be undone and will permanently remove all data associated with this account.");

            let content_area_confirm = content_area_delete.clone();
            let vault_name_confirm = vault_name_delete.clone();
            let account_name_confirm = account_name_delete.clone();

            show_confirmation_dialog(
                "Delete Account",
                &message,
                "Delete",
                None::<&gtk4::Window>,
                move || {
                    match delete_account(&vault_name_confirm, &account_name_confirm) {
                        Ok(()) => {
                            HomeView::new(&content_area_confirm).create();
                        }
                        Err(e) => {
                            eprintln!("Failed to delete account '{account_name_confirm}': {e}");
                        }
                    }
                },
            );
        });

            let content_area_rename = self.content_area.clone();
            let vault_name_rename = self.vault_name.to_string();
            let account_name_rename = self.account_name.to_string();
            rename_button.connect_clicked(move |_| {
                show_rename_account_dialog(
                    &vault_name_rename,
                    &account_name_rename,
                    &content_area_rename,
                );
            });

            actions_box.append(&edit_button);
            actions_box.append(&rename_button);
            actions_box.append(&delete_button);
        }

        header_box.append(&info_box);
        header_box.append(&actions_box);

        header_box
    }

    fn details_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 20);
        section.add_css_class("account-section");
        section.set_margin_top(20);

        let header_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 8))
            .margins(16, 0, 24, 24)
            .build();

        let title = Label::new(Some("Account Details"));
        title.add_css_class("title-3");
        title.set_halign(gtk4::Align::Start);

        let subtitle = Label::new(Some("Basic account information and credentials"));
        subtitle.add_css_class("dim-label");
        subtitle.add_css_class("caption");
        subtitle.set_halign(gtk4::Align::Start);

        header_box.append(&title);
        header_box.append(&subtitle);
        section.append(&header_box);

        let details_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 18))
            .margins(0, 20, 24, 24)
            .halign(Align::Center)
            .build();

        let account = account_rc.borrow();

        if self.edit_mode {
            // Editable fields in edit mode
            let website_row =
                create_editable_field_row("Website", &account.website, account_rc, "website");
            details_box.append(&website_row);

            let username_row =
                create_editable_field_row("Username", &account.username, account_rc, "username");
            details_box.append(&username_row);
        } else {
            // Read-only fields in view mode
            let website_row = create_field_row("Website", &account.website, true);
            details_box.append(&website_row);

            let username_row = create_field_row("Username", &account.username, true);
            details_box.append(&username_row);
        }

        let created_row = create_field_row("Created", &account.created_at, false);
        details_box.append(&created_row);

        let modified_row = create_field_row("Last Modified", &account.modified_at, false);
        details_box.append(&modified_row);

        section.append(&details_box);
        section
    }

    fn password_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 20);
        section.add_css_class("account-section");
        section.set_margin_top(20);

        let header_box = CreateBox::new()
            .new_box(Box::new(Orientation::Horizontal, 12))
            .margins(20, 0, 24, 24)
            .build();

        let title_box = Box::new(Orientation::Vertical, 4);
        title_box.set_hexpand(true);

        let title = Label::new(Some("Password"));
        title.add_css_class("title-3");
        title.set_halign(gtk4::Align::Start);

        let subtitle = Label::new(Some("Account password and security"));
        subtitle.add_css_class("dim-label");
        subtitle.add_css_class("caption");
        subtitle.set_halign(gtk4::Align::Start);

        title_box.append(&title);
        title_box.append(&subtitle);

        let generate_button = Button::new();
        generate_button.set_label("Generate New");
        generate_button.add_css_class("suggested-action");
        generate_button.set_valign(gtk4::Align::Center);

        header_box.append(&title_box);
        if self.edit_mode {
            header_box.append(&generate_button);
        }

        section.append(&header_box);

        let password_box = CreateBox::new()
            .new_box(Box::new(Orientation::Horizontal, 12))
            .margins(0, 24, 24, 24)
            .halign(Align::Center)
            .build();

        let password_entry = Entry::new();
        let account = account_rc.borrow();

        if self.edit_mode {
            account.password.with_exposed(|password| {
                password_entry.set_text(password);
            });
        } else {
            let masked_password = account.password.masked(8);
            password_entry.set_text(&masked_password);
        }

        password_entry.set_editable(self.edit_mode);
        password_entry.set_hexpand(true);
        password_entry.set_size_request(250, -1); // Reduced width for better responsiveness
        password_entry.add_css_class("password-field");

        // Connect password changes in edit mode
        if self.edit_mode {
            let account_rc_edit = account_rc.clone();
            password_entry.connect_changed(move |entry| {
                let text = entry.text().to_string();
                let mut account = account_rc_edit.borrow_mut();
                account.password.update(text);
            });
        }

        // Connect generate button functionality (only in edit mode)
        if self.edit_mode {
            let password_entry_gen = password_entry.clone();
            let account_rc_gen = account_rc.clone();
            generate_button.connect_clicked(move |_| {
                show_password_generator_dialog(&password_entry_gen, &account_rc_gen);
            });
        }

        let reveal_button = Button::new();
        let reveal_button_content = ButtonContent::builder()
            .icon_name("view-reveal-symbolic")
            .build();
        reveal_button.set_child(Some(&reveal_button_content));
        reveal_button.add_css_class("flat");
        reveal_button.set_tooltip_text(Some("Show/Hide Password"));

        // Add reveal/hide functionality (only in view mode)
        if !self.edit_mode {
            let password_entry_clone = password_entry.clone();
            let account_rc_clone = account_rc.clone();
            let is_revealed = Rc::new(RefCell::new(false));
            let is_revealed_clone = is_revealed.clone();

            reveal_button.connect_clicked(move |_| {
                let mut revealed = is_revealed_clone.borrow_mut();
                let account = account_rc_clone.borrow();

                if *revealed {
                    // Hide password
                    let masked_password = account.password.masked(8);
                    password_entry_clone.set_text(&masked_password);
                    *revealed = false;
                } else {
                    // Show password using secure exposure
                    account.password.with_exposed(|password| {
                        password_entry_clone.set_text(password);
                    });
                    *revealed = true;
                }
            });
        }

        let copy_button = Button::new();
        let copy_button_content = ButtonContent::builder()
            .icon_name("edit-copy-symbolic")
            .build();
        copy_button.set_child(Some(&copy_button_content));
        copy_button.add_css_class("flat");
        copy_button.set_tooltip_text(Some("Copy Password"));

        let account_rc_copy = account_rc.clone();
        copy_button.connect_clicked(move |button| {
            let account = account_rc_copy.borrow();
            let display = button.display();
            let clipboard = display.clipboard();

            let password_copy = account.password.expose_for_clipboard();
            clipboard.set_text(&password_copy);

            // Schedule clipboard clearing after 30 seconds for security
            let clipboard_clone = clipboard.clone();
            glib::timeout_add_seconds_local(30, move || {
                clipboard_clone.set_text("");
                println!("Clipboard cleared for security");
                glib::ControlFlow::Break
            });
        });

        password_box.append(&password_entry);

        // Only show reveal button in view mode
        if !self.edit_mode {
            password_box.append(&reveal_button);
        }

        password_box.append(&copy_button);
        section.append(&password_box);

        section
    }

    fn additional_fields_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 16);
        section.add_css_class("account-section");
        section.set_margin_top(16);

        let header_box = CreateBox::new()
            .new_box(Box::new(Orientation::Horizontal, 12))
            .margins(20, 0, 24, 24)
            .build();

        let title_box = Box::new(Orientation::Vertical, 4);
        title_box.set_hexpand(true);

        let title = Label::new(Some("Additional Fields"));
        title.add_css_class("title-3");
        title.set_halign(gtk4::Align::Start);

        let subtitle = Label::new(Some("Custom fields for additional account information"));
        subtitle.add_css_class("dim-label");
        subtitle.add_css_class("caption");
        subtitle.set_halign(gtk4::Align::Start);

        title_box.append(&title);
        title_box.append(&subtitle);

        let add_button = Button::new();
        add_button.set_label("Add Field");
        add_button.add_css_class("suggested-action");
        add_button.set_valign(gtk4::Align::Center);

        if self.edit_mode {
            let account_rc_clone = account_rc.clone();
            let content_area_clone = self.content_area.clone();
            let vault_name_clone = self.vault_name.to_string();
            add_button.connect_clicked(move |_| {
                show_add_field_dialog(&account_rc_clone, &content_area_clone, &vault_name_clone);
            });
        }

        header_box.append(&title_box);
        if self.edit_mode {
            header_box.append(&add_button);
        }

        section.append(&header_box);

        let fields_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 8))
            .margins(0, 20, 24, 24)
            .halign(Align::Center)
            .build();

        let account = account_rc.borrow();
        for (field_name, field_value) in &account.additional_fields {
            // Create a vertical box for each field (field name + field controls)
            let field_container = Box::new(Orientation::Vertical, 4);

            let field_label = Label::new(Some(field_name));
            field_label.add_css_class("field-label");
            field_label.set_halign(gtk4::Align::Start);
            field_label.set_margin_start(4);

            let password_field_box = Box::new(Orientation::Horizontal, 8);
            password_field_box.set_halign(gtk4::Align::Center);

            let field_entry = Entry::new();
            field_entry.set_text(field_value);
            field_entry.set_editable(self.edit_mode);
            field_entry.set_hexpand(true);
            field_entry.set_size_request(350, -1); // Make textbox longer
            field_entry.add_css_class("password-field");
            field_entry.set_placeholder_text(Some(&format!("Enter {}", field_name.to_lowercase())));

            // Connect field changes in edit mode
            if self.edit_mode {
                let account_rc_field = account_rc.clone();
                let field_name_clone = field_name.clone();
                field_entry.connect_changed(move |entry| {
                    let text = entry.text().to_string();
                    let mut account = account_rc_field.borrow_mut();
                    account
                        .additional_fields
                        .insert(field_name_clone.clone(), text);
                });
            }

            // Copy button for the field
            let copy_button = Button::new();
            let copy_button_content = ButtonContent::builder()
                .icon_name("edit-copy-symbolic")
                .build();
            copy_button.set_child(Some(&copy_button_content));
            copy_button.add_css_class("flat");
            copy_button.set_tooltip_text(Some(&format!("Copy {field_name}")));

            // Add copy functionality
            let field_value_copy = field_value.clone();
            copy_button.connect_clicked(move |button| {
                let display = button.display();
                let clipboard = display.clipboard();
                clipboard.set_text(&field_value_copy);

                // Schedule clipboard clearing after 30 seconds for security
                let clipboard_clone = clipboard.clone();
                glib::timeout_add_seconds_local(30, move || {
                    clipboard_clone.set_text("");
                    glib::ControlFlow::Break
                });
            });

            password_field_box.append(&field_entry);
            password_field_box.append(&copy_button);

            // Add edit/delete buttons in edit mode
            if self.edit_mode {
                let edit_button = Button::new();
                let edit_button_content = ButtonContent::builder()
                    .icon_name("document-edit-symbolic")
                    .build();
                edit_button.set_child(Some(&edit_button_content));
                edit_button.add_css_class("flat");
                edit_button.set_tooltip_text(Some("Edit field"));

                let delete_button = Button::new();
                let delete_button_content = ButtonContent::builder()
                    .icon_name("user-trash-symbolic")
                    .build();
                delete_button.set_child(Some(&delete_button_content));
                delete_button.add_css_class("flat");
                delete_button.add_css_class("destructive-action");
                delete_button.set_tooltip_text(Some("Delete field"));

                let account_rc_edit = account_rc.clone();
                let content_area_edit = self.content_area.clone();
                let vault_name_edit = self.vault_name.to_string();
                let field_name_edit = field_name.clone();
                edit_button.connect_clicked(move |_| {
                    show_edit_field_dialog(
                        &account_rc_edit,
                        &content_area_edit,
                        &field_name_edit,
                        &vault_name_edit,
                    );
                });

                let account_rc_delete = account_rc.clone();
                let content_area_delete = self.content_area.clone();
                let vault_name_delete = self.vault_name.to_string();
                let field_name_delete = field_name.clone();
                delete_button.connect_clicked(move |_| {
                    show_delete_field_dialog(
                        &account_rc_delete,
                        &content_area_delete,
                        &vault_name_delete,
                        &field_name_delete,
                    );
                });

                password_field_box.append(&edit_button);
                password_field_box.append(&delete_button);
            }

            field_container.append(&field_label);
            field_container.append(&password_field_box);
            fields_box.append(&field_container);
        }

        // If no additional fields, show a placeholder
        if account.additional_fields.is_empty() {
            let placeholder_box = Box::new(Orientation::Horizontal, 8);
            placeholder_box.set_halign(gtk4::Align::Center);

            let placeholder = Label::new(Some("No additional fields"));
            placeholder.add_css_class("dim-label");
            placeholder.set_halign(gtk4::Align::Start);
            placeholder.set_hexpand(true);

            placeholder_box.append(&placeholder);
            fields_box.append(&placeholder_box);
        }

        section.append(&fields_box);
        section
    }

    fn notes_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 20);
        section.add_css_class("account-section");
        section.set_margin_top(20);

        let header_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 8))
            .margins(20, 0, 24, 24)
            .build();

        let title = Label::new(Some("Notes"));
        title.add_css_class("title-3");
        title.set_halign(gtk4::Align::Start);

        let subtitle = Label::new(Some("Additional notes and information"));
        subtitle.add_css_class("dim-label");
        subtitle.add_css_class("caption");
        subtitle.set_halign(gtk4::Align::Start);

        header_box.append(&title);
        header_box.append(&subtitle);
        section.append(&header_box);

        let notes_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 16))
            .margins(0, 24, 24, 24)
            .halign(Align::Center)
            .build();

        // Notes text area (using Entry for now, could be TextView for multiline)
        let notes_entry = Entry::new();
        let account = account_rc.borrow();
        notes_entry.set_text(&account.notes);
        notes_entry.set_hexpand(true);
        notes_entry.set_size_request(250, -1); // Reduced width for better responsiveness
        notes_entry.set_editable(self.edit_mode);
        notes_entry.add_css_class("notes-field");

        // Connect notes changes in edit mode
        if self.edit_mode {
            let account_rc_notes = account_rc.clone();
            notes_entry.connect_changed(move |entry| {
                let text = entry.text().to_string();
                let mut account = account_rc_notes.borrow_mut();
                account.notes = text;
            });
        }

        notes_box.append(&notes_entry);
        section.append(&notes_box);
        section
    }

    fn actions_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 16);

        let separator = Separator::new(Orientation::Horizontal);
        section.append(&separator);

        let actions_box = Box::new(Orientation::Horizontal, 12);
        actions_box.set_halign(gtk4::Align::Center);

        let save_button = Button::new();
        save_button.set_label("Save Changes");
        save_button.add_css_class("suggested-action");
        save_button.set_size_request(120, -1);

        // Add save functionality
        let account_rc_clone = account_rc.clone();
        let vault_name_clone = self.vault_name.clone();
        let content_area_clone = self.content_area.clone();
        save_button.connect_clicked(move |_| {
            let mut account = account_rc_clone.borrow_mut();
            account.update_modified_time();
            let account_name = account.name.clone();

            match update_account(&vault_name_clone, &account) {
                Ok(()) => {
                    drop(account); // Release the borrow
                    // Exit edit mode and show the updated account
                    AccountView::new(&content_area_clone, &vault_name_clone, &account_name, false)
                        .create();
                }
                Err(e) => {
                    eprintln!("Failed to save account: {e}");
                }
            }
        });

        let cancel_button = Button::new();
        cancel_button.set_label("Cancel");
        cancel_button.set_size_request(120, -1);

        let vault_name_clone2 = self.vault_name.clone();
        let content_area_clone2 = self.content_area.clone();
        let account_rc_clone3 = account_rc.clone();
        cancel_button.connect_clicked(move |_| {
            let account = account_rc_clone3.borrow();
            AccountView::new(
                &content_area_clone2,
                &vault_name_clone2,
                &account.name,
                false,
            )
            .create();
        });

        actions_box.append(&cancel_button);
        actions_box.append(&save_button);

        section.append(&actions_box);
        section
    }
}
