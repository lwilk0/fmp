/// All this code is awful, sorry future me. There must be a better way????
use crate::{
    password::{
        PasswordConfig, calculate_password_strength, generate_password, get_strength_color_class,
        get_strength_description,
    },
    vault::Account,
};

use adw::{ActionRow, ButtonContent, PreferencesGroup, PreferencesWindow, prelude::*};
use gtk4::{
    Adjustment, Box as GtkBox, Button, Entry, Label, Orientation, PolicyType, ProgressBar,
    ScrolledWindow, SpinButton, Switch, TextView, gdk,
    glib::{self},
};
use std::{cell::RefCell, rc::Rc};

/// Shows the password generator dialog and updates the provided entry field and account
pub fn show_password_generator_dialog(
    target_entry: Option<&Entry>,
    account_ref: Option<&Rc<RefCell<Account>>>,
) {
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
        target_entry,
        account_ref,
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
    length_row.set_margin_start(8);
    length_row.set_margin_end(8);

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
        row.set_margin_start(8);
        row.set_margin_end(8);

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
    additional_row.set_margin_start(8);
    additional_row.set_margin_end(8);

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
    excluded_row.set_margin_start(8);
    excluded_row.set_margin_end(8);

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
    password_row.set_margin_start(8);
    password_row.set_margin_end(8);

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
    strength_row.set_margin_start(8);
    strength_row.set_margin_end(8);

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
        update_strength_indicator(&strength_progress, &strength_description, &initial_password);
    }

    let strength_box = GtkBox::new(Orientation::Vertical, 4);
    strength_box.append(&strength_progress);
    strength_box.append(&strength_description);
    strength_row.add_suffix(&strength_box);

    let actions_row = ActionRow::new();
    actions_row.set_title("Actions");
    actions_row.set_subtitle("Generate, copy, or use the password");
    actions_row.set_margin_start(8);
    actions_row.set_margin_end(8);

    let button_container = GtkBox::new(Orientation::Horizontal, 8);
    button_container.set_halign(gtk4::Align::End);

    let generate_button = Button::new();
    let generate_content = ButtonContent::new();
    generate_content.set_label("Generate");
    generate_content.set_icon_name("view-refresh-symbolic");
    generate_button.set_child(Some(&generate_content));
    generate_button.add_css_class("suggested-action");
    generate_button.set_tooltip_text(Some("Generate a new password"));

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

                update_strength_indicator(
                    &strength_progress,
                    &strength_description,
                    &generated_password,
                );
            }
            Err(error_message) => {
                log::error!("Failed to generate password: {error_message}");
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
    copy_button.set_tooltip_text(Some("Copy password to clipboard"));

    let display_copy_ref = password_display.clone();
    let copy_button_ref = copy_button.clone();
    copy_button.connect_clicked(move |_| {
        let buffer = display_copy_ref.buffer();
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();
        let password_text = buffer.text(&start_iter, &end_iter, false);

        if !password_text.is_empty()
            && password_text != "Click 'Generate Password' to create a password"
            && let Some(display) = gdk::Display::default()
        {
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

            glib::timeout_add_seconds_local(30, move || {
                clipboard.set_text("");
                log::info!("Clipboard cleared for security");
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
                account.password.update(&generated_password);
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

fn update_strength_indicator(progress: &ProgressBar, desc: &Label, password: &str) {
    let strength = calculate_password_strength(password);
    progress.set_fraction(strength as f64 / 100.0);
    progress.set_text(Some(&format!("{strength}%")));
    desc.set_text(get_strength_description(strength));
    for cls in [
        "strength-weak",
        "strength-fair",
        "strength-good",
        "strength-strong",
    ] {
        progress.remove_css_class(cls);
    }
    progress.add_css_class(get_strength_color_class(strength));
}
