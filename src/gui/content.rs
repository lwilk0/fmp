use crate::{
    gui::{
        dialogs::{
            show_backup_vault_dialog, show_delete_backup_dialog, show_delete_vault_dialog,
            show_password_generator_dialog, show_rename_vault_dialog, show_restore_vault_dialog,
            show_totp_authentication_dialog, show_totp_management_dialog, show_totp_setup_dialog,
        },
        views::{account_view::AccountView, home_view::HomeView, vault_view::VaultView},
        widgets::loading_spinner::{create_loading_button, set_button_loading_state},
    },
    storage::filesystem::{backup_exists, read_directory},
    totp::{is_totp_enabled, is_totp_required},
    vault::{Account, Locations, create_account, create_vault, warm_up_gpg},
};
use adw::{ActionRow, ButtonContent, Clamp, PreferencesGroup, prelude::*};

use gtk4::{
    Align, Box, Button, Entry, Label, Orientation, PolicyType, ScrolledWindow, pango::EllipsizeMode,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::atomic::AtomicU64};

// Global counter for tracking vault loading operations
pub static VAULT_LOADING_COUNTER: AtomicU64 = AtomicU64::new(0);

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

    let content_area_clone2 = content_area.clone();
    let vault_name_clone2 = vault_name.to_string();
    cancel_button.connect_clicked(move |_| {
        VaultView::new(&content_area_clone2, &vault_name_clone2).create();
    });

    let create_button = Button::new();
    create_button.set_label("Create Account");
    create_button.add_css_class("suggested-action");
    create_button.set_size_request(120, -1);

    let account_rc_clone = new_account.clone();
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    create_button.connect_clicked(move |_| {
        let account = account_rc_clone.borrow();

        if account.name.is_empty() {
            return;
        }

        match create_account(&vault_name_clone, &account) {
            Ok(()) => {
                VaultView::new(&content_area_clone, &vault_name_clone).create();
            }
            Err(e) => {
                eprintln!("Failed to create account: {e}");
            }
        }
    });

    actions_box.append(&cancel_button);
    actions_box.append(&create_button);
    main_box.append(&actions_box);

    scrollable.set_child(Some(&main_box));
    content_area.append(&scrollable);
}

