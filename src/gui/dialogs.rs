use adw::prelude::*;

use crate::password::{PasswordConfig, generate_password};
use crate::storage::filesystem::{
    backup_exists, create_backup, delete_backup, delete_vault, install_backup, list_backups,
    rename_account, rename_vault,
};
use crate::totp::{
    confirm_totp_setup, disable_totp, enable_totp, get_totp_qr_info, is_totp_enabled,
    prepare_totp_setup, verify_totp_code, verify_totp_code_with_secret,
};
use crate::vault::Account;
use gtk4::Box as GtkBox;
use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::glib::{self, Bytes};
use gtk4::{
    Adjustment, Button, CheckButton, Dialog, Entry, Frame, Image, Label, Orientation, PolicyType,
    ResponseType, ScrolledWindow, SpinButton, TextView,
};
use image::{DynamicImage, ImageBuffer, Luma};
use qrcode::QrCode;
use std::cell::RefCell;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use std::rc::Rc;
/// Shows the password generator dialog and updates the provided entry field and account
pub fn show_password_generator_dialog(entry: &Entry, account_rc: &Rc<RefCell<Account>>) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Password Generator"));
    dialog.set_modal(true);
    dialog.set_default_size(500, 600);

    // Create main content box
    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    // Title
    let title = Label::new(Some("Generate Secure Password"));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    // Password configuration
    let config = Rc::new(RefCell::new(PasswordConfig::default()));

    // Length section
    let length_section = create_length_section(&config);
    content_box.append(&length_section);

    // Character types section
    let char_types_section = create_character_types_section(&config);
    content_box.append(&char_types_section);

    // Additional/Excluded characters section
    let custom_chars_section = create_custom_characters_section(&config);
    content_box.append(&custom_chars_section);

    // Generated password display
    let password_display_section =
        create_password_display_section(&config, Some(entry), Some(account_rc), Some(&dialog));
    content_box.append(&password_display_section);

    dialog.set_child(Some(&content_box));

    dialog.present();
}

/// Creates the password length configuration section
fn create_length_section(config: &Rc<RefCell<PasswordConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Password Length"));
    title.add_css_class("title-4");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    let length_box = GtkBox::new(Orientation::Horizontal, 12);

    // Spin button for length
    let adjustment = Adjustment::new(16.0, 1.0, 128.0, 1.0, 5.0, 0.0);
    let spin_button = SpinButton::new(Some(&adjustment), 1.0, 0);
    spin_button.set_value(16.0);

    let config_clone = config.clone();
    spin_button.connect_value_changed(move |spin| {
        let mut config = config_clone.borrow_mut();
        config.length = spin.value() as usize;
    });

    let length_label = Label::new(Some("characters"));
    length_label.add_css_class("dim-label");

    length_box.append(&spin_button);
    length_box.append(&length_label);
    section.append(&length_box);

    section
}

/// Creates the character types configuration section
fn create_character_types_section(config: &Rc<RefCell<PasswordConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Character Types"));
    title.add_css_class("title-4");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    // Create checkboxes for each character type
    let checkboxes = vec![
        ("Lowercase letters (a-z)", "include_lowercase", true),
        ("Uppercase letters (A-Z)", "include_uppercase", true),
        ("Numbers (0-9)", "include_numbers", true),
        ("Symbols (!@#$%...)", "include_symbols", false),
        ("Spaces", "include_spaces", false),
        ("Extended characters (áéíóú...)", "include_extended", false),
    ];

    for (label_text, field_name, default_value) in checkboxes {
        let checkbox = CheckButton::new();
        checkbox.set_label(Some(label_text));
        checkbox.set_active(default_value);

        let config_clone = config.clone();
        let field_name = field_name.to_string();
        checkbox.connect_toggled(move |cb| {
            let mut config = config_clone.borrow_mut();
            let active = cb.is_active();
            match field_name.as_str() {
                "include_lowercase" => config.include_lowercase = active,
                "include_uppercase" => config.include_uppercase = active,
                "include_numbers" => config.include_numbers = active,
                "include_symbols" => config.include_symbols = active,
                "include_spaces" => config.include_spaces = active,
                "include_extended" => config.include_extended = active,
                _ => {}
            }
        });

        section.append(&checkbox);
    }

    section
}

