use adw::prelude::*;

use crate::gui::sidebar::refresh_vaults_section;
use crate::vault::{
    Account, Locations, create_account, create_vault, get_full_account_details, read_directory,
    update_account,
};
use gtk4::pango::EllipsizeMode;
use gtk4::{
    Box, Button, Entry, FlowBox, Label, Orientation, PolicyType, ScrolledWindow, SelectionMode,
    Separator, gdk,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// Shows the home view in the content area
pub fn show_home_view(content_area: &Box) {
    clear_content(content_area);

    let title = Label::new(Some("Home"));
    title.add_css_class("title-1");
    title.set_margin_bottom(20);
    content_area.append(&title);

    let placeholder = Label::new(Some("Home panel will be implemented here"));
    placeholder.add_css_class("dim-label");
    content_area.append(&placeholder);
}

/// Shows the settings view in the content area
pub fn show_settings_view(content_area: &Box) {
    clear_content(content_area);

    let title = Label::new(Some("Settings"));
    title.add_css_class("title-1");
    title.set_margin_bottom(20);
    content_area.append(&title);

    let placeholder = Label::new(Some("Settings panel will be implemented here"));
    placeholder.add_css_class("dim-label");
    content_area.append(&placeholder);
}

/// Clears all content from the content area
fn clear_content(content_area: &Box) {
    while let Some(child) = content_area.first_child() {
        content_area.remove(&child);
    }
}

/// Shows a specific vault view with improved account display
pub fn show_vault_view(content_area: &Box, vault_name: &str) {
    clear_content(content_area);

    // Create main container with scrolling
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let main_box = Box::new(Orientation::Vertical, 16);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    // Vault title with description
    let header_box = Box::new(Orientation::Vertical, 8);
    let title = Label::new(Some(vault_name));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);

    let subtitle = Label::new(Some("Manage your accounts and passwords"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Start);

    header_box.append(&title);
    header_box.append(&subtitle);
    main_box.append(&header_box);

    // Accounts section
    let accounts_section = create_accounts_grid(content_area, vault_name);
    main_box.append(&accounts_section);

    scrolled_window.set_child(Some(&main_box));
    content_area.append(&scrolled_window);
}

/// Creates a modern grid layout for accounts
fn create_accounts_grid(content_area: &Box, vault_name: &str) -> Box {
    let container = Box::new(Orientation::Vertical, 16);

    // Section header
    let header_box = Box::new(Orientation::Horizontal, 12);
    let accounts_title = Label::new(Some("Accounts"));
    accounts_title.add_css_class("title-3");
    accounts_title.set_halign(gtk4::Align::Start);
    accounts_title.set_hexpand(true);

    // Add button for creating new accounts
    let add_button = Button::new();
    add_button.set_label("Add Account");
    add_button.add_css_class("suggested-action");
    add_button.set_halign(gtk4::Align::End);

    // Connect add account functionality
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    add_button.connect_clicked(move |_| {
        show_new_account_view(&content_area_clone, &vault_name_clone);
    });

    header_box.append(&accounts_title);
    header_box.append(&add_button);
    container.append(&header_box);

    // Get available accounts
    let all_accounts = get_available_accounts(vault_name);

    if all_accounts.is_empty() {
        let empty_state = create_empty_state();
        container.append(&empty_state);
    } else {
        // Create flow box for responsive grid layout
        let flow_box = FlowBox::new();
        flow_box.set_max_children_per_line(4);
        flow_box.set_min_children_per_line(1);
        flow_box.set_row_spacing(12);
        flow_box.set_column_spacing(12);
        flow_box.set_selection_mode(SelectionMode::None);

        for account_name in all_accounts {
            let account_card = create_account_card(account_name.as_str(), content_area, vault_name);
            flow_box.insert(&account_card, -1);
        }

        container.append(&flow_box);
    }

    container
}

/// Creates an empty state widget when no accounts are found
fn create_empty_state() -> Box {
    let empty_box = Box::new(Orientation::Vertical, 16);
    empty_box.set_halign(gtk4::Align::Center);
    empty_box.set_valign(gtk4::Align::Center);
    empty_box.set_margin_top(48);
    empty_box.set_margin_bottom(48);

    let icon_label = Label::new(Some("🔐"));
    icon_label.add_css_class("title-1");

    let message = Label::new(Some("No accounts found"));
    message.add_css_class("title-4");

    let description = Label::new(Some("Create your first account to get started"));
    description.add_css_class("dim-label");

    empty_box.append(&icon_label);
    empty_box.append(&message);
    empty_box.append(&description);

    empty_box
}

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

/// Creates a card-style button for a specific account
fn create_account_card(account_name: &str, content_area: &Box, vault_name: &str) -> Button {
    let button = Button::new();
    button.add_css_class("card");
    button.add_css_class("account-card");
    button.set_size_request(200, 120);

    // Create card content
    let card_box = Box::new(Orientation::Vertical, 8);
    card_box.set_margin_top(16);
    card_box.set_margin_bottom(16);
    card_box.set_margin_start(16);
    card_box.set_margin_end(16);

    // Account icon (you can customize this based on account type)
    let icon_label = Label::new(Some("👤"));
    icon_label.add_css_class("title-2");
    icon_label.set_halign(gtk4::Align::Center);

    // Account name
    let name_label = Label::new(Some(account_name));
    name_label.add_css_class("title-4");
    name_label.set_halign(gtk4::Align::Center);
    name_label.set_ellipsize(EllipsizeMode::End);
    name_label.set_max_width_chars(20);

    // Account type/description (placeholder)
    let type_label = Label::new(Some("Password Account"));
    type_label.add_css_class("dim-label");
    type_label.add_css_class("caption");
    type_label.set_halign(gtk4::Align::Center);

    card_box.append(&icon_label);
    card_box.append(&name_label);
    card_box.append(&type_label);
    button.set_child(Some(&card_box));

    // Connect click handler
    let account_name_clone = account_name.to_string();
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    button.connect_clicked(move |_| {
        println!("Opening account: {account_name_clone}");
        show_account_view(&content_area_clone, &vault_name_clone, &account_name_clone);
    });

    button
}

/// Shows the account view with detailed information and controls
pub fn show_account_view(content_area: &Box, vault_name: &str, account_name: &str) {
    show_account_view_with_mode(content_area, vault_name, account_name, false);
}

/// Shows the account view with edit mode option
fn show_account_view_with_mode(
    content_area: &Box,
    vault_name: &str,
    account_name: &str,
    edit_mode: bool,
) {
    clear_content(content_area);

    // Try to load account data
    let account_data = match get_full_account_details(vault_name, account_name) {
        Ok(account) => Some(account),
        Err(e) => {
            eprintln!("Failed to load account data: {}", e);
            None
        }
    };

    // Create main container with scrolling
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    // Wrap account data in Rc<RefCell<>> for sharing between sections
    let account_rc = Rc::new(RefCell::new(account_data.unwrap_or_else(|| {
        let mut default_account = Account::default();
        default_account.name = account_name.to_string();
        default_account
    })));

    // Header section with account name and actions
    let header_section = create_account_header(
        &account_rc,
        content_area,
        vault_name,
        account_name,
        edit_mode,
    );
    main_box.append(&header_section);

    // Account details section
    let details_section = create_account_details_section(&account_rc, edit_mode);
    main_box.append(&details_section);

    // Password section
    let password_section = create_password_section(&account_rc, edit_mode);
    main_box.append(&password_section);

    // Additional fields section
    let additional_fields_section = create_additional_fields_section(&account_rc);
    main_box.append(&additional_fields_section);

    // Notes section
    let notes_section = create_notes_section(&account_rc, edit_mode);
    main_box.append(&notes_section);

    // Action buttons section (only show in edit mode)
    if edit_mode {
        let actions_section = create_account_actions_section(&account_rc, vault_name, content_area);
        main_box.append(&actions_section);
    }

    scrolled_window.set_child(Some(&main_box));
    content_area.append(&scrolled_window);
}

/// Creates the account header with title and action buttons
fn create_account_header(
    account_rc: &Rc<RefCell<Account>>,
    content_area: &Box,
    vault_name: &str,
    account_name: &str,
    edit_mode: bool,
) -> Box {
    let header_box = Box::new(Orientation::Horizontal, 16);
    header_box.set_halign(gtk4::Align::Fill);

    // Left side - account info
    let info_box = Box::new(Orientation::Vertical, 4);
    info_box.set_hexpand(true);

    let account = account_rc.borrow();
    let title = Label::new(Some(&account.name));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);

    let subtitle = Label::new(Some(&account.account_type));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Start);

    info_box.append(&title);
    info_box.append(&subtitle);

    // Right side - action buttons
    let actions_box = Box::new(Orientation::Horizontal, 8);

    if !edit_mode {
        let edit_button = Button::new();
        edit_button.set_label("Edit");
        edit_button.add_css_class("suggested-action");

        // Connect edit functionality
        let content_area_clone = content_area.clone();
        let vault_name_clone = vault_name.to_string();
        let account_name_clone = account_name.to_string();
        edit_button.connect_clicked(move |_| {
            show_account_view_with_mode(
                &content_area_clone,
                &vault_name_clone,
                &account_name_clone,
                true,
            );
        });

        let delete_button = Button::new();
        delete_button.set_label("Delete");
        delete_button.add_css_class("destructive-action");

        actions_box.append(&edit_button);
        actions_box.append(&delete_button);
    }

    header_box.append(&info_box);
    header_box.append(&actions_box);

    header_box
}