/// Shows the create vault view
pub fn show_create_vault_view(content_area: &Box) {
    clear_content(content_area);

    let main_box = CreateBox::new()
        .new_box(Box::new(Orientation::Vertical, 24))
        .margins(48, 48, 48, 48)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let title = Label::new(Some("Create New Vault"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Center);

    let subtitle = Label::new(Some("Enter a name for your new vault and select a GPG key"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Center);

    main_box.append(&title);
    main_box.append(&subtitle);

    let form_box = Box::new(Orientation::Vertical, 16);
    form_box.set_size_request(400, -1);

    let name_label = Label::new(Some("Vault Name"));
    name_label.set_halign(gtk4::Align::Start);
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Enter vault name"));

    let recipient_label = Label::new(Some("GPG Key ID"));
    recipient_label.set_halign(gtk4::Align::Start);
    let recipient_entry = Entry::new();
    recipient_entry.set_placeholder_text(Some("Enter GPG key ID or email"));

    form_box.append(&name_label);
    form_box.append(&name_entry);
    form_box.append(&recipient_label);
    form_box.append(&recipient_entry);

    main_box.append(&form_box);

    let actions_box = CreateBox::new()
        .new_box(Box::new(Orientation::Horizontal, 12))
        .halign(Align::Center)
        .build();

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    let content_area_clone2 = content_area.clone();
    cancel_button.connect_clicked(move |_| HomeView::new(&content_area_clone2).create());

    let (create_button, _) = create_loading_button("Create Vault", "Creating vault...");
    create_button.add_css_class("suggested-action");
    create_button.set_size_request(120, -1);

    let content_area_clone = content_area.clone();
    create_button.connect_clicked(move |button| {
        let vault_name = name_entry.text().to_string();
        let recipient = recipient_entry.text().to_string();

        if vault_name.is_empty() || recipient.is_empty() {
            return;
        }

        set_button_loading_state(button, true);

        // Use a timeout to simulate async operation and allow UI to update
        glib::timeout_add_local(std::time::Duration::from_millis(100), {
            let content_area_clone = content_area_clone.clone();
            let vault_name = vault_name.clone();
            let recipient = recipient.clone();
            let button = button.clone();

            move || {
                match create_vault(&vault_name, &recipient) {
                    Ok(()) => {
                        // Refresh sidebar to show new vault
                        crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area_clone);
                        proceed_with_gate_warmup(&content_area_clone, &vault_name);
                    }
                    Err(e) => {
                        eprintln!("Failed to create vault: {e}");
                        set_button_loading_state(&button, false);
                    }
                }
                glib::ControlFlow::Break
            }
        });
    });

    actions_box.append(&cancel_button);
    actions_box.append(&create_button);
    main_box.append(&actions_box);

    content_area.append(&main_box);
}

/// Creates a modern list layout for accounts using `PreferencesGroup`
pub fn create_accounts_grid(content_area: &Box, vault_name: &str) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Accounts");
    group.set_description(Some("Manage your account credentials"));

    let all_accounts = get_available_accounts(vault_name);

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();

    if all_accounts.is_empty() {
        group.add(
            &CreateActionRow::new()
                .title("No Accounts Yet")
                .subtitle("Create your first account to get started")
                .button_label("Add Account")
                .css_class("suggested-action")
                .callback({
                    let content_area = content_area_clone.clone();
                    let vault_name = vault_name_clone.clone();
                    move || show_new_account_view(&content_area, &vault_name)
                })
                .build(),
        );
    } else {
        group.add(
            &CreateActionRow::new()
                .title("Add New Account")
                .subtitle("Create a new account directory")
                .button_label("Add")
                .css_class("suggested-action")
                .callback({
                    let content_area = content_area_clone.clone();
                    let vault_name = vault_name_clone.clone();
                    move || show_new_account_view(&content_area, &vault_name)
                })
                .build(),
        );

        for account_name in all_accounts {
            let account_row = create_account_row(account_name.as_str(), content_area, vault_name);
            group.add(&account_row);
        }
    }

    group
}

/// Creates an account row for the preferences group
fn create_account_row(account_name: &str, content_area: &Box, vault_name: &str) -> ActionRow {
    CreateActionRow::new()
        .title(account_name)
        .subtitle("Password Account")
        .button_label("View")
        .css_class("flat")
        .callback({
            let account_name_clone = account_name.to_string();
            let vault_name_clone = vault_name.to_string();
            let content_area_clone = content_area.clone();
            move || {
                AccountView::new(
                    &content_area_clone,
                    &vault_name_clone,
                    &account_name_clone,
                    false,
                )
                .create()
            }
        })
        .build()
}

/// Creates the TOTP (2FA) management section for the vault view
pub fn create_totp_management_section(content_area: &Box, vault_name: &str) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Two-Factor Authentication");
    group.set_description(Some("Secure your vault with TOTP authentication"));

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();

    let is_enabled = is_totp_enabled(vault_name);

    if is_enabled {
        group.add(
            &CreateActionRow::new()
                .title("TOTP Status")
                .subtitle("Two-factor authentication is enabled and active")
                .button_label("Manage")
                .css_class("flat")
                .callback({
                    let content_area = content_area_clone.clone();
                    let vault_name = vault_name_clone.clone();
                    move || show_totp_management_dialog(&vault_name, &content_area)
                })
                .build(),
        );
    } else {
        group.add(
            &CreateActionRow::new()
                .title("Enable Two-Factor Authentication")
                .subtitle("Add an extra layer of security to your vault")
                .button_label("Enable 2FA")
                .css_class("suggested-action")
                .callback({
                    let vault_name = vault_name_clone.clone();
                    move || show_totp_setup_dialog(&vault_name)
                })
                .build(),
        );
    }

    group
}

