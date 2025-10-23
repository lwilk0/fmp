use crate::{
    password::{
        PasswordConfig, calculate_password_strength, generate_password, get_strength_color_class,
        get_strength_description,
    },
    storage::filesystem::{
        create_backup, delete_backup, delete_vault, install_backup, rename_account, rename_vault,
    },
    totp::{
        confirm_totp_setup, disable_totp, get_totp_qr_info, prepare_totp_setup, verify_totp_code,
        verify_totp_code_with_secret,
    },
    vault::Account,
};

use adw::{
    ActionRow, ButtonContent, Clamp, HeaderBar, PreferencesGroup, PreferencesWindow,
    Window as AdwWindow, prelude::*,
};
use gtk4::{
    Adjustment, Box as GtkBox, Button, ButtonsType, Dialog, Entry, Image, Label, MessageDialog,
    MessageType, Orientation, PolicyType, ProgressBar, ResponseType, ScrolledWindow, SpinButton,
    Switch, TextView, gdk,
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::{self, Bytes},
};
use image::{DynamicImage, Luma};
use qrcode::QrCode;
use std::{
    cell::RefCell,
    fs::{File, create_dir_all},
    path::PathBuf,
    rc::Rc,
};

/// Shows the password generator dialog and updates the provided entry field and account
pub fn show_password_generator_dialog(target_entry: &Entry, account_ref: &Rc<RefCell<Account>>) {
    let generator_window = PreferencesWindow::new();
    generator_window.set_title(Some("Password Generator"));
    generator_window.set_modal(true);
    generator_window.set_default_size(560, 640);
    generator_window.set_search_enabled(false);

    // Password configuration - use single shared instance
    let password_config = Rc::new(RefCell::new(PasswordConfig::default()));

    let page = adw::PreferencesPage::new();
    page.set_title("Password Generator");
    page.set_icon_name(Some("dialog-password-symbolic"));

    let length_group = create_password_length_preferences_group(&password_config);
    let character_group = create_character_types_preferences_group(&password_config);
    let custom_group = create_custom_characters_preferences_group(&password_config);
    let display_group = create_password_display_preferences_group(
        &password_config,
        Some(target_entry),
        Some(account_ref),
        Some(&generator_window),
    );

    page.add(&length_group);
    page.add(&character_group);
    page.add(&custom_group);
    page.add(&display_group);

    generator_window.add(&page);
    generator_window.present();
}

/// Shows the password generator dialog without the "Use Password" button (for standalone use)
pub fn show_standalone_password_generator_dialog() {
    let generator_window = PreferencesWindow::new();
    generator_window.set_title(Some("Password Generator"));
    generator_window.set_modal(true);
    generator_window.set_default_size(560, 640);
    generator_window.set_search_enabled(false);

    // Password configuration - use single shared instance
    let password_config = Rc::new(RefCell::new(PasswordConfig::default()));

    let page = adw::PreferencesPage::new();
    page.set_title("Password Generator");
    page.set_icon_name(Some("dialog-password-symbolic"));

    let length_group = create_password_length_preferences_group(&password_config);
    let character_group = create_character_types_preferences_group(&password_config);
    let custom_group = create_custom_characters_preferences_group(&password_config);
    let display_group = create_password_display_preferences_group(
        &password_config,
        None,
        None,
        Some(&generator_window),
    );

    page.add(&length_group);
    page.add(&character_group);
    page.add(&custom_group);
    page.add(&display_group);

    generator_window.add(&page);
    generator_window.present();
}

/// Creates the password length configuration preferences group
fn create_password_length_preferences_group(
    password_config: &Rc<RefCell<PasswordConfig>>,
) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Password Length");
    group.set_description(Some("Configure the length of generated passwords"));

    let length_row = ActionRow::new();
    length_row.set_title("Length");
    length_row.set_subtitle("Number of characters in the password");

    let length_adjustment = Adjustment::new(16.0, 1.0, 128.0, 1.0, 5.0, 0.0);
    let length_spinner = SpinButton::new(Some(&length_adjustment), 1.0, 0);
    length_spinner.set_value(16.0);
    length_spinner.set_tooltip_text(Some("Set the desired password length (1-128 characters)"));
    length_spinner.set_valign(gtk4::Align::Center);

    let config_weak = Rc::downgrade(password_config);
    length_spinner.connect_value_changed(move |spinner| {
        if let Some(config_ref) = config_weak.upgrade() {
            config_ref.borrow_mut().length = spinner.value() as usize;
        }
    });

    length_row.add_suffix(&length_spinner);
    group.add(&length_row);

    group
}

