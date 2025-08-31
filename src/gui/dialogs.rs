use adw::prelude::*;

use crate::password::{PasswordConfig, generate_password};
use crate::totp::{disable_totp, enable_totp, get_totp_qr_info, is_totp_enabled, verify_totp_code};
use crate::vault::Account;
use gtk4::Box as GtkBox;
use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::glib::Bytes;
use gtk4::{
    Adjustment, Button, CheckButton, Dialog, Entry, Image, Label, Orientation, PolicyType,
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