/// Creates the vault management section with backup and vault operations
pub fn create_vault_management_section(content_area: &Box, vault_name: &str) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Vault Management");
    group.set_description(Some("Backup, restore, rename, and delete vault operations"));

    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();

    group.add(
        &CreateActionRow::new()
            .title("Create Backup")
            .subtitle("Create a backup of this vault")
            .button_label("Backup")
            .css_class("flat")
            .callback({
                let vault_name = vault_name_clone.clone();
                let content_area = content_area_clone.clone();
                move || show_backup_vault_dialog(&vault_name, &content_area)
            })
            .build(),
    );

    let has_backup = backup_exists(vault_name);

    group.add(
        &CreateActionRow::new()
            .title("Restore Backup")
            .subtitle("Restore vault from backup")
            .button_label("Restore")
            .css_class("flat")
            .activatable(has_backup)
            .callback({
                let vault_name = vault_name_clone.clone();
                let content_area = content_area_clone.clone();
                move || show_restore_vault_dialog(&vault_name, &content_area)
            })
            .build(),
    );

    group.add(
        &CreateActionRow::new()
            .title("Delete Backup")
            .subtitle("Delete vault backup")
            .button_label("Delete")
            .css_class("destructive-action")
            .activatable(has_backup)
            .callback({
                let vault_name = vault_name_clone.clone();
                let content_area = content_area_clone.clone();
                move || show_delete_backup_dialog(&vault_name, &content_area)
            })
            .build(),
    );

    group.add(
        &CreateActionRow::new()
            .title("Rename Vault")
            .subtitle("Change the name of this vault")
            .button_label("Rename")
            .css_class("flat")
            .callback({
                let vault_name = vault_name_clone.clone();
                let content_area = content_area_clone.clone();
                move || show_rename_vault_dialog(&vault_name, &content_area)
            })
            .build(),
    );

    group.add(
        &CreateActionRow::new()
            .title("Delete Vault")
            .subtitle("Permanently delete this vault and its data")
            .button_label("Delete")
            .css_class("destructive-action")
            .callback({
                let vault_name = vault_name_clone;
                let content_area = content_area_clone;
                move || show_delete_vault_dialog(&vault_name, &content_area)
            })
            .build(),
    );

    group
}

/// Creates a field row with label and value
pub fn create_field_row(label_text: &str, value_text: &str, copyable: bool) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 16);
    row_box.set_halign(gtk4::Align::Fill);
    row_box.set_margin_top(4);
    row_box.set_margin_bottom(4);
    

    let label = Label::new(Some(label_text));
    label.add_css_class("dim-label");
    label.set_halign(gtk4::Align::Start);
    label.set_valign(gtk4::Align::Center);
    label.set_size_request(160, -1);
    label.set_ellipsize(EllipsizeMode::End);
    label.set_max_width_chars(25);
    label.set_tooltip_text(Some(label_text));

    let value_box = Box::new(Orientation::Horizontal, 8);
    value_box.set_hexpand(true);

    let value_entry = Entry::new();
    value_entry.set_text(value_text);
    value_entry.set_editable(false);
    value_entry.set_hexpand(true);
    value_entry.set_size_request(250, -1);
    value_entry.add_css_class("password-field");
    value_entry.set_tooltip_text(Some(value_text));

    value_box.append(&value_entry);

    if copyable {
        let copy_button = Button::new();
        let copy_button_content = ButtonContent::builder()
            .icon_name("edit-copy-symbolic")
            .build();
        copy_button.set_child(Some(&copy_button_content));
        copy_button.add_css_class("flat");
        copy_button.set_tooltip_text(Some("Copy to clipboard"));

        // Add copy functionality
        let value_text_owned = value_text.to_string();
        copy_button.connect_clicked(move |button| {
            let display = button.display();
            let clipboard = display.clipboard();
            clipboard.set_text(&value_text_owned);

            // Schedule clipboard clearing after 60 seconds for non-password fields
            let clipboard_clone = clipboard.clone();
            glib::timeout_add_seconds_local(60, move || {
                clipboard_clone.set_text("");
                glib::ControlFlow::Break
            });
        });

        value_box.append(&copy_button);
    }

    row_box.append(&label);
    row_box.append(&value_box);

    row_box
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