/// Creates the character types configuration preferences group
fn create_character_types_preferences_group(
    password_config: &Rc<RefCell<PasswordConfig>>,
) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Character Types");
    group.set_description(Some("Select which character types to include in passwords"));

    let character_type_options = vec![
        (
            "Lowercase Letters",
            "Include lowercase letters (a-z)",
            "include_lowercase",
            true,
        ),
        (
            "Uppercase Letters",
            "Include uppercase letters (A-Z)",
            "include_uppercase",
            true,
        ),
        ("Numbers", "Include numbers (0-9)", "include_numbers", true),
        (
            "Symbols",
            "Include symbols (!@#$%...)",
            "include_symbols",
            false,
        ),
        (
            "Spaces",
            "Include space characters",
            "include_spaces",
            false,
        ),
        (
            "Extended Characters",
            "Include extended characters (áéíóú...)",
            "include_extended",
            false,
        ),
    ];

    for (title, subtitle, field_name, default_enabled) in character_type_options {
        let row = ActionRow::new();
        row.set_title(title);
        row.set_subtitle(subtitle);

        let switch = Switch::new();
        switch.set_active(default_enabled);
        switch.set_valign(gtk4::Align::Center);

        // Use weak reference to avoid circular references and reduce memory usage
        let config_weak = Rc::downgrade(password_config);
        match field_name {
            "include_lowercase" => {
                switch.connect_state_set(move |_, state| {
                    if let Some(config_ref) = config_weak.upgrade() {
                        config_ref.borrow_mut().include_lowercase = state;
                    }
                    glib::Propagation::Proceed
                });
            }
            "include_uppercase" => {
                switch.connect_state_set(move |_, state| {
                    if let Some(config_ref) = config_weak.upgrade() {
                        config_ref.borrow_mut().include_uppercase = state;
                    }
                    glib::Propagation::Proceed
                });
            }
            "include_numbers" => {
                switch.connect_state_set(move |_, state| {
                    if let Some(config_ref) = config_weak.upgrade() {
                        config_ref.borrow_mut().include_numbers = state;
                    }
                    glib::Propagation::Proceed
                });
            }
            "include_symbols" => {
                switch.connect_state_set(move |_, state| {
                    if let Some(config_ref) = config_weak.upgrade() {
                        config_ref.borrow_mut().include_symbols = state;
                    }
                    glib::Propagation::Proceed
                });
            }
            "include_spaces" => {
                switch.connect_state_set(move |_, state| {
                    if let Some(config_ref) = config_weak.upgrade() {
                        config_ref.borrow_mut().include_spaces = state;
                    }
                    glib::Propagation::Proceed
                });
            }
            "include_extended" => {
                switch.connect_state_set(move |_, state| {
                    if let Some(config_ref) = config_weak.upgrade() {
                        config_ref.borrow_mut().include_extended = state;
                    }
                    glib::Propagation::Proceed
                });
            }
            _ => {}
        }

        row.add_suffix(&switch);
        row.set_activatable_widget(Some(&switch));
        group.add(&row);
    }

    group
}

/// Creates the custom characters configuration preferences group
fn create_custom_characters_preferences_group(
    password_config: &Rc<RefCell<PasswordConfig>>,
) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Custom Characters");
    group.set_description(Some(
        "Add or exclude specific characters from password generation",
    ));

    let additional_row = ActionRow::new();
    additional_row.set_title("Additional Characters");
    additional_row.set_subtitle("Extra characters to include in passwords");

    let additional_entry = Entry::new();
    additional_entry.set_placeholder_text(Some("e.g., @#$"));
    additional_entry.set_tooltip_text(Some(
        "Add extra characters to include in password generation",
    ));

    additional_entry.set_valign(gtk4::Align::Center);
    additional_entry.set_size_request(200, -1);

    let config_weak = Rc::downgrade(password_config);
    additional_entry.connect_changed(move |entry| {
        if let Some(config_ref) = config_weak.upgrade() {
            config_ref.borrow_mut().additional_characters = entry.text().to_string();
        }
    });

    additional_row.add_suffix(&additional_entry);
    group.add(&additional_row);

    let excluded_row = ActionRow::new();
    excluded_row.set_title("Excluded Characters");
    excluded_row.set_subtitle("Characters to avoid in passwords (e.g., confusing characters)");

    let excluded_entry = Entry::new();
    excluded_entry.set_placeholder_text(Some("e.g., 0O1l"));
    excluded_entry.set_tooltip_text(Some("Characters to exclude from password generation"));
    excluded_entry.set_valign(gtk4::Align::Center);
    excluded_entry.set_size_request(200, -1);

    let config_weak = Rc::downgrade(password_config);
    excluded_entry.connect_changed(move |entry| {
        if let Some(config_ref) = config_weak.upgrade() {
            config_ref.borrow_mut().excluded_characters = entry.text().to_string();
        }
    });

    excluded_row.add_suffix(&excluded_entry);
    group.add(&excluded_row);

    group
}