/// Creates the custom characters configuration section
fn create_custom_characters_section(config: &Rc<RefCell<PasswordConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Custom Characters"));
    title.add_css_class("title-4");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    // Additional characters
    let additional_box = GtkBox::new(Orientation::Vertical, 4);
    let additional_label = Label::new(Some("Additional characters to include:"));
    additional_label.add_css_class("dim-label");
    additional_label.set_halign(gtk4::Align::Start);

    let additional_entry = Entry::new();
    additional_entry.set_placeholder_text(Some("e.g., @#$"));

    let config_clone = config.clone();
    additional_entry.connect_changed(move |entry| {
        let mut config = config_clone.borrow_mut();
        config.additional_characters = entry.text().to_string();
    });

    additional_box.append(&additional_label);
    additional_box.append(&additional_entry);
    section.append(&additional_box);

    // Excluded characters
    let excluded_box = GtkBox::new(Orientation::Vertical, 4);
    let excluded_label = Label::new(Some("Characters to exclude:"));
    excluded_label.add_css_class("dim-label");
    excluded_label.set_halign(gtk4::Align::Start);

    let excluded_entry = Entry::new();
    excluded_entry.set_placeholder_text(Some("e.g., 0O1l"));

    let config_clone = config.clone();
    excluded_entry.connect_changed(move |entry| {
        let mut config = config_clone.borrow_mut();
        config.excluded_characters = entry.text().to_string();
    });

    excluded_box.append(&excluded_label);
    excluded_box.append(&excluded_entry);
    section.append(&excluded_box);

    section
}

/// Creates the password display section
fn create_password_display_section(
    config: &Rc<RefCell<PasswordConfig>>,
    entry: Option<&Entry>,
    account_rc: Option<&Rc<RefCell<Account>>>,
    dialog: Option<&Dialog>,
) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Generated Password"));
    title.add_css_class("title-4");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    // Text view for password display
    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_wrap_mode(gtk4::WrapMode::Char);
    text_view.set_size_request(-1, 80);
    text_view.add_css_class("password-display");

    // Set monospace font
    let buffer = text_view.buffer();
    buffer.set_text("Click 'Generate Password' to create a password");

    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    scrolled.set_child(Some(&text_view));
    scrolled.set_size_request(-1, 100);

    section.append(&scrolled);

    // Button container
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.set_halign(gtk4::Align::Center);

    // Generate button
    let generate_button = Button::new();
    generate_button.set_label("Generate Password");
    generate_button.add_css_class("suggested-action");

    let text_view_clone = text_view.clone();
    let config_clone = config.clone();
    generate_button.connect_clicked(move |_| {
        let config = config_clone.borrow();
        match generate_password(&*config) {
            Ok(password) => {
                let buffer = text_view_clone.buffer();
                buffer.set_text(&password);
            }
            Err(e) => {
                eprintln!("Failed to generate password: {}", e);
                let buffer = text_view_clone.buffer();
                buffer.set_text("Error generating password");
            }
        }
    });

    button_box.append(&generate_button);

    // Add Use and Cancel buttons only if we have the necessary parameters
    if let (Some(entry), Some(account_rc), Some(dialog)) = (entry, account_rc, dialog) {
        // Use button
        let use_button = Button::new();
        use_button.set_label("Use");
        use_button.add_css_class("flat");

        let text_view_use = text_view.clone();
        let entry_clone = entry.clone();
        let account_rc_clone = account_rc.clone();
        let dialog_clone = dialog.clone();
        use_button.connect_clicked(move |_| {
            let buffer = text_view_use.buffer();
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let password = buffer.text(&start, &end, false);

            if !password.is_empty() && password != "Click 'Generate Password' to create a password"
            {
                entry_clone.set_text(&password);
                let mut account = account_rc_clone.borrow_mut();
                account.password.update(password.to_string());
                dialog_clone.close();
            }
        });

        // Cancel button
        let cancel_button = Button::new();
        cancel_button.set_label("Cancel");
        cancel_button.add_css_class("flat");

        let dialog_cancel = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_cancel.close();
        });

        button_box.append(&use_button);
        button_box.append(&cancel_button);
    }

    section.append(&button_box);
    section
}

/// Checks if this is the first run of the application
pub fn is_first_run() -> bool {
    let config_path = get_config_file_path();
    !config_path.exists()
}

/// Creates the first-run marker file
pub fn mark_first_run_complete() -> Result<(), std::io::Error> {
    let config_path = get_config_file_path();

    // Create the config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        create_dir_all(parent)?;
    }

    // Create the marker file
    File::create(&config_path)?;

    Ok(())
}

/// Gets the path to the first-run marker file
fn get_config_file_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("fmp").join("fmp-ran")
}