/// Creates an editable field row for account creation/editing
pub fn create_editable_field_row(
    label_text: &str,
    initial_value: &str,
    account_rc: &Rc<RefCell<Account>>,
    field_name: &str,
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

    let entry = Entry::new();
    entry.set_text(initial_value);
    entry.set_hexpand(true);
    entry.set_size_request(250, -1);
    //entry.add_css_class("password-field");

    let account_rc_clone = account_rc.clone();
    let field_name_owned = field_name.to_string();
    entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        let mut account = account_rc_clone.borrow_mut();

        match field_name_owned.as_str() {
            "name" => account.name = text,
            "account_type" => account.account_type = text,
            "website" => account.website = text,
            "username" => account.username = text,
            "password" => account.password.update(text),
            "notes" => account.notes = text,
            _ => {}
        }
    });

    row_box.append(&label);
    row_box.append(&entry);

    row_box
}

/// Clears all content from the content area
pub fn clear_content(content_area: &Box) {
    while let Some(child) = content_area.first_child() {
        content_area.remove(&child);
    }
}

/// Proceeds with gate warm-up and then shows the vault view
pub fn proceed_with_gate_warmup(content_area: &Box, vault_name: &str) {
    // Check if TOTP is required for this vault
    if is_totp_required(vault_name) {
        let content_area_clone = content_area.clone();
        let vault_name_clone = vault_name.to_string();

        show_totp_authentication_dialog(&vault_name_clone.clone(), move || {
            proceed_with_gpg_warmup(&content_area_clone, &vault_name_clone);
        });
    } else {
        proceed_with_gpg_warmup(content_area, vault_name);
    }
}

/// Proceeds with GPG warm-up after TOTP verification (if required)
fn proceed_with_gpg_warmup(content_area: &Box, vault_name: &str) {
    // Attempt to warm up GPG by decrypting the gate file
    match warm_up_gpg(vault_name) {
        Ok(()) => {
            VaultView::new(content_area, vault_name).create();
        }
        Err(e) => {
            show_error_message(
                content_area,
                "Failed to Access Vault",
                &format!("Could not decrypt vault gate file: {e}"),
            );
        }
    }
}

/// Shows an error message in the content area
fn show_error_message(content_area: &Box, title: &str, message: &str) {
    clear_content(content_area);

    let main_box = CreateBox::new()
        .new_box(Box::new(Orientation::Vertical, 24))
        .margins(48, 48, 48, 48)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let error_title = Label::new(Some(title));
    error_title.add_css_class("title-1");
    error_title.set_halign(gtk4::Align::Center);

    let error_message = Label::new(Some(message));
    error_message.set_wrap(true);
    error_message.set_max_width_chars(80);
    error_message.set_halign(gtk4::Align::Center);
    error_message.add_css_class("dim-label");
    error_message.set_justify(gtk4::Justification::Center);

    main_box.append(&error_title);
    main_box.append(&error_message);

    content_area.append(&main_box);
}

// TODO move to filesystem
/// Reads available vaults from the vaults directory
pub fn get_available_accounts(vault_name: &str) -> Vec<String> {
    let account_dir = get_account_directory(vault_name);
    read_directory(&account_dir).unwrap_or_else(|_| {
        eprintln!(
            "Failed to read account directory: {}",
            account_dir.display()
        );
        Vec::new()
    })
}

/// Gets the account directory path
fn get_account_directory(vault_name: &str) -> PathBuf {
    let locations = Locations::new(vault_name, "");
    locations.account
}

pub struct CreateActionRow<F: Fn() + 'static> {
    title: Option<String>,
    subtitle: Option<String>,
    button_label: Option<String>,
    css_class: Option<String>,
    callback: Option<F>,
    activatable: bool,
    margin_start: i32,
    margin_end: i32,
    add_button: bool,
    suffix_widget: Option<gtk4::Widget>,
}

impl<F: Fn() + 'static> Default for CreateActionRow<F> {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            button_label: None,
            css_class: None,
            activatable: true,
            margin_start: 8,
            margin_end: 8,
            callback: None,
            add_button: true,
            suffix_widget: None,
        }
    }
}