#[allow(clippy::too_many_lines)]
/// Creates the password display preferences group with auto-generation capability
fn create_password_display_preferences_group(
    password_config: &Rc<RefCell<PasswordConfig>>,
    target_entry: Option<&Entry>,
    account_ref: Option<&Rc<RefCell<Account>>>,
    parent_window: Option<&PreferencesWindow>,
) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Generated Password");
    group.set_description(Some("View and manage your generated password"));

    let password_row = ActionRow::new();
    password_row.set_title("Password");
    password_row.set_subtitle("Generated password will appear here");

    let password_display = TextView::new();
    password_display.set_editable(false);
    password_display.set_cursor_visible(false);
    password_display.set_wrap_mode(gtk4::WrapMode::Char);
    password_display.set_size_request(-1, 60);
    password_display.add_css_class("password-display");
    password_display.set_margin_top(8);
    password_display.set_margin_bottom(8);

    let display_buffer = password_display.buffer();
    let initial_config = password_config.borrow();
    match generate_password(&initial_config) {
        Ok(initial_password) => {
            display_buffer.set_text(&initial_password);
        }
        Err(_) => {
            display_buffer.set_text("Click 'Generate Password' to create a password");
        }
    }
    drop(initial_config);

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    scrolled_window.set_child(Some(&password_display));
    scrolled_window.set_size_request(-1, 80);

    let password_container = GtkBox::new(Orientation::Vertical, 8);
    password_container.append(&scrolled_window);

    let strength_row = ActionRow::new();
    strength_row.set_title("Password Strength");

    let strength_progress = ProgressBar::new();
    strength_progress.set_show_text(true);
    strength_progress.add_css_class("password-strength");
    strength_progress.set_valign(gtk4::Align::Center);
    strength_progress.set_size_request(200, -1);

    let strength_description = Label::new(Some(""));
    strength_description.add_css_class("caption");
    strength_description.set_halign(gtk4::Align::Start);
    strength_description.set_valign(gtk4::Align::Center);

    // Update strength indicator for initial password
    let buffer = password_display.buffer();
    let start_iter = buffer.start_iter();
    let end_iter = buffer.end_iter();
    let initial_password = buffer.text(&start_iter, &end_iter, false);
    if !initial_password.is_empty()
        && initial_password != "Click 'Generate Password' to create a password"
    {
        let strength = calculate_password_strength(&initial_password);
        #[allow(clippy::cast_lossless)]
        strength_progress.set_fraction(strength as f64 / 100.0);
        strength_progress.set_text(Some(&format!("{strength}%")));
        strength_description.set_text(get_strength_description(strength));
        strength_progress.remove_css_class("strength-weak");
        strength_progress.remove_css_class("strength-fair");
        strength_progress.remove_css_class("strength-good");
        strength_progress.remove_css_class("strength-strong");
        strength_progress.add_css_class(get_strength_color_class(strength));
    }

    let strength_box = GtkBox::new(Orientation::Vertical, 4);
    strength_box.append(&strength_progress);
    strength_box.append(&strength_description);
    strength_row.add_suffix(&strength_box);

    let actions_row = ActionRow::new();
    actions_row.set_title("Actions");
    actions_row.set_subtitle("Generate, copy, or use the password");

    let button_container = GtkBox::new(Orientation::Horizontal, 8);
    button_container.set_halign(gtk4::Align::End);

    let generate_button = Button::new();
    let generate_content = ButtonContent::new();
    generate_content.set_label("Generate");
    generate_content.set_icon_name("view-refresh-symbolic");
    generate_button.set_child(Some(&generate_content));
    generate_button.add_css_class("suggested-action");
    generate_button.set_tooltip_text(Some("Generate a new password (Ctrl+G or F5)"));

    let display_ref = password_display.clone();
    let config_ref = password_config.clone();
    let strength_progress_ref = strength_progress.clone();
    let strength_desc_ref = strength_description.clone();
    generate_button.connect_clicked(move |_| {
        let config = config_ref.borrow();
        match generate_password(&config) {
            Ok(generated_password) => {
                let buffer = display_ref.buffer();
                buffer.set_text(&generated_password);

                let strength = calculate_password_strength(&generated_password);
                #[allow(clippy::cast_lossless)]
                strength_progress_ref.set_fraction(strength as f64 / 100.0);
                strength_progress_ref.set_text(Some(&format!("{strength}%")));
                strength_desc_ref.set_text(get_strength_description(strength));

                strength_progress_ref.remove_css_class("strength-weak");
                strength_progress_ref.remove_css_class("strength-fair");
                strength_progress_ref.remove_css_class("strength-good");
                strength_progress_ref.remove_css_class("strength-strong");
                strength_progress_ref.add_css_class(get_strength_color_class(strength));
            }
            Err(error_message) => {
                eprintln!("Failed to generate password: {error_message}");
                let buffer = display_ref.buffer();
                buffer.set_text("Error generating password");

                strength_progress_ref.set_fraction(0.0);
                strength_progress_ref.set_text(Some("0%"));
                strength_desc_ref.set_text("");
            }
        }
    });

    let copy_button = Button::new();
    let copy_content = ButtonContent::new();
    copy_content.set_label("Copy");
    copy_content.set_icon_name("edit-copy-symbolic");
    copy_button.set_child(Some(&copy_content));
    copy_button.add_css_class("flat");
    copy_button.set_tooltip_text(Some("Copy password to clipboard (Ctrl+C)"));

    let display_copy_ref = password_display.clone();
    let copy_button_ref = copy_button.clone();
    copy_button.connect_clicked(move |_| {
        let buffer = display_copy_ref.buffer();
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();
        let password_text = buffer.text(&start_iter, &end_iter, false);

        if !password_text.is_empty() && password_text != "Click 'Generate Password' to create a password" && let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            clipboard.set_text(&password_text);

            // Visual feedback for copy action
            let copy_content_ref = copy_content.clone();
            copy_content_ref.set_label("Copied!");
            copy_button_ref.add_css_class("success");

            let button_clone = copy_button_ref.clone();
            let content_clone = copy_content.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                content_clone.set_label("Copy");
                button_clone.remove_css_class("success");
                glib::ControlFlow::Break
            });
        }
    });

    button_container.append(&generate_button);
    button_container.append(&copy_button);

    // Add Use button only if we have the necessary parameters
    if let (Some(entry), Some(account_rc), Some(window)) =
        (target_entry, account_ref, parent_window)
    {
        let use_button = Button::new();
        let use_content = ButtonContent::new();
        use_content.set_label("Use Password");
        use_content.set_icon_name("emblem-ok-symbolic");
        use_button.set_child(Some(&use_content));
        use_button.add_css_class("suggested-action");

        let display_use_ref = password_display.clone();
        let entry_ref = entry.clone();
        let account_use_ref = account_rc.clone();
        let window_ref = window.clone();
        use_button.connect_clicked(move |_| {
            let buffer = display_use_ref.buffer();
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let generated_password = buffer.text(&start_iter, &end_iter, false);

            if !generated_password.is_empty()
                && generated_password != "Click 'Generate Password' to create a password"
            {
                entry_ref.set_text(&generated_password);
                let mut account = account_use_ref.borrow_mut();
                account.password.update(generated_password.to_string());
                window_ref.close();
            }
        });

        button_container.append(&use_button);
    }

    actions_row.add_suffix(&button_container);

    password_row.set_child(Some(&password_container));
    group.add(&password_row);
    group.add(&strength_row);
    group.add(&actions_row);

    group
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
    let welcome_window = AdwWindow::new();
    welcome_window.set_title(Some("Welcome to Forgot My Password"));
    welcome_window.set_modal(true);
    welcome_window.set_default_size(600, 500);
    welcome_window.add_css_class("welcome-dialog");

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Welcome"))));
    header_bar.add_css_class("flat");

    let main_container = GtkBox::new(Orientation::Vertical, 0);
    main_container.append(&header_bar);

    let clamp = Clamp::new();
    clamp.set_maximum_size(500);
    clamp.set_tightening_threshold(400);

    let content_box = GtkBox::new(Orientation::Vertical, 24);
    content_box.set_margin_top(32);
    content_box.set_margin_bottom(32);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let title_container = GtkBox::new(Orientation::Horizontal, 12);
    title_container.set_halign(gtk4::Align::Center);

    let title_icon = Label::new(Some("🔐"));
    title_icon.add_css_class("title-1");

    let dialog_title = Label::new(Some("Welcome to Forgot My Password!"));
    dialog_title.add_css_class("title-1");

    title_container.append(&title_icon);
    title_container.append(&dialog_title);
    content_box.append(&title_container);

    let welcome_message_text = "Thank you for choosing Forgot My Password (FMP), a secure local password manager.\n\nFMP helps you:\n• Store passwords securely with GPG encryption\n• Generate strong, unique passwords\n• Manage TOTP codes for two-factor authentication\n• Keep your sensitive data safe and organized\n\nTo get started:\n1. Create your first vault to store passwords\n2. Add accounts with their login credentials\n3. Use the password generator for strong passwords\n4. Enable TOTP for accounts that support it\n\nYour data is encrypted and stored locally for maximum security.";

    let welcome_message_label = Label::new(Some(welcome_message_text));
    welcome_message_label.set_wrap(true);
    welcome_message_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    welcome_message_label.set_justify(gtk4::Justification::Left);
    welcome_message_label.set_halign(gtk4::Align::Fill);
    welcome_message_label.set_valign(gtk4::Align::Start);
    welcome_message_label.add_css_class("body");

    let message_scrolled_window = ScrolledWindow::new();
    message_scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    message_scrolled_window.set_child(Some(&welcome_message_label));
    message_scrolled_window.set_size_request(-1, 280);
    message_scrolled_window.add_css_class("card");
    message_scrolled_window.set_margin_top(8);
    message_scrolled_window.set_margin_bottom(16);

    content_box.append(&message_scrolled_window);

    let button_container = GtkBox::new(Orientation::Horizontal, 12);
    button_container.set_halign(gtk4::Align::Center);
    button_container.set_margin_top(8);

    let learn_more_button = Button::new();
    learn_more_button.set_label("Learn More about GPG");
    learn_more_button.add_css_class("flat");

    learn_more_button.connect_clicked(move |_| {
        show_gpg_info_dialog();
    });

    let get_started_button = Button::new();
    get_started_button.set_label("Get Started");
    get_started_button.add_css_class("suggested-action");
    get_started_button.add_css_class("pill");

    let window_ref = welcome_window.clone();
    get_started_button.connect_clicked(move |_| {
        if let Err(error_message) = mark_first_run_complete() {
            eprintln!("Failed to mark first run complete: {error_message}");
        }
        window_ref.close();
    });

    button_container.append(&learn_more_button);
    button_container.append(&get_started_button);
    content_box.append(&button_container);

    clamp.set_child(Some(&content_box));
    main_container.append(&clamp);

    welcome_window.set_content(Some(&main_container));
    welcome_window.present();
}