/// Creates the account details section
fn create_account_details_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 16);

    // Section title
    let title = Label::new(Some("Account Details"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    // Details grid
    let details_box = Box::new(Orientation::Vertical, 12);

    let account = account_rc.borrow();

    if edit_mode {
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

    // Always show read-only date fields
    let created_row = create_field_row("Created", &account.created_at, false);
    details_box.append(&created_row);

    let modified_row = create_field_row("Last Modified", &account.modified_at, false);
    details_box.append(&modified_row);

    section.append(&details_box);
    section
}

/// Creates the password section
fn create_password_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 16);

    // Section title with generate button
    let header_box = Box::new(Orientation::Horizontal, 12);
    let title = Label::new(Some("Password"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);
    title.set_hexpand(true);

    let generate_button = Button::new();
    generate_button.set_label("Generate New");
    generate_button.add_css_class("flat");

    header_box.append(&title);
    header_box.append(&generate_button);
    section.append(&header_box);

    // Password field with reveal/copy buttons
    let password_box = Box::new(Orientation::Horizontal, 8);

    let password_entry = Entry::new();
    let account = account_rc.borrow();

    // In edit mode, show the actual password; in view mode, show masked
    if edit_mode {
        password_entry.set_text(&account.password);
    } else {
        let masked_password = "•".repeat(account.password.len().max(8));
        password_entry.set_text(&masked_password);
    }
    password_entry.set_editable(edit_mode);
    password_entry.set_hexpand(true);
    password_entry.add_css_class("password-field");

    // Connect password changes in edit mode
    if edit_mode {
        let account_rc_edit = account_rc.clone();
        password_entry.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let mut account = account_rc_edit.borrow_mut();
            account.password = text;
        });
    }

    let reveal_button = Button::new();
    reveal_button.set_label("👁");
    reveal_button.add_css_class("flat");
    reveal_button.set_tooltip_text(Some("Show/Hide Password"));

    // Add reveal/hide functionality (only in view mode)
    if !edit_mode {
        let password_entry_clone = password_entry.clone();
        let account_rc_clone = account_rc.clone();
        let is_revealed = Rc::new(RefCell::new(false));
        let is_revealed_clone = is_revealed.clone();

        reveal_button.connect_clicked(move |_| {
            let mut revealed = is_revealed_clone.borrow_mut();
            let account = account_rc_clone.borrow();

            if *revealed {
                // Hide password
                let masked_password = "•".repeat(account.password.len().max(8));
                password_entry_clone.set_text(&masked_password);
                *revealed = false;
            } else {
                // Show password
                password_entry_clone.set_text(&account.password);
                *revealed = true;
            }
        });
    }

    let copy_button = Button::new();
    copy_button.set_label("📋");
    copy_button.add_css_class("flat");
    copy_button.set_tooltip_text(Some("Copy Password"));

    // Add copy functionality
    let account_rc_copy = account_rc.clone();
    copy_button.connect_clicked(move |button| {
        let account = account_rc_copy.borrow();
        let display = button.display();
        let clipboard = display.clipboard();
        clipboard.set_text(&account.password);
        println!("Password copied to clipboard");
    });

    password_box.append(&password_entry);

    // Only show reveal button in view mode
    if !edit_mode {
        password_box.append(&reveal_button);
    }

    password_box.append(&copy_button);

    section.append(&password_box);

    section
}