impl<F: Fn() + 'static> CreateActionRow<F> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    pub fn button_label(mut self, bl: impl Into<String>) -> Self {
        self.button_label = Some(bl.into());
        self
    }
    pub fn css_class(mut self, cc: impl Into<String>) -> Self {
        self.css_class = Some(cc.into());
        self
    }
    pub fn activatable(mut self, a: bool) -> Self {
        self.activatable = a;
        self
    }
    /*pub fn margin_start(mut self, m: i32) -> Self {
        self.margin_start = m;
        self
    }
    pub fn margin_end(mut self, m: i32) -> Self {
        self.margin_end = m;
        self
    }*/
    pub fn callback(mut self, c: F) -> Self {
        self.callback = Some(c);
        self
    }
    pub fn add_button(mut self, ab: bool) -> Self {
        self.add_button = ab;
        self
    }
    pub fn suffix(mut self, widget: &impl gtk4::prelude::IsA<gtk4::Widget>) -> Self {
        self.suffix_widget = Some(widget.clone().upcast());
        self
    }

    pub fn build(self) -> ActionRow {
        let row = ActionRow::new();
        let button = Button::new();

        if let Some(t) = self.title {
            row.set_title(&t);
        }
        if let Some(s) = self.subtitle {
            row.set_subtitle(&s);
        }
        row.set_activatable(self.activatable);
        row.set_margin_start(self.margin_start);
        row.set_margin_end(self.margin_end);

        if let Some(custom_widget) = self.suffix_widget {
            row.add_suffix(&custom_widget);
        }

        if self.add_button {
            if let Some(bl) = self.button_label {
                button.set_label(&bl)
            }
            if let Some(cc) = self.css_class {
                button.add_css_class(&cc)
            }

            button.set_valign(gtk4::Align::Center);
            row.add_suffix(&button);
            row.set_activatable_widget(Some(&button));

            if let Some(c) = self.callback {
                button.connect_clicked(move |_| c());
            }
        }

        row
    }
}

pub struct CreateBox {
    new_box: Box,
    top: i32,
    bottom: i32,
    start: i32,
    end: i32,
    halign: Option<Align>,
    valign: Option<Align>,
}

impl Default for CreateBox {
    fn default() -> Self {
        Self {
            new_box: Box::new(Orientation::Vertical, 0),
            top: 24,
            bottom: 24,
            start: 24,
            end: 24,
            halign: None,
            valign: None,
        }
    }
}

impl CreateBox {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_box(mut self, nb: Box) -> Self {
        self.new_box = nb;
        self
    }
    pub fn margins(mut self, top: i32, bottom: i32, start: i32, end: i32) -> Self {
        self.top = top;
        self.bottom = bottom;
        self.start = start;
        self.end = end;
        self
    }
    pub fn halign(mut self, h: impl Into<Align>) -> Self {
        self.halign = Some(h.into());
        self
    }
    pub fn valign(mut self, v: impl Into<Align>) -> Self {
        self.valign = Some(v.into());
        self
    }

    pub fn build(self) -> Box {
        let content_box = self.new_box;

        content_box.set_margin_top(self.top);
        content_box.set_margin_bottom(self.bottom);
        content_box.set_margin_start(self.start);
        content_box.set_margin_end(self.end);

        if let Some(h) = self.halign {
            content_box.set_halign(h)
        }
        if let Some(v) = self.valign {
            content_box.set_valign(v)
        }

        content_box
    }
}

pub struct CreateScrollableView {
    max_width: i32,
    tighten_threshold: i32,
}

impl Default for CreateScrollableView {
    fn default() -> Self {
        Self {
            max_width: 800,
            tighten_threshold: 600,
        }
    }
}

impl CreateScrollableView {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn max_width(mut self, width: i32) -> Self {
        self.max_width = width;
        self
    }
    pub fn tighten_threshold(mut self, threshold: i32) -> Self {
        self.tighten_threshold = threshold;
        self
    }

    pub fn build(self) -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let clamp = Clamp::new();
        clamp.set_maximum_size(self.max_width);
        clamp.set_tightening_threshold(self.tighten_threshold);

        scrolled.set_child(Some(&clamp));
        scrolled
    }
}