/// Shows the GPG information dialog with setup instructions
fn show_gpg_info_dialog() {
    let info_window = AdwWindow::new();
    info_window.set_title(Some("GPG Setup Information"));
    info_window.set_modal(true);
    info_window.set_default_size(600, 500);
    info_window.add_css_class("info-dialog");

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("GPG Setup"))));
    header_bar.add_css_class("flat");

    let main_container = GtkBox::new(Orientation::Vertical, 0);
    main_container.append(&header_bar);

    let clamp = Clamp::new();
    clamp.set_maximum_size(500);
    clamp.set_tightening_threshold(400);

    let content_box = GtkBox::new(Orientation::Vertical, 24);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let title_container = GtkBox::new(Orientation::Horizontal, 12);
    title_container.set_halign(gtk4::Align::Center);

    let title_icon = Label::new(Some("🔑"));
    title_icon.add_css_class("title-2");

    let dialog_title = Label::new(Some("Setting up GPG"));
    dialog_title.add_css_class("title-2");

    title_container.append(&title_icon);
    title_container.append(&dialog_title);
    content_box.append(&title_container);

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

    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled.set_child(Some(&instructions_box));
    scrolled.set_size_request(-1, 200);
    scrolled.add_css_class("card");
    scrolled.set_margin_top(8);
    scrolled.set_margin_bottom(16);

    content_box.append(&scrolled);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);

    let close_button = Button::new();
    close_button.set_label("Close");
    close_button.add_css_class("suggested-action");

    let window_clone = info_window.clone();
    close_button.connect_clicked(move |_| {
        window_clone.close();
    });

    button_box.append(&close_button);
    content_box.append(&button_box);

    clamp.set_child(Some(&content_box));
    main_container.append(&clamp);

    info_window.set_content(Some(&main_container));
    info_window.present();
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
    let dialog = MessageDialog::new(
        parent,
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Warning,
        ButtonsType::None,
        message,
    );

    dialog.set_title(Some(title));
    dialog.add_css_class("confirmation-dialog");

    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button(confirm_label, ResponseType::Accept);

    if let Some(confirm_button) = dialog.widget_for_response(ResponseType::Accept) {
        confirm_button.add_css_class("destructive-action");
    }

    dialog.set_default_response(ResponseType::Cancel);

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            on_confirm();
        }
        dialog.close();
    });

    dialog.present();
}