/// Creates the additional fields section
fn create_additional_fields_section(account_rc: &Rc<RefCell<Account>>) -> Box {
    let section = Box::new(Orientation::Vertical, 16);

    // Section title with add button
    let header_box = Box::new(Orientation::Horizontal, 12);
    let title = Label::new(Some("Additional Fields"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);
    title.set_hexpand(true);

    let add_button = Button::new();
    add_button.set_label("Add Field");
    add_button.add_css_class("flat");

    header_box.append(&title);
    header_box.append(&add_button);
    section.append(&header_box);

    // Additional fields from account data
    let fields_box = Box::new(Orientation::Vertical, 12);

    let account = account_rc.borrow();
    for (field_name, field_value) in &account.additional_fields {
        let field_row = create_field_row(field_name, field_value, true);
        fields_box.append(&field_row);
    }

    // If no additional fields, show a placeholder
    if account.additional_fields.is_empty() {
        let placeholder = Label::new(Some("No additional fields"));
        placeholder.add_css_class("dim-label");
        placeholder.set_halign(gtk4::Align::Start);
        fields_box.append(&placeholder);
    }

    section.append(&fields_box);
    section
}

/// Creates the notes section
fn create_notes_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 16);

    let title = Label::new(Some("Notes"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);
    section.append(&title);

    // Notes text area (using Entry for now, could be TextView for multiline)
    let notes_entry = Entry::new();
    let account = account_rc.borrow();
    notes_entry.set_text(&account.notes);
    notes_entry.set_hexpand(true);
    notes_entry.set_editable(edit_mode);
    notes_entry.add_css_class("notes-field");

    // Connect notes changes in edit mode
    if edit_mode {
        let account_rc_notes = account_rc.clone();
        notes_entry.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let mut account = account_rc_notes.borrow_mut();
            account.notes = text;
        });
    }

    section.append(&notes_entry);
    section
}

