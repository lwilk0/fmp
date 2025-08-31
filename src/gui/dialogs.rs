use adw::prelude::*;

use crate::password::{PasswordConfig, generate_password};
use crate::vault::Account;
use gtk4::{
    Adjustment, Box, Button, CheckButton, Dialog, Entry, Label, Orientation, PolicyType,
    ScrolledWindow, SpinButton, TextView,
};
use std::cell::RefCell;
use std::rc::Rc;
/// Shows the password generator dialog and updates the provided entry field and account
pub fn show_password_generator_dialog(entry: &Entry, account_rc: &Rc<RefCell<Account>>) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Password Generator"));
    dialog.set_modal(true);
    dialog.set_default_size(500, 600);

    // Create main content box
    let content_box = Box::new(Orientation::Vertical, 16);
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
fn create_length_section(config: &Rc<RefCell<PasswordConfig>>) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Password Length"));
    title.add_css_class("title-4");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    let length_box = Box::new(Orientation::Horizontal, 12);

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
fn create_character_types_section(config: &Rc<RefCell<PasswordConfig>>) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

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
fn create_custom_characters_section(config: &Rc<RefCell<PasswordConfig>>) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

    let title = Label::new(Some("Custom Characters"));
    title.add_css_class("title-4");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    // Additional characters
    let additional_box = Box::new(Orientation::Vertical, 4);
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
    let excluded_box = Box::new(Orientation::Vertical, 4);
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
) -> Box {
    let section = Box::new(Orientation::Vertical, 8);

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
    let button_box = Box::new(Orientation::Horizontal, 8);
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