/// Shows the TOTP setup dialog for enabling 2FA on a vault
pub fn show_totp_setup_dialog(vault_name: &str) {
    let totp_window = PreferencesWindow::new();
    totp_window.set_title(Some("Enable Two-Factor Authentication"));
    totp_window.set_modal(true);
    totp_window.set_default_size(600, 650);
    totp_window.set_search_enabled(false);

    let page = adw::PreferencesPage::new();
    page.set_title("Two-Factor Authentication");
    page.set_icon_name(Some("security-high-symbolic"));

    let setup_group = PreferencesGroup::new();
    setup_group.set_title(&format!("Enable 2FA for \"{vault_name}\""));
    setup_group.set_description(Some("Secure your vault with two-factor authentication"));

    let instructions_row = ActionRow::new();
    instructions_row.set_title("Setup Instructions");
    instructions_row.set_subtitle(
        "Scan the QR code with your authenticator app, then enter the verification code",
    );
    setup_group.add(&instructions_row);

    // QR Code group (will be populated after setup)
    let qr_group = PreferencesGroup::new();
    qr_group.set_title("QR Code");
    qr_group.set_description(Some("Scan this code with your authenticator app"));

    // Show loading message initially
    let loading_row = ActionRow::new();
    loading_row.set_title("Generating QR code...");
    loading_row.set_subtitle("Please wait while the 2FA QR code is generated...");
    qr_group.add(&loading_row);

    // Verification group (will be populated after setup)
    let verification_group = PreferencesGroup::new();
    verification_group.set_title("Verification");
    verification_group.set_description(Some("Enter the code from your authenticator app"));

    page.add(&setup_group);
    page.add(&qr_group);
    page.add(&verification_group);

    totp_window.add(&page);
    totp_window.present();

    // Prepare TOTP secret and QR code after showing dialog (without enabling yet)
    match prepare_totp_setup(vault_name) {
        Ok((secret, secret_b32, otpauth_uri)) => {
            // Remove loading message
            qr_group.remove(&loading_row);

            let qr_row = ActionRow::new();
            qr_row.set_title("QR Code");
            qr_row.set_subtitle(&format!("Secret: {secret_b32}"));

            if let Ok(qr_image) = generate_qr_code_image(&otpauth_uri) {
                let qr_image_widget = Image::from_pixbuf(Some(&qr_image));
                qr_image_widget.set_pixel_size(200);
                qr_row.set_child(Some(&qr_image_widget));
            }

            qr_group.add(&qr_row);

            // Add verification input
            let verification_row = ActionRow::new();
            verification_row.set_title("Verification Code");
            verification_row.set_subtitle("Enter the 6-digit code from your authenticator app");

            let code_entry = Entry::new();
            code_entry.set_placeholder_text(Some("000000"));
            code_entry.set_max_length(6);
            code_entry.set_input_purpose(gtk4::InputPurpose::Digits);
            code_entry.set_valign(gtk4::Align::Center);
            code_entry.set_size_request(120, -1);

            let verify_button = Button::new();
            verify_button.set_label("Enable 2FA");
            verify_button.add_css_class("suggested-action");
            verify_button.set_valign(gtk4::Align::Center);

            let button_box = GtkBox::new(Orientation::Horizontal, 8);
            button_box.append(&code_entry);
            button_box.append(&verify_button);
            verification_row.add_suffix(&button_box);

            let vault_name_clone = vault_name.to_string();
            let secret_clone = secret.clone();
            let window_clone = totp_window.clone();

            verify_button.connect_clicked(move |_| {
                let code = code_entry.text();
                if code.len() == 6 {
                    match verify_totp_code_with_secret(&secret_clone, &code) {
                        Ok(true) => {
                            if let Err(e) = confirm_totp_setup(&vault_name_clone, &secret_clone) {
                                eprintln!("Failed to confirm TOTP setup: {e}");
                            } else {
                                window_clone.close();
                            }
                        }
                        Ok(false) => {
                            code_entry.add_css_class("error");
                            // Remove error class after a delay
                            let entry_clone = code_entry.clone();
                            glib::timeout_add_local(std::time::Duration::from_secs(3), move || {
                                entry_clone.remove_css_class("error");
                                glib::ControlFlow::Break
                            });
                        }
                        Err(e) => {
                            eprintln!("TOTP verification error: {e}");
                        }
                    }
                }
            });

            verification_group.add(&verification_row);
        }
        Err(e) => {
            // Remove loading message and show error
            qr_group.remove(&loading_row);

            let error_row = ActionRow::new();
            error_row.set_title("Setup Failed");
            error_row.set_subtitle(&format!("Failed to prepare 2FA: {e}"));
            qr_group.add(&error_row);
        }
    }
}