/// Shows the welcome dialog for first-time users
pub fn show_welcome_dialog() {
    let dialog = Dialog::new();
    dialog.set_title(Some("Welcome to Forgot My Password"));
    dialog.set_modal(true);
    dialog.set_default_size(650, 550);
    dialog.add_css_class("welcome-dialog");

    // Create main content box
    let content_box = GtkBox::new(Orientation::Vertical, 24);
    content_box.set_margin_top(32);
    content_box.set_margin_bottom(32);
    content_box.set_margin_start(32);
    content_box.set_margin_end(32);

    // Welcome title with icon
    let title_box = GtkBox::new(Orientation::Horizontal, 12);
    title_box.set_halign(gtk4::Align::Center);

    let icon = Label::new(Some("🔐"));
    icon.add_css_class("title-1");

    let title = Label::new(Some("Welcome to Forgot My Password!"));
    title.add_css_class("title-1");

    title_box.append(&icon);
    title_box.append(&title);
    content_box.append(&title_box);

    // Welcome message
    let welcome_text = "Thank you for choosing Forgot My Password (FMP) - your secure password manager.\n\nFMP helps you:\n• Store passwords securely with GPG encryption\n• Generate strong, unique passwords\n• Manage TOTP codes for two-factor authentication\n• Keep your sensitive data safe and organized\n\nTo get started:\n1. Create your first vault to store passwords\n2. Add accounts with their login credentials\n3. Use the password generator for strong passwords\n4. Enable TOTP for accounts that support it\n\nYour data is encrypted and stored locally for maximum security.";

    let message_label = Label::new(Some(welcome_text));
    message_label.set_wrap(true);
    message_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    message_label.set_justify(gtk4::Justification::Left);
    message_label.set_halign(gtk4::Align::Fill);
    message_label.set_valign(gtk4::Align::Start);
    message_label.add_css_class("body");

    // Create scrolled window for the message
    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled.set_child(Some(&message_label));
    scrolled.set_size_request(-1, 320);
    scrolled.add_css_class("card");
    scrolled.set_margin_top(8);
    scrolled.set_margin_bottom(8);

    content_box.append(&scrolled);

    // Button box
    let button_box = GtkBox::new(Orientation::Horizontal, 16);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(8);

    // Learn More about GPG button
    let learn_more_button = Button::new();
    learn_more_button.set_label("Learn More about GPG");
    learn_more_button.add_css_class("flat");
    learn_more_button.set_size_request(180, -1);

    learn_more_button.connect_clicked(move |_| {
        show_gpg_info_dialog();
    });

    // Get Started button
    let get_started_button = Button::new();
    get_started_button.set_label("Get Started");
    get_started_button.add_css_class("suggested-action");
    get_started_button.add_css_class("pill");
    get_started_button.set_size_request(140, -1);

    let dialog_clone = dialog.clone();
    get_started_button.connect_clicked(move |_| {
        // Mark first run as complete
        if let Err(e) = mark_first_run_complete() {
            eprintln!("Failed to mark first run complete: {}", e);
        }
        dialog_clone.close();
    });

    button_box.append(&learn_more_button);
    button_box.append(&get_started_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the GPG information dialog with setup instructions
fn show_gpg_info_dialog() {
    let dialog = Dialog::new();
    dialog.set_title(Some("GPG Setup Information"));
    dialog.set_modal(true);
    dialog.set_default_size(550, 400);
    dialog.add_css_class("info-dialog");

    // Create main content box
    let content_box = GtkBox::new(Orientation::Vertical, 20);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    // Title with icon
    let title_box = GtkBox::new(Orientation::Horizontal, 12);
    title_box.set_halign(gtk4::Align::Center);

    let icon = Label::new(Some("🔑"));
    icon.add_css_class("title-2");

    let title = Label::new(Some("Setting up GPG"));
    title.add_css_class("title-2");

    title_box.append(&icon);
    title_box.append(&title);
    content_box.append(&title_box);

    // GPG instructions with better formatting
    let instructions_box = GtkBox::new(Orientation::Vertical, 16);

    let intro_text =
        "You'll need to install GPG (GNU Privacy Guard) if it's not already on your system.";
    let intro_label = Label::new(Some(intro_text));
    intro_label.set_wrap(true);
    intro_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    intro_label.set_justify(gtk4::Justification::Left);
    intro_label.set_halign(gtk4::Align::Start);
    intro_label.add_css_class("body");

    let commands_title = Label::new(Some("Common GPG Commands:"));
    commands_title.add_css_class("title-4");
    commands_title.set_halign(gtk4::Align::Start);
    commands_title.set_margin_top(8);

    let commands_text = "• Generate a new key: `gpg --full-generate-key`\n• Use an existing key from your keyring\n• Import a key: `gpg --import your-key.asc`\n• List keys: `gpg --list-keys`";
    let commands_label = Label::new(Some(commands_text));
    commands_label.set_wrap(true);
    commands_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    commands_label.set_justify(gtk4::Justification::Left);
    commands_label.set_halign(gtk4::Align::Start);
    commands_label.add_css_class("body");
    commands_label.add_css_class("monospace");

    instructions_box.append(&intro_label);
    instructions_box.append(&commands_title);
    instructions_box.append(&commands_label);

    // Create scrolled window for the instructions
    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled.set_child(Some(&instructions_box));
    scrolled.set_size_request(-1, 200);
    scrolled.add_css_class("card");
    scrolled.set_margin_top(8);
    scrolled.set_margin_bottom(8);

    content_box.append(&scrolled);

    // Button box
    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);

    // Close button
    let close_button = Button::new();
    close_button.set_label("Close");
    close_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    close_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    button_box.append(&close_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows a confirmation dialog for dangerous actions
/// Takes a callback that will be executed if the user confirms the action
pub fn show_confirmation_dialog<F>(
    title: &str,
    message: &str,
    confirm_label: &str,
    parent: Option<&impl IsA<gtk4::Window>>,
    on_confirm: F,
) where
    F: Fn() + 'static,
{
    let dialog = Dialog::new();
    dialog.set_title(Some(title));
    dialog.set_modal(true);
    dialog.set_default_size(450, 250);
    dialog.add_css_class("confirmation-dialog");

    if let Some(parent_window) = parent {
        dialog.set_transient_for(Some(parent_window));
    }

    // Create main content box
    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    // Warning icon and message
    let message_box = GtkBox::new(Orientation::Horizontal, 12);
    message_box.set_halign(gtk4::Align::Center);

    // Warning icon
    let icon = Label::new(Some("⚠️"));
    icon.add_css_class("title-2");
    message_box.append(&icon);

    // Message text
    let message_label = Label::new(Some(message));
    message_label.set_wrap(true);
    message_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    message_label.set_justify(gtk4::Justification::Center);
    message_label.add_css_class("body");
    message_box.append(&message_label);

    content_box.append(&message_box);

    // Button box
    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);

    // Cancel button
    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    // Confirm button
    let confirm_button = Button::new();
    confirm_button.set_label(confirm_label);
    confirm_button.add_css_class("destructive-action");

    let dialog_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_cancel.close();
    });

    let dialog_confirm = dialog.clone();
    confirm_button.connect_clicked(move |_| {
        on_confirm();
        dialog_confirm.close();
    });

    button_box.append(&cancel_button);
    button_box.append(&confirm_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the TOTP setup dialog for enabling 2FA on a vault
pub fn show_totp_setup_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Enable Two-Factor Authentication"));
    dialog.set_modal(true);
    dialog.set_default_size(600, 650);
    dialog.add_css_class("totp-dialog");

    // Create main content box
    let content_box = GtkBox::new(Orientation::Vertical, 6);
    content_box.set_margin_top(8);
    content_box.set_margin_bottom(8);
    content_box.set_margin_start(16);
    content_box.set_margin_end(16);

    // Title with icon
    let title_container = GtkBox::new(Orientation::Vertical, 4);
    title_container.set_halign(gtk4::Align::Center);
    title_container.add_css_class("dialog-title-container");

    let icon = Label::new(Some("🔐"));
    icon.add_css_class("dialog-icon");
    icon.set_halign(gtk4::Align::Center);

    let title = Label::new(Some(&format!("Enable 2FA for \"{}\"", vault_name)));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Center);

    let subtitle = Label::new(Some("Secure your vault with two-factor authentication"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Center);

    title_container.append(&icon);
    title_container.append(&title);
    title_container.append(&subtitle);
    content_box.append(&title_container);

    // Instructions
    let instructions = Label::new(Some(
        "Scan the QR code below with your authenticator app (like Google Authenticator, Authy, or 1Password), then enter the 6-digit code to verify setup.",
    ));
    instructions.set_wrap(true);
    instructions.set_wrap_mode(gtk4::pango::WrapMode::Word);
    instructions.set_justify(gtk4::Justification::Center);
    instructions.add_css_class("body");
    instructions.set_margin_bottom(16);
    content_box.append(&instructions);

    // Show loading message initially
    let loading_label = Label::new(Some("Generating QR code..."));
    loading_label.add_css_class("dim-label");
    loading_label.set_halign(gtk4::Align::Center);
    loading_label.set_margin_top(20);
    loading_label.set_margin_bottom(20);
    content_box.append(&loading_label);

    dialog.set_child(Some(&content_box));
    dialog.present();

    // Prepare TOTP secret and QR code after showing dialog (without enabling yet)
    match prepare_totp_setup(vault_name) {
        Ok((secret, secret_b32, otpauth_uri)) => {
            // Remove loading message
            content_box.remove(&loading_label);

            // QR Code section
            let qr_section = create_qr_code_section(&otpauth_uri, &secret_b32);
            content_box.append(&qr_section);

            // Verification section with the prepared secret
            let verification_section = create_totp_verification_section_with_secret(
                vault_name,
                &secret,
                &dialog,
                content_area,
            );
            content_box.append(&verification_section);
        }
        Err(e) => {
            // Remove loading message
            content_box.remove(&loading_label);

            let error_label = Label::new(Some(&format!("Failed to prepare 2FA: {}", e)));
            error_label.add_css_class("error");
            error_label.set_wrap(true);
            content_box.append(&error_label);

            // Close button for error case
            let close_button = Button::new();
            close_button.set_label("Close");
            close_button.add_css_class("suggested-action");
            close_button.set_halign(gtk4::Align::Center);
            close_button.set_margin_top(16);

            let dialog_clone = dialog.clone();
            close_button.connect_clicked(move |_| {
                dialog_clone.close();
            });

            content_box.append(&close_button);
        }
    }
}

/// Shows the TOTP management dialog for an already enabled vault
pub fn show_totp_management_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Manage Two-Factor Authentication"));
    dialog.set_modal(true);
    dialog.set_default_size(600, 650);
    dialog.add_css_class("totp-dialog");

    // Create main content box
    let content_box = GtkBox::new(Orientation::Vertical, 20);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    // Title with icon
    let title_container = GtkBox::new(Orientation::Vertical, 8);
    title_container.set_halign(gtk4::Align::Center);
    title_container.add_css_class("dialog-title-container");

    let icon = Label::new(Some("🔐"));
    icon.add_css_class("dialog-icon");
    icon.set_halign(gtk4::Align::Center);

    let title = Label::new(Some(&format!("2FA Settings for \"{}\"", vault_name)));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Center);

    let subtitle = Label::new(Some("Manage your two-factor authentication settings"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Center);

    title_container.append(&icon);
    title_container.append(&title);
    title_container.append(&subtitle);
    content_box.append(&title_container);

    // Status message
    let status_label = Label::new(Some(
        "Two-factor authentication is currently enabled for this vault.",
    ));
    status_label.add_css_class("success");
    status_label.set_halign(gtk4::Align::Center);
    content_box.append(&status_label);

    // Get existing TOTP info
    match get_totp_qr_info(vault_name) {
        Ok((secret, otpauth_uri)) => {
            // QR Code section (for backup/re-setup)
            let qr_section = create_qr_code_section(&otpauth_uri, &secret);
            content_box.append(&qr_section);

            // Action buttons
            let button_box = GtkBox::new(Orientation::Horizontal, 12);
            button_box.set_halign(gtk4::Align::Center);
            button_box.set_margin_top(16);

            // Disable 2FA button
            let disable_button = Button::new();
            disable_button.set_label("Disable 2FA");
            disable_button.add_css_class("destructive-action");

            let vault_name_clone = vault_name.to_string();
            let content_area_clone = content_area.clone();
            let dialog_clone = dialog.clone();
            disable_button.connect_clicked(move |_| {
                show_confirmation_dialog(
                    "Disable Two-Factor Authentication",
                    &format!("Are you sure you want to disable 2FA for vault \"{}\"?\n\nThis will make your vault less secure.", vault_name_clone),
                    "Disable 2FA",
                    Some(&dialog_clone),
                    {
                        let vault_name_clone2 = vault_name_clone.clone();
                        let content_area_clone2 = content_area_clone.clone();
                        let dialog_clone2 = dialog_clone.clone();
                        move || {
                            match disable_totp(&vault_name_clone2) {
                                Ok(()) => {
                                    dialog_clone2.close();
                                    // Refresh the vault view to update 2FA status
                                    crate::gui::content::show_vault_view(&content_area_clone2, &vault_name_clone2);
                                }
                                Err(e) => {
                                    eprintln!("Failed to disable 2FA: {}", e);
                                }
                            }
                        }
                    }
                );
            });

            // Close button
            let close_button = Button::new();
            close_button.set_label("Close");
            close_button.add_css_class("flat");

            let dialog_close = dialog.clone();
            close_button.connect_clicked(move |_| {
                dialog_close.close();
            });

            button_box.append(&disable_button);
            button_box.append(&close_button);
            content_box.append(&button_box);
        }
        Err(e) => {
            let error_label = Label::new(Some(&format!("Failed to load 2FA info: {}", e)));
            error_label.add_css_class("error");
            error_label.set_wrap(true);
            content_box.append(&error_label);

            // Close button for error case
            let close_button = Button::new();
            close_button.set_label("Close");
            close_button.add_css_class("suggested-action");
            close_button.set_halign(gtk4::Align::Center);
            close_button.set_margin_top(16);

            let dialog_clone = dialog.clone();
            close_button.connect_clicked(move |_| {
                dialog_clone.close();
            });

            content_box.append(&close_button);
        }
    }

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Creates the QR code section with the code image and manual entry option
fn create_qr_code_section(otpauth_uri: &str, secret: &str) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 6);

    // QR Code container with frame
    let qr_container = GtkBox::new(Orientation::Vertical, 6);
    qr_container.set_halign(gtk4::Align::Center);
    qr_container.add_css_class("qr-container");

    let qr_title = Label::new(Some("Scan QR Code"));
    qr_title.add_css_class("title-3");
    qr_title.set_halign(gtk4::Align::Center);
    qr_title.set_margin_bottom(6);
    qr_container.append(&qr_title);

    // Generate and display QR code
    match generate_qr_code_image(otpauth_uri) {
        Ok(pixbuf) => {
            // Create a frame for the QR code
            let qr_frame = gtk4::Frame::new(None);
            qr_frame.add_css_class("qr-frame");
            qr_frame.set_halign(gtk4::Align::Center);
            qr_frame.set_valign(gtk4::Align::Center);

            let qr_image = Image::from_pixbuf(Some(&pixbuf));
            qr_image.add_css_class("qr-code");
            qr_image.set_margin_top(4);
            qr_image.set_margin_bottom(4);
            qr_image.set_margin_start(4);
            qr_image.set_margin_end(4);
            // Ensure the image displays at its natural size
            qr_image.set_size_request(300, 300);
            qr_image.set_halign(gtk4::Align::Center);
            qr_image.set_valign(gtk4::Align::Center);

            qr_frame.set_child(Some(&qr_image));
            qr_container.append(&qr_frame);
        }
        Err(e) => {
            let error_label = Label::new(Some(&format!("Failed to generate QR code: {}", e)));
            error_label.add_css_class("error");
            error_label.set_halign(gtk4::Align::Center);
            qr_container.append(&error_label);
        }
    }

    section.append(&qr_container);

    // Manual entry section
    let manual_section = GtkBox::new(Orientation::Vertical, 6);
    manual_section.set_margin_top(8);
    manual_section.add_css_class("manual-entry-section");

    let manual_title = Label::new(Some("Or enter manually"));
    manual_title.add_css_class("title-4");
    manual_title.set_halign(gtk4::Align::Center);
    manual_title.set_margin_bottom(6);
    manual_section.append(&manual_title);

    let secret_container = GtkBox::new(Orientation::Vertical, 6);
    secret_container.set_halign(gtk4::Align::Center);
    secret_container.set_margin_start(24);
    secret_container.set_margin_end(24);

    let secret_label = Label::new(Some("Secret Key"));
    secret_label.add_css_class("caption-heading");
    secret_label.set_halign(gtk4::Align::Start);
    secret_label.set_margin_bottom(4);

    let secret_entry = Entry::new();
    secret_entry.set_text(secret);
    secret_entry.set_editable(false);
    secret_entry.add_css_class("secret-key-entry");
    secret_entry.set_width_chars(32);

    secret_container.append(&secret_label);
    secret_container.append(&secret_entry);
    manual_section.append(&secret_container);

    section.append(&manual_section);
    section
}