/// Creates the action buttons section
fn create_account_actions_section(
    account_rc: &Rc<RefCell<Account>>,
    vault_name: &str,
    content_area: &Box,
) -> Box {
    let section = Box::new(Orientation::Vertical, 16);

    // Separator
    let separator = Separator::new(Orientation::Horizontal);
    section.append(&separator);

    // Action buttons
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);

    let save_button = Button::new();
    save_button.set_label("Save Changes");
    save_button.add_css_class("suggested-action");
    save_button.set_size_request(120, -1);

    // Add save functionality
    let account_rc_clone = account_rc.clone();
    let vault_name_clone = vault_name.to_string();
    let content_area_clone3 = content_area.clone();
    save_button.connect_clicked(move |_| {
        let mut account = account_rc_clone.borrow_mut();
        account.update_modified_time();
        let account_name = account.name.clone();

        match update_account(&vault_name_clone, &*account) {
            Ok(()) => {
                println!("Account saved successfully");
                drop(account); // Release the borrow
                // Exit edit mode and show the updated account
                show_account_view(&content_area_clone3, &vault_name_clone, &account_name);
            }
            Err(e) => {
                eprintln!("Failed to save account: {}", e);
                // TODO: Show error toast
            }
        }
    });

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    // Connect cancel functionality - exit edit mode
    let vault_name_clone2 = vault_name.to_string();
    let content_area_clone2 = content_area.clone();
    let account_rc_clone3 = account_rc.clone();
    cancel_button.connect_clicked(move |_| {
        let account = account_rc_clone3.borrow();
        show_account_view(&content_area_clone2, &vault_name_clone2, &account.name);
    });

    actions_box.append(&cancel_button);
    actions_box.append(&save_button);

    section.append(&actions_box);
    section
}