/// Shows the TOTP management dialog for an already enabled vault
pub fn show_totp_management_dialog(vault_name: &str, content_area: &GtkBox) {
    let totp_window = PreferencesWindow::new();
    totp_window.set_title(Some("Manage Two-Factor Authentication"));
    totp_window.set_modal(true);
    totp_window.set_default_size(600, 500);
    totp_window.set_search_enabled(false);

    let page = adw::PreferencesPage::new();
    page.set_title("Two-Factor Authentication");
    page.set_icon_name(Some("security-high-symbolic"));

    let status_group = PreferencesGroup::new();
    status_group.set_title(&format!("2FA Settings for \"{vault_name}\""));
    status_group.set_description(Some("Manage your two-factor authentication settings"));

    let status_row = ActionRow::new();
    status_row.set_title("Status");
    status_row.set_subtitle("Two-factor authentication is currently enabled");

    let status_icon = Image::from_icon_name("emblem-ok-symbolic");
    status_icon.add_css_class("success");
    status_row.add_prefix(&status_icon);
    status_group.add(&status_row);

    match get_totp_qr_info(vault_name) {
        Ok((secret, otpauth_uri)) => {
            let qr_group: PreferencesGroup = PreferencesGroup::new();
            qr_group.set_title("Backup QR Code");
            qr_group.set_description(Some("Use this QR code to set up 2FA on additional devices"));

            let qr_row = ActionRow::new();
            qr_row.set_title("QR Code");
            qr_row.set_subtitle(&format!("Secret: {secret}"));

            if let Ok(qr_image) = generate_qr_code_image(&otpauth_uri) {
                let qr_image_widget = Image::from_pixbuf(Some(&qr_image));
                qr_image_widget.set_pixel_size(200);
                qr_row.set_child(Some(&qr_image_widget));
            }

            qr_group.add(&qr_row);

            let actions_group = PreferencesGroup::new();
            actions_group.set_title("Actions");
            actions_group.set_description(Some("Manage your two-factor authentication"));

            let disable_row = ActionRow::new();
            disable_row.set_title("Disable Two-Factor Authentication");
            disable_row.set_subtitle("Remove 2FA protection from this vault");

            let disable_button = Button::new();
            disable_button.set_label("Disable");
            disable_button.add_css_class("destructive-action");
            disable_button.set_valign(gtk4::Align::Center);

            let vault_name_clone = vault_name.to_string();
            let content_area_clone = content_area.clone();
            let window_clone = totp_window.clone();
            disable_button.connect_clicked(move |_| {
                show_confirmation_dialog(
                    "Disable Two-Factor Authentication",
                    &format!("Are you sure you want to disable 2FA for vault \"{vault_name_clone}\"?\n\nThis will make your vault less secure."),
                    "Disable 2FA",
                    Some(&window_clone),
                    {
                        let vault_name_clone2 = vault_name_clone.clone();
                        let content_area_clone2 = content_area_clone.clone();
                        let window_clone2 = window_clone.clone();
                        move || {
                            match disable_totp(&vault_name_clone2) {
                                Ok(()) => {
                                    window_clone2.close();
                                    crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area_clone2);
                                    crate::gui::content::show_vault_view(&content_area_clone2, &vault_name_clone2);
                                }
                                Err(e) => {
                                    eprintln!("Failed to disable 2FA: {e}");
                                }
                            }
                        }
                    }
                );
            });

            disable_row.add_suffix(&disable_button);
            disable_row.set_activatable_widget(Some(&disable_button));
            actions_group.add(&disable_row);

            page.add(&status_group);
            page.add(&qr_group);
            page.add(&actions_group);
        }
        Err(e) => {
            let error_row = ActionRow::new();
            error_row.set_title("Error Loading 2FA Information");
            error_row.set_subtitle(&format!("Failed to load 2FA info: {e}"));
            status_group.add(&error_row);

            page.add(&status_group);
        }
    }

    totp_window.add(&page);
    totp_window.present();
}