/// Creates the TOTP verification section for confirming setup
fn create_totp_verification_section(
    vault_name: &str,
    dialog: &Dialog,
    content_area: &GtkBox,
) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);
    section.set_margin_top(8);
    section.add_css_class("verification-section");

    let verify_title = Label::new(Some("Verify Setup"));
    verify_title.add_css_class("title-3");
    verify_title.set_halign(gtk4::Align::Center);
    verify_title.set_margin_bottom(6);
    section.append(&verify_title);

    let verify_instructions =
        Label::new(Some("Enter the 6-digit code from your authenticator app"));
    verify_instructions.add_css_class("body");
    verify_instructions.set_halign(gtk4::Align::Center);
    verify_instructions.set_margin_bottom(12);
    section.append(&verify_instructions);

    // Code entry container
    let code_container = GtkBox::new(Orientation::Vertical, 8);
    code_container.set_halign(gtk4::Align::Center);

    let code_entry = Entry::new();
    code_entry.set_placeholder_text(Some("000000"));
    code_entry.set_max_length(8); // Allow some flexibility
    code_entry.set_width_chars(10);
    code_entry.set_halign(gtk4::Align::Center);
    code_entry.add_css_class("totp-code-entry");

    code_container.append(&code_entry);
    section.append(&code_container);

    // Status label for feedback
    let status_label = Label::new(None);
    status_label.set_halign(gtk4::Align::Center);
    section.append(&status_label);

    // Button box
    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    // Verify button
    let verify_button = Button::new();
    verify_button.set_label("Verify & Enable");
    verify_button.add_css_class("suggested-action");

    // Cancel button
    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    // Connect verify button
    let vault_name_clone = vault_name.to_string();
    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let code_entry_clone = code_entry.clone();
    let status_label_clone = status_label.clone();
    verify_button.connect_clicked(move |_| {
        let code = code_entry_clone.text();

        match verify_totp_code(&vault_name_clone, &code) {
            Ok(true) => {
                status_label_clone.set_text("✅ 2FA enabled successfully!");
                status_label_clone.add_css_class("success");

                // Close dialog after a brief delay and refresh vault view
                let dialog_clone2 = dialog_clone.clone();
                let content_area_clone2 = content_area_clone.clone();
                let vault_name_clone2 = vault_name_clone.clone();

                glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                    dialog_clone2.close();
                    // Refresh the vault view to show 2FA is now enabled
                    crate::gui::content::show_vault_view(&content_area_clone2, &vault_name_clone2);
                    glib::ControlFlow::Break
                });
            }
            Ok(false) => {
                status_label_clone.set_text("❌ Invalid code. Please try again.");
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
                code_entry_clone.select_region(0, -1); // Select all text for easy replacement
            }
            Err(e) => {
                status_label_clone.set_text(&format!("❌ Error: {}", e));
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
            }
        }
    });

    // Connect cancel button
    let dialog_cancel = dialog.clone();
    let vault_name_cancel = vault_name.to_string();
    cancel_button.connect_clicked(move |_| {
        // If user cancels, we should disable the TOTP that was just enabled
        let _ = disable_totp(&vault_name_cancel);
        dialog_cancel.close();
    });

    // Allow Enter key to trigger verification
    let verify_button_clone = verify_button.clone();
    code_entry.connect_activate(move |_| {
        verify_button_clone.emit_clicked();
    });

    button_box.append(&verify_button);
    button_box.append(&cancel_button);
    section.append(&button_box);

    section
}