/// Creates a field row with label and value
fn create_field_row(label_text: &str, value_text: &str, copyable: bool) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 12);
    row_box.set_halign(gtk4::Align::Fill);

    // Label
    let label = Label::new(Some(label_text));
    label.add_css_class("dim-label");
    label.set_halign(gtk4::Align::Start);
    label.set_size_request(150, -1);

    // Value container
    let value_box = Box::new(Orientation::Horizontal, 8);
    value_box.set_hexpand(true);

    let value_entry = Entry::new();
    value_entry.set_text(value_text);
    value_entry.set_editable(false);
    value_entry.set_hexpand(true);
    value_entry.add_css_class("flat");

    value_box.append(&value_entry);

    // Copy button for copyable fields
    if copyable {
        let copy_button = Button::new();
        copy_button.set_label("📋");
        copy_button.add_css_class("flat");
        copy_button.set_tooltip_text(Some("Copy to clipboard"));

        // Add copy functionality
        let value_text_owned = value_text.to_string();
        copy_button.connect_clicked(move |button| {
            let display = button.display();
            let clipboard = display.clipboard();
            clipboard.set_text(&value_text_owned);
            println!("Copied '{}' to clipboard", value_text_owned);
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
    let row_box = Box::new(Orientation::Horizontal, 12);
    row_box.set_halign(gtk4::Align::Fill);

    // Label
    let label = Label::new(Some(label_text));
    label.add_css_class("dim-label");
    label.set_halign(gtk4::Align::Start);
    label.set_size_request(150, -1);

    // Password container
    let password_container = Box::new(Orientation::Horizontal, 8);
    password_container.set_hexpand(true);

    // Entry field
    let entry = Entry::new();
    entry.set_text(initial_value);
    entry.set_hexpand(true);
    entry.set_visibility(false); // Start hidden
    entry.set_invisible_char(Some('•'));

    // Show/hide button
    let reveal_button = Button::new();
    reveal_button.set_label("👁");
    reveal_button.add_css_class("flat");
    reveal_button.set_tooltip_text(Some("Show/Hide Password"));

    // Connect entry changes to account data
    let account_rc_clone = account_rc.clone();
    entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        let mut account = account_rc_clone.borrow_mut();
        account.password = text;
    });

    // Connect reveal button
    let entry_clone = entry.clone();
    let is_revealed = Rc::new(RefCell::new(false));
    let is_revealed_clone = is_revealed.clone();
    reveal_button.connect_clicked(move |_| {
        let mut revealed = is_revealed_clone.borrow_mut();
        *revealed = !*revealed;
        entry_clone.set_visibility(*revealed);
    });

    password_container.append(&entry);
    password_container.append(&reveal_button);

    row_box.append(&label);
    row_box.append(&password_container);

    row_box
}

/// Shows the new account creation view
pub fn show_new_account_view(content_area: &Box, vault_name: &str) {
    clear_content(content_area);

    // Create main container with scrolling
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    // Header
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

    // Create a new account with default values
    let new_account = Rc::new(RefCell::new(Account::default()));

    // Form fields
    let form_box = Box::new(Orientation::Vertical, 16);

    // Account name
    let name_row = create_editable_field_row("Account Name", "", &new_account, "name");
    form_box.append(&name_row);

    // Account type
    let type_row = create_editable_field_row(
        "Account Type",
        "Password Account",
        &new_account,
        "account_type",
    );
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

    // Action buttons
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_top(24);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    // Connect cancel functionality
    let content_area_clone2 = content_area.clone();
    let vault_name_clone2 = vault_name.to_string();
    cancel_button.connect_clicked(move |_| {
        show_vault_view(&content_area_clone2, &vault_name_clone2);
    });

    let create_button = Button::new();
    create_button.set_label("Create Account");
    create_button.add_css_class("suggested-action");
    create_button.set_size_request(120, -1);

    // Add create functionality
    let account_rc_clone = new_account.clone();
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    create_button.connect_clicked(move |_| {
        let account = account_rc_clone.borrow();

        if account.name.is_empty() {
            eprintln!("Account name is required");
            return;
        }

        match create_account(&vault_name_clone, &*account) {
            Ok(()) => {
                println!("Account created successfully");
                // Return to vault view
                show_vault_view(&content_area_clone, &vault_name_clone);
            }
            Err(e) => {
                eprintln!("Failed to create account: {}", e);
            }
        }
    });

    actions_box.append(&cancel_button);
    actions_box.append(&create_button);
    main_box.append(&actions_box);

    scrolled_window.set_child(Some(&main_box));
    content_area.append(&scrolled_window);
}

