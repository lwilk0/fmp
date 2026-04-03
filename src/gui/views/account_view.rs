use crate::{
    gui::{
        content::{
            CreateBox, CreateScrollableView, clear_content, create_editable_field_row,
            create_field_row,
        },
        dialogs::{
            account_management::{
                show_add_field_dialog, show_delete_field_dialog, show_edit_field_dialog,
                show_rename_account_dialog,
            },
            common::show_confirmation_dialog,
            password_generator::show_password_generator_dialog,
        },
        views::{home_view::HomeView, vault_view::VaultView},
    },
    vault::{Account, create_account, delete_account, get_full_account_details, update_account},
};
use adw::{ButtonContent, PreferencesGroup, prelude::*};
use gtk4::{
    Align, Box, Button, Entry, Label, Orientation, PolicyType, ScrolledWindow, Separator, TextView,
    pango::EllipsizeMode,
};
use std::{cell::RefCell, rc::Rc};
pub struct AccountView<'a> {
    content_area: &'a Box,
    vault_name: String,
    account_name: String,
    edit_mode: bool,
}

/// Shows the new account creation view
pub fn show_new_account_view(content_area: &Box, vault_name: &str) {
    clear_content(content_area);

    let main_box = CreateBox::new()
        .new_box(Box::new(Orientation::Vertical, 24))
        .build();

    let scrollable = CreateScrollableView::new()
        .max_width(800)
        .tighten_threshold(600)
        .build();

    let header_box = Box::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Create New Account"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);

    let subtitle = Label::new(Some("Enter the details for your new account"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Start);

    header_box.append(&title);
    header_box.append(&subtitle);
    main_box.append(&header_box);

    let new_account = Rc::new(RefCell::new(Account::default()));

    let form_box = Box::new(Orientation::Vertical, 16);

    // Account name
    let name_row = create_editable_field_row("Account Name", "", &new_account, "name");
    form_box.append(&name_row);
    form_box.set_margin_top(0);

    // Account type
    let type_row = create_editable_field_row(
        "Account Type",
        "Password Account",
        &new_account,
        "account_type",
    ); // Account name

    form_box.append(&type_row);

    // Website
    let website_row = create_editable_field_row("Website", "", &new_account, "website");
    form_box.append(&website_row);

    // Username
    let username_row = create_editable_field_row("Username", "", &new_account, "username");
    form_box.append(&username_row);

    // Password (with show/hide functionality)
    let password_row = create_password_field_row("Password", "", &new_account);
    form_box.append(&password_row);

    // Notes
    let notes_row = create_editable_field_row("Notes", "", &new_account, "notes");
    form_box.append(&notes_row);

    main_box.append(&form_box);

    let actions_box = CreateBox::new()
        .new_box(Box::new(Orientation::Horizontal, 12))
        .halign(Align::Center)
        .build();

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();

    cancel_button.connect_clicked(move |_| {
        VaultView::new(&content_area_clone, &vault_name_clone).create();
    });

    let create_button = Button::new();
    create_button.set_label("Create Account");
    create_button.add_css_class("suggested-action");
    create_button.set_size_request(120, -1);

    let account_rc_clone = new_account.clone();

    let content_area_clone2 = content_area.clone();
    let vault_name_clone2 = vault_name.to_string();

    create_button.connect_clicked(move |_| {
        let account = account_rc_clone.borrow();

        if account.name.is_empty() {
            return;
        }

        match create_account(&vault_name_clone2, &account) {
            Ok(()) => {
                VaultView::new(&content_area_clone2, &vault_name_clone2).create();
            }
            Err(e) => {
                log::error!("Failed to create account: {e}");
            }
        }
    });

    actions_box.append(&cancel_button);
    actions_box.append(&create_button);
    main_box.append(&actions_box);

    scrollable.set_child(Some(&main_box));
    content_area.append(&scrollable);
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

        let account_type = if account.account_type.trim().is_empty() {
            "Account".to_string()
        } else {
            account.account_type.clone()
        };

        let subtitle_text = format!("{account_type} in {0}", self.vault_name);

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

        let buttons_box = Box::new(Orientation::Horizontal, 4);
        buttons_box.set_hexpand(true);
        buttons_box.add_css_class("button_fix");

        if !self.edit_mode {
            let back_button = Button::new();
            back_button.set_label("Back");
            back_button.set_tooltip_text(Some("Go back to Vault"));
            back_button.add_css_class("suggested-action");

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
                                log::error!("Failed to delete account '{account_name_confirm}': {e}");
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

            let content_area_back = self.content_area.clone();
            let vault_name_back = self.vault_name.to_string();
            back_button.connect_clicked(move |_| {
                VaultView::new(&content_area_back, &vault_name_back).create();
            });

            buttons_box.append(&back_button);
            buttons_box.append(&edit_button);
            buttons_box.append(&rename_button);
            buttons_box.append(&delete_button);
        }

        actions_box.append(&buttons_box);

        header_box.append(&info_box);
        header_box.append(&actions_box);

        header_box
    }

    fn details_section(&self, account_rc: &Rc<RefCell<Account>>) -> PreferencesGroup {
        let group = PreferencesGroup::new();
        group.set_title("Account Details");
        group.set_description(Some("Basic account information and credentials"));
        group.add_css_class("group_background");

        let account = account_rc.borrow();

        if self.edit_mode {
            // Editable fields in edit mode
            group.add(&create_editable_field_row(
                "Website",
                &account.website,
                account_rc,
                "website",
            ));

            group.add(&create_editable_field_row(
                "Username",
                &account.username,
                account_rc,
                "username",
            ))
        } else {
            // Read-only fields in view mode
            group.add(&create_field_row("Website", &account.website, true));

            group.add(&create_field_row("Username", &account.username, true))
        }

        group.add(&create_field_row("Created", &account.created_at, false));

        group.add(&create_field_row(
            "Last Modified",
            &account.modified_at,
            false,
        ));

        group
    }

    fn password_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 20);
        section.add_css_class("account-section");

        let group = PreferencesGroup::new();
        group.set_title("Password");
        group.set_description(Some("Account password"));
        group.add_css_class("group_background");

        let generate_button = Button::new();
        generate_button.set_label("Generate New");
        generate_button.add_css_class("suggested-action");
        generate_button.set_valign(gtk4::Align::Center);

        let password_box = CreateBox::new()
            .new_box(Box::new(Orientation::Horizontal, 12))
            .margins(0, 24, 24, 24)
            .halign(Align::Center)
            .build();

        if self.edit_mode {
            password_box.append(&generate_button);
        }

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

            glib::timeout_add_seconds_local(30, move || {
                clipboard.set_text("");
                log::info!("Clipboard cleared for security");
                glib::ControlFlow::Break
            });
        });

        password_box.append(&password_entry);

        // Only show reveal button in view mode
        if !self.edit_mode {
            password_box.append(&reveal_button);
        }
        password_box.append(&copy_button);

        group.add(&password_box);
        section.append(&group);

        section
    }

    fn additional_fields_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 16);
        section.add_css_class("account-section");

        let group = PreferencesGroup::new();
        group.set_title("Additional Fields");
        group.set_description(Some("Custom fields for additional account information"));
        group.add_css_class("group_background");

        let add_button = Button::new();
        add_button.set_label("Add Field");
        add_button.add_css_class("suggested-action");
        add_button.set_halign(gtk4::Align::Center);

        if self.edit_mode {
            let account_rc_clone = account_rc.clone();
            let content_area_clone = self.content_area.clone();
            let vault_name_clone = self.vault_name.to_string();
            add_button.connect_clicked(move |_| {
                show_add_field_dialog(&account_rc_clone, &content_area_clone, &vault_name_clone);
            });
        }

        if self.edit_mode {
            group.add(&add_button);
        }

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
            field_label.add_css_class("dim-label");
            field_label.set_halign(gtk4::Align::Start);
            field_label.set_hexpand(true);

            let password_field_box = Box::new(Orientation::Horizontal, 8);
            // password_field_box.set_halign(gtk4::Align::Center);

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
                delete_button.set_tooltip_text(Some("Delete field"));

                let account_rc_edit = account_rc.clone();
                let content_area_edit = self.content_area.clone();
                let vault_name_edit = self.vault_name.to_string();
                let field_name_edit = field_name.clone();
                edit_button.connect_clicked(move |_| {
                    show_edit_field_dialog(
                        &account_rc_edit,
                        &content_area_edit,
                        &vault_name_edit,
                        &field_name_edit,
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

        group.add(&fields_box);
        section.append(&group);
        section
    }

    fn notes_section(&self, account_rc: &Rc<RefCell<Account>>) -> Box {
        let section = Box::new(Orientation::Vertical, 20);
        section.add_css_class("account-section");

        let group = PreferencesGroup::new();
        group.set_title("Notes");
        group.set_description(Some("Additional notes and information"));
        group.add_css_class("group_background");

        let notes_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 16))
            .margins(0, 24, 24, 24)
            .halign(Align::Center)
            .build();

        let notes_entry = TextView::new();
        let account = account_rc.borrow();
        let note = notes_entry.buffer();
        note.set_text(&account.notes);

        notes_entry.set_hexpand(true);
        notes_entry.set_size_request(250, -1);
        notes_entry.set_editable(self.edit_mode);
        notes_entry.set_width_request(450);
        notes_entry.set_wrap_mode(gtk4::WrapMode::WordChar);
        notes_entry.add_css_class("notes-field");

        if self.edit_mode {
            let account_rc_notes = account_rc.clone();
            let buffer = notes_entry.buffer();
            buffer.connect_changed(move |buf| {
                let text = buf.text(&buf.start_iter(), &buf.end_iter(), true);
                let mut account = account_rc_notes.borrow_mut();
                account.notes = text.to_string();
            });
        }

        notes_box.append(&notes_entry);
        group.add(&notes_box);
        section.append(&group);
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

        let account_rc_clone = account_rc.clone();
        let vault_name_clone = self.vault_name.clone();
        let content_area_clone = self.content_area.clone();
        save_button.connect_clicked(move |_| {
            let mut account = account_rc_clone.borrow_mut();
            account.update_modified_time();
            let account_name = account.name.clone();

            match update_account(&vault_name_clone, &account) {
                Ok(()) => {
                    drop(account);
                    // Exit edit mode and show the updated account
                    AccountView::new(&content_area_clone, &vault_name_clone, &account_name, false)
                        .create();
                }
                Err(e) => {
                    log::error!("Failed to save account: {e}");
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

/// Creates a password field row with show/hide functionality for account creation
fn create_password_field_row(
    label_text: &str,
    initial_value: &str,
    account_rc: &Rc<RefCell<Account>>,
) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 16);
    row_box.set_halign(gtk4::Align::Fill);
    row_box.set_margin_top(2);
    row_box.set_margin_bottom(2);

    let label = Label::new(Some(label_text));
    label.add_css_class("dim-label");
    label.set_halign(gtk4::Align::Start);
    label.set_valign(gtk4::Align::Center);
    label.set_size_request(160, -1);
    label.set_ellipsize(EllipsizeMode::End);
    label.set_max_width_chars(25);
    label.set_tooltip_text(Some(label_text));

    let password_container = Box::new(Orientation::Horizontal, 8);
    password_container.set_hexpand(true);

    let entry = Entry::new();
    entry.set_text(initial_value);
    entry.set_hexpand(true);
    entry.set_size_request(250, -1);
    entry.set_visibility(false);
    entry.set_invisible_char(Some('•'));

    let generate_button = Button::new();
    generate_button.set_label("Generate");
    generate_button.add_css_class("flat");
    generate_button.set_tooltip_text(Some("Generate Password"));

    let reveal_button = Button::new();
    let reveal_button_content = ButtonContent::builder()
        .icon_name("view-reveal-symbolic")
        .build();
    reveal_button.set_child(Some(&reveal_button_content));
    reveal_button.add_css_class("flat");
    reveal_button.set_tooltip_text(Some("Show/Hide Password"));

    // Connect entry changes to account data
    let account_rc_clone = account_rc.clone();
    entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        let mut account = account_rc_clone.borrow_mut();
        account.password.update(text);
    });

    let entry_gen = entry.clone();
    let account_rc_gen = account_rc.clone();
    generate_button.connect_clicked(move |_| {
        show_password_generator_dialog(&entry_gen, &account_rc_gen);
    });

    let entry_clone = entry.clone();
    let is_revealed = Rc::new(RefCell::new(false));
    let is_revealed_clone = is_revealed.clone();
    reveal_button.connect_clicked(move |_| {
        let mut revealed = is_revealed_clone.borrow_mut();
        *revealed = !*revealed;
        entry_clone.set_visibility(*revealed);
    });

    password_container.append(&entry);
    password_container.append(&generate_button);
    password_container.append(&reveal_button);

    row_box.append(&label);
    row_box.append(&password_container);

    row_box
}