/// Creates the TOTP verification section for confirming setup with a prepared secret
fn create_totp_verification_section_with_secret(
    vault_name: &str,
    secret: &[u8],
    dialog: &Dialog,
    content_area: &GtkBox,
) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);
    section.set_margin_top(8);
    section.add_css_class("verification-section");

    let verify_title = Label::new(Some("Verify Setup"));
    verify_title.add_css_class("title-3");
    verify_title.set_halign(gtk4::Align::Center);
    verify_title.set_margin_bottom(6);
    section.append(&verify_title);

    let verify_instructions =
        Label::new(Some("Enter the 6-digit code from your authenticator app"));
    verify_instructions.add_css_class("body");
    verify_instructions.set_halign(gtk4::Align::Center);
    verify_instructions.set_margin_bottom(12);
    section.append(&verify_instructions);

    // Code entry container
    let code_container = GtkBox::new(Orientation::Vertical, 8);
    code_container.set_halign(gtk4::Align::Center);

    let code_entry = Entry::new();
    code_entry.set_placeholder_text(Some("000000"));
    code_entry.set_max_length(8); // Allow some flexibility
    code_entry.set_width_chars(10);
    code_entry.set_halign(gtk4::Align::Center);
    code_entry.add_css_class("totp-code-entry");

    code_container.append(&code_entry);
    section.append(&code_container);

    // Status label for feedback
    let status_label = Label::new(None);
    status_label.set_halign(gtk4::Align::Center);
    section.append(&status_label);

    // Button box
    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    // Verify button
    let verify_button = Button::new();
    verify_button.set_label("Verify & Enable");
    verify_button.add_css_class("suggested-action");

    // Cancel button
    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    // Connect verify button
    let vault_name_clone = vault_name.to_string();
    let dialog_clone = dialog.clone();
    let content_area_clone = content_area.clone();
    let code_entry_clone = code_entry.clone();
    let status_label_clone = status_label.clone();
    let secret_clone = secret.to_vec();
    verify_button.connect_clicked(move |_| {
        let code = code_entry_clone.text();

        match verify_totp_code_with_secret(&secret_clone, &code) {
            Ok(true) => {
                // Code is valid, now actually enable TOTP
                match confirm_totp_setup(&vault_name_clone, &secret_clone) {
                    Ok(()) => {
                        status_label_clone.set_text("✅ 2FA enabled successfully!");
                        status_label_clone.add_css_class("success");

                        // Close dialog after a brief delay and refresh vault view
                        let dialog_clone2 = dialog_clone.clone();
                        let content_area_clone2 = content_area_clone.clone();
                        let vault_name_clone2 = vault_name_clone.clone();

                        glib::timeout_add_local(
                            std::time::Duration::from_millis(1500),
                            move || {
                                dialog_clone2.close();
                                // Refresh the vault view to show 2FA is now enabled
                                crate::gui::content::show_vault_view(
                                    &content_area_clone2,
                                    &vault_name_clone2,
                                );
                                glib::ControlFlow::Break
                            },
                        );
                    }
                    Err(e) => {
                        status_label_clone.set_text(&format!("❌ Failed to enable 2FA: {}", e));
                        status_label_clone.remove_css_class("success");
                        status_label_clone.add_css_class("error");
                    }
                }
            }
            Ok(false) => {
                status_label_clone.set_text("❌ Invalid code. Please try again.");
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
                code_entry_clone.select_region(0, -1); // Select all text for easy replacement
            }
            Err(e) => {
                status_label_clone.set_text(&format!("❌ Error: {}", e));
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
            }
        }
    });

    // Connect cancel button - no cleanup needed since TOTP wasn't enabled yet
    let dialog_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_cancel.close();
    });

    // Allow Enter key to trigger verification
    let verify_button_clone = verify_button.clone();
    code_entry.connect_activate(move |_| {
        verify_button_clone.emit_clicked();
    });

    button_box.append(&verify_button);
    button_box.append(&cancel_button);
    section.append(&button_box);

    section
}