/// Creates an editable field row for account creation/editing
fn create_editable_field_row(
    label_text: &str,
    initial_value: &str,
    account_rc: &Rc<RefCell<Account>>,
    field_name: &str,
) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 12);
    row_box.set_halign(gtk4::Align::Fill);

    // Label
    let label = Label::new(Some(label_text));
    label.add_css_class("dim-label");
    label.set_halign(gtk4::Align::Start);
    label.set_size_request(150, -1);

    // Entry field
    let entry = Entry::new();
    entry.set_text(initial_value);
    entry.set_hexpand(true);

    // Connect entry changes to account data
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
            "password" => account.password = text,
            "notes" => account.notes = text,
            _ => {}
        }
    });

    row_box.append(&label);
    row_box.append(&entry);

    row_box
}

/// Shows the vault creation view
pub fn show_new_vault_view(content_area: &Box) {
    clear_content(content_area);

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_top(48);
    main_box.set_margin_bottom(48);
    main_box.set_margin_start(48);
    main_box.set_margin_end(48);
    main_box.set_halign(gtk4::Align::Center);
    main_box.set_valign(gtk4::Align::Center);

    // Header
    let title = Label::new(Some("Create New Vault"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Center);

    let subtitle = Label::new(Some("Enter a name for your new vault and select a GPG key"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Center);

    main_box.append(&title);
    main_box.append(&subtitle);

    // Form
    let form_box = Box::new(Orientation::Vertical, 16);
    form_box.set_size_request(400, -1);

    // Vault name
    let name_label = Label::new(Some("Vault Name"));
    name_label.set_halign(gtk4::Align::Start);
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Enter vault name"));

    // GPG recipient
    let recipient_label = Label::new(Some("GPG Key ID"));
    recipient_label.set_halign(gtk4::Align::Start);
    let recipient_entry = Entry::new();
    recipient_entry.set_placeholder_text(Some("Enter GPG key ID or email"));

    form_box.append(&name_label);
    form_box.append(&name_entry);
    form_box.append(&recipient_label);
    form_box.append(&recipient_entry);

    main_box.append(&form_box);

    // Action buttons
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_top(24);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    // Connect cancel functionality
    let content_area_clone2 = content_area.clone();
    cancel_button.connect_clicked(move |_| {
        show_home_view(&content_area_clone2);
    });

    let create_button = Button::new();
    create_button.set_label("Create Vault");
    create_button.add_css_class("suggested-action");
    create_button.set_size_request(120, -1);

    // Add create functionality
    let content_area_clone = content_area.clone();
    create_button.connect_clicked(move |_| {
        let vault_name = name_entry.text().to_string();
        let recipient = recipient_entry.text().to_string();

        if vault_name.is_empty() || recipient.is_empty() {
            eprintln!("Both vault name and GPG key are required");
            return;
        }

        match create_vault(&vault_name, &recipient) {
            Ok(()) => {
                println!("Vault created successfully");
                // Try to refresh the sidebar
                if let Some(window) = content_area_clone
                    .root()
                    .and_then(|root| root.downcast::<gtk4::Window>().ok())
                {
                    if let Some(child) = window.child() {
                        if let Some(paned) = child.downcast::<gtk4::Paned>().ok() {
                            if let Some(sidebar) = paned
                                .start_child()
                                .and_then(|child| child.downcast::<Box>().ok())
                            {
                                refresh_vaults_section(&sidebar, &content_area_clone);
                            }
                        }
                    }
                }
                // Show the new vault
                show_vault_view(&content_area_clone, &vault_name);
            }
            Err(e) => {
                eprintln!("Failed to create vault: {}", e);
            }
        }
    });

    actions_box.append(&cancel_button);
    actions_box.append(&create_button);
    main_box.append(&actions_box);

    content_area.append(&main_box);
}