/// Generates a QR code image from an otpauth URI
fn generate_qr_code_image(otpauth_uri: &str) -> Result<Pixbuf, Box<dyn std::error::Error>> {
    // Generate QR code with larger module size for better performance and visibility
    let qr_code = QrCode::new(otpauth_uri)?;

    // Use the QR code renderer's built-in scaling for much better performance
    let image = qr_code
        .render::<Luma<u8>>()
        .min_dimensions(300, 300)
        .module_dimensions(15, 15)
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
        false,
        8,
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
            crate::gui::content::show_vault_view(&content_area_clone, &vault_name_clone);
        }
        Err(e) => {
            eprintln!("Failed to create backup: {e}");
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

    let title = Label::new(Some(&format!("Restore Backup for '{vault_name}'")));
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
                eprintln!("Failed to restore backup: {e}");
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
            crate::gui::content::show_vault_view(&content_area_clone, &vault_name_clone);
        }
        Err(e) => {
            eprintln!("Failed to delete backup: {e}");
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
                    crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area_clone);
                    crate::gui::content::show_vault_view(&content_area_clone, &new_name);
                }
                Err(e) => {
                    eprintln!("Failed to rename vault: {e}");
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

    let title = Label::new(Some(&format!("Delete Vault '{vault_name}'")));
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
                    crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area_clone);
                    crate::gui::content::show_home_view(&content_area_clone);
                }
                Err(e) => {
                    eprintln!("Failed to delete vault: {e}");
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
                    crate::gui::content::show_account_view_with_mode(
                        &content_area_clone,
                        &vault_name_clone,
                        &new_name,
                        false,
                    );
                }
                Err(e) => {
                    eprintln!("Failed to rename account: {e}");
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

                    crate::gui::content::show_account_view_with_mode(
                        &content_area_clone,
                        &vault_name_clone,
                        &account_name,
                        true,
                    );
                }
                Err(e) => {
                    drop(account);
                    eprintln!("Failed to save account: {e}");
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

/// Shows a simple error dialog
fn show_error_dialog(title: &str, message: &str) {
    let dialog = Dialog::new();
    dialog.set_title(Some(title));
    dialog.set_modal(true);
    dialog.set_default_size(300, 150);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let message_label = Label::new(Some(message));
    message_label.set_wrap(true);
    message_label.set_halign(gtk4::Align::Center);
    content_box.append(&message_label);

    let ok_button = Button::new();
    ok_button.set_label("OK");
    ok_button.add_css_class("suggested-action");
    ok_button.set_halign(gtk4::Align::Center);

    let dialog_clone = dialog.clone();
    ok_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    content_box.append(&ok_button);
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

                    crate::gui::content::show_account_view_with_mode(
                        &content_area_clone,
                        &vault_name_clone,
                        &account_name,
                        true, // Keep in edit mode
                    );
                }
                Err(e) => {
                    drop(account);
                    eprintln!("Failed to save account: {e}");
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

    let warning_icon = Label::new(Some("⚠️"));
    warning_icon.add_css_class("title-2");

    let title_label = Label::new(Some("Delete Field"));
    title_label.add_css_class("title-3");

    title_box.append(&warning_icon);
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
                crate::gui::content::show_account_view_with_mode(
                    &content_area_clone,
                    &vault_name_clone,
                    &account_name,
                    true, // Keep in edit mode
                );
            }
            Err(e) => {
                drop(account);
                eprintln!("Failed to save account: {e}");
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

/// Shows the TOTP authentication dialog for vault access
pub fn show_totp_authentication_dialog<F>(vault_name: &str, on_success: F)
where
    F: Fn() + 'static + Clone,
{
    let dialog = Dialog::new();
    dialog.set_title(Some("Two-Factor Authentication"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 250);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Access Vault '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let instructions = Label::new(Some("Enter the 6-digit code from your authenticator app"));
    instructions.add_css_class("body");
    instructions.set_halign(gtk4::Align::Center);
    instructions.set_margin_bottom(12);
    content_box.append(&instructions);

    let code_container = GtkBox::new(Orientation::Vertical, 8);
    code_container.set_halign(gtk4::Align::Center);

    let code_entry = Entry::new();
    code_entry.set_placeholder_text(Some("000000"));
    code_entry.set_max_length(8);
    code_entry.set_width_chars(10);
    code_entry.set_halign(gtk4::Align::Center);
    code_entry.add_css_class("totp-code-entry");

    code_container.append(&code_entry);
    content_box.append(&code_container);

    let status_label = Label::new(None);
    status_label.set_halign(gtk4::Align::Center);
    content_box.append(&status_label);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let verify_button = Button::new();
    verify_button.set_label("Enter");
    verify_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let vault_name_clone = vault_name.to_string();
    let dialog_clone = dialog.clone();
    let code_entry_clone = code_entry.clone();
    let status_label_clone = status_label.clone();
    let on_success_clone = on_success.clone();
    verify_button.connect_clicked(move |_| {
        let code = code_entry_clone.text();

        match verify_totp_code(&vault_name_clone, &code) {
            Ok(true) => {
                status_label_clone.set_text("✅ Authentication successful!");
                status_label_clone.add_css_class("success");

                let dialog_clone2 = dialog_clone.clone();
                let on_success_clone2 = on_success_clone.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                    dialog_clone2.close();
                    on_success_clone2();
                    glib::ControlFlow::Break
                });
            }
            Ok(false) => {
                status_label_clone.set_text("❌ Invalid code. Please try again.");
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
                code_entry_clone.select_region(0, -1);
            }
            Err(e) => {
                status_label_clone.set_text(&format!("❌ Error: {e}"));
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
            }
        }
    });

    let verify_button_clone = verify_button.clone();
    code_entry.connect_activate(move |_| {
        verify_button_clone.emit_clicked();
    });

    button_box.append(&cancel_button);
    button_box.append(&verify_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}