/// Generates a QR code image from an otpauth URI
fn generate_qr_code_image(otpauth_uri: &str) -> Result<Pixbuf, Box<dyn std::error::Error>> {
    // Generate QR code with larger module size for better performance and visibility
    let qr_code = QrCode::new(otpauth_uri)?;

    // Use the QR code renderer's built-in scaling for much better performance
    let image = qr_code
        .render::<Luma<u8>>()
        .min_dimensions(300, 300) // Set minimum size to 300x300 for good visibility
        .module_dimensions(15, 15) // Each module is 15x15 pixels (very large and clear)
        .build();

    // Convert to RGB for GTK
    let rgb_image = DynamicImage::ImageLuma8(image).to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let data = rgb_image.into_raw();

    // Create GTK Pixbuf
    let bytes = Bytes::from(&data);
    let pixbuf = Pixbuf::from_bytes(
        &bytes,
        Colorspace::Rgb,
        false, // has_alpha
        8,     // bits_per_sample
        width as i32,
        height as i32,
        (width * 3) as i32, // rowstride (3 bytes per pixel for RGB)
    );

    Ok(pixbuf)
}

/// Shows the backup vault dialog
pub fn show_backup_vault_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Create Vault Backup"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Create Backup for '{}'", vault_name)));
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
    backup_button.connect_clicked(move |_| {
        match create_backup(&vault_name_clone) {
            Ok(()) => {
                dialog_clone.close();
                // Refresh the vault view to update backup status
                crate::gui::content::show_vault_view(&content_area_clone, &vault_name_clone);
            }
            Err(e) => {
                eprintln!("Failed to create backup: {}", e);
            }
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&backup_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the restore vault dialog
pub fn show_restore_vault_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Restore Vault Backup"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 250);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Restore Backup for '{}'", vault_name)));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let warning = Label::new(Some(
        "⚠️ This will replace all current vault data with the backup.\nThis action cannot be undone.",
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
                crate::gui::content::show_vault_view(&content_area_clone, &vault_name_clone);
            }
            Err(e) => {
                eprintln!("Failed to restore backup: {}", e);
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
pub fn show_delete_backup_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Delete Vault Backup"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Delete Backup for '{}'", vault_name)));
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
    delete_button.connect_clicked(move |_| {
        match delete_backup(&vault_name_clone) {
            Ok(()) => {
                dialog_clone.close();
                // Refresh the vault view to update backup status
                crate::gui::content::show_vault_view(&content_area_clone, &vault_name_clone);
            }
            Err(e) => {
                eprintln!("Failed to delete backup: {}", e);
            }
        }
    });

    button_box.append(&cancel_button);
    button_box.append(&delete_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the rename vault dialog
pub fn show_rename_vault_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Rename Vault"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Rename Vault '{}'", vault_name)));
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
                    // Navigate to the renamed vault
                    crate::gui::content::show_vault_view(&content_area_clone, &new_name);
                    // TODO: Refresh sidebar to show new name
                }
                Err(e) => {
                    eprintln!("Failed to rename vault: {}", e);
                }
            }
        }
    });

    // Allow Enter key to trigger rename
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
pub fn show_delete_vault_dialog(vault_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Delete Vault"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 250);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Delete Vault '{}'", vault_name)));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let warning = Label::new(Some(
        "⚠️ This will permanently delete the vault and all its data.\nThis action cannot be undone.",
    ));
    warning.add_css_class("error");
    warning.set_wrap(true);
    warning.set_halign(gtk4::Align::Center);
    content_box.append(&warning);

    let confirmation_label =
        Label::new(Some(&format!("Type '{}' to confirm deletion:", vault_name)));
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
    delete_button.set_sensitive(false); // Initially disabled

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    // Enable delete button only when correct name is typed
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
                    // Navigate back to home view
                    crate::gui::content::show_home_view(&content_area_clone);
                    // TODO: Refresh sidebar to remove deleted vault
                }
                Err(e) => {
                    eprintln!("Failed to delete vault: {}", e);
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

/// Shows the rename account dialog
pub fn show_rename_account_dialog(vault_name: &str, account_name: &str, content_area: &GtkBox) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Rename Account"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Rename Account '{}'", account_name)));
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
                    // Navigate to the renamed account
                    crate::gui::content::show_account_view(
                        &content_area_clone,
                        &vault_name_clone,
                        &new_name,
                    );
                }
                Err(e) => {
                    eprintln!("Failed to rename account: {}", e);
                }
            }
        }
    });

    // Allow Enter key to trigger rename
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
