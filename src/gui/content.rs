use adw::ButtonContent;
use adw::prelude::*;

use crate::gui::sidebar::refresh_vaults_section;
use crate::password::{PasswordConfig, generate_password};
use crate::totp::{is_totp_required, verify_totp_code};
use crate::vault::{
    Account, Locations, create_account, create_vault, get_full_account_details, read_directory,
    update_account, warm_up_gpg,
};
use gtk4::pango::EllipsizeMode;
use gtk4::{
    Adjustment, Box, Button, CheckButton, Dialog, Entry, FlowBox, Frame, Label, Orientation,
    PolicyType, ResponseType, Scale, ScrolledWindow, SelectionMode, Separator, SpinButton,
    TextBuffer, TextView,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// Shows the home view in the content area
pub fn show_home_view(content_area: &Box) {
    clear_content(content_area);

    // Create scrolled window for the home content
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_margin_top(32);
    main_box.set_margin_bottom(32);
    main_box.set_margin_start(48);
    main_box.set_margin_end(48);

    // Welcome section
    let welcome_section = create_welcome_section();
    main_box.append(&welcome_section);

    // Statistics section
    let stats_section = create_statistics_section();
    main_box.append(&stats_section);

    // Quick actions section
    let actions_section = create_quick_actions_section(content_area);
    main_box.append(&actions_section);

    scrolled_window.set_child(Some(&main_box));
    content_area.append(&scrolled_window);
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
/// Handles vault opening with gate logic (TOTP verification and GPG warm-up)
pub fn open_vault_with_gate(content_area: &Box, vault_name: &str) {
    let vault_name = vault_name.to_string();
    let content_area = content_area.clone();

    // Check if TOTP is required for this vault
    if is_totp_required(&vault_name) {
        show_totp_verification_dialog(&content_area, &vault_name);
    } else {
        // No TOTP required, proceed with gate warm-up
        proceed_with_gate_warmup(&content_area, &vault_name);
    }
}

/// Shows TOTP verification dialog
fn show_totp_verification_dialog(content_area: &Box, vault_name: &str) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Two-Factor Authentication"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 200);

    // Get the main window to set as transient parent
    if let Some(window) = content_area
        .root()
        .and_then(|root| root.downcast::<gtk4::Window>().ok())
    {
        dialog.set_transient_for(Some(&window));
    }

    let content_box = Box::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    // Title and description
    let title = Label::new(Some("Enter 2FA Code"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Center);

    let description = Label::new(Some(&format!(
        "Enter the 6-digit code from your authenticator app for vault '{}'",
        vault_name
    )));
    description.set_wrap(true);
    description.set_halign(gtk4::Align::Center);
    description.add_css_class("dim-label");

    // TOTP code entry
    let code_entry = Entry::new();
    code_entry.set_placeholder_text(Some("000000"));
    code_entry.set_max_length(8);
    code_entry.set_input_purpose(gtk4::InputPurpose::Digits);
    code_entry.set_halign(gtk4::Align::Center);
    code_entry.set_size_request(150, -1);

    // Error label (initially hidden)
    let error_label = Label::new(None);
    error_label.add_css_class("error");
    error_label.set_halign(gtk4::Align::Center);
    error_label.set_visible(false);

    content_box.append(&title);
    content_box.append(&description);
    content_box.append(&code_entry);
    content_box.append(&error_label);

    dialog.set_child(Some(&content_box));

    // Add buttons
    dialog.add_button("Cancel", ResponseType::Cancel);
    let verify_button = dialog.add_button("Verify", ResponseType::Accept);
    verify_button.add_css_class("suggested-action");

    // Set up response handling
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    let code_entry_clone = code_entry.clone();
    let error_label_clone = error_label.clone();

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let code = code_entry_clone.text().to_string();

            match verify_totp_code(&vault_name_clone, &code) {
                Ok(true) => {
                    // TOTP verification successful
                    dialog.close();
                    proceed_with_gate_warmup(&content_area_clone, &vault_name_clone);
                }
                Ok(false) => {
                    // Invalid TOTP code
                    error_label_clone.set_text("Invalid code. Please try again.");
                    error_label_clone.set_visible(true);
                    code_entry_clone.set_text("");
                    return; // Don't close dialog
                }
                Err(e) => {
                    // Error during verification
                    error_label_clone.set_text(&format!("Error: {}", e));
                    error_label_clone.set_visible(true);
                    return; // Don't close dialog
                }
            }
        } else {
            // Cancel pressed
            dialog.close();
        }
    });

    // Focus the entry and allow Enter to submit
    code_entry.grab_focus();
    let dialog_clone = dialog.clone();
    code_entry.connect_activate(move |_| {
        dialog_clone.response(ResponseType::Accept);
    });

    dialog.present();
}

/// Proceeds with gate warm-up and then shows the vault view
fn proceed_with_gate_warmup(content_area: &Box, vault_name: &str) {
    // Attempt to warm up GPG by decrypting the gate file
    match warm_up_gpg(vault_name) {
        Ok(()) => {
            // Gate warm-up successful, show the vault
            show_vault_view(content_area, vault_name);
        }
        Err(e) => {
            // Gate warm-up failed, show error
            show_error_message(
                content_area,
                "Failed to Access Vault",
                &format!("Could not decrypt vault gate file: {}", e),
            );
        }
    }
}

/// Shows an error message in the content area
fn show_error_message(content_area: &Box, title: &str, message: &str) {
    clear_content(content_area);

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_top(48);
    main_box.set_margin_bottom(48);
    main_box.set_margin_start(48);
    main_box.set_margin_end(48);
    main_box.set_halign(gtk4::Align::Center);
    main_box.set_valign(gtk4::Align::Center);

    let error_title = Label::new(Some(title));
    error_title.add_css_class("title-1");
    error_title.set_halign(gtk4::Align::Center);

    let error_message = Label::new(Some(message));
    error_message.set_wrap(true);
    error_message.set_max_width_chars(60);
    error_message.set_halign(gtk4::Align::Center);
    error_message.add_css_class("dim-label");

    main_box.append(&error_title);
    main_box.append(&error_message);

    content_area.append(&main_box);
}

pub fn show_vault_view(content_area: &Box, vault_name: &str) {
    clear_content(content_area);

    // Increment usage count for this vault
    increment_vault_usage(vault_name);

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

    let main_box = Box::new(Orientation::Vertical, 0);
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
    let additional_fields_section = create_additional_fields_section(&account_rc, edit_mode);
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
    section.add_css_class("account-section");
    section.set_margin_top(16);

    // Section header
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(16);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

    // Section title
    let title = Label::new(Some("Account Details"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);

    header_box.append(&title);
    section.append(&header_box);

    // Details grid
    let details_box = Box::new(Orientation::Vertical, 12);
    details_box.set_margin_bottom(16);
    details_box.set_margin_start(24);
    details_box.set_margin_end(24);
    details_box.set_halign(gtk4::Align::Center);

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

#[allow(clippy::too_many_lines)]
/// Creates the password section
fn create_password_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 16);
    section.add_css_class("account-section");
    section.set_margin_top(16);

    // Section title with generate button
    let header_box = Box::new(Orientation::Horizontal, 12);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

    let title = Label::new(Some("Password"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);

    let generate_button = Button::new();
    generate_button.set_label("Generate New");
    generate_button.add_css_class("flat");

    header_box.append(&title);
    if edit_mode {
        header_box.append(&generate_button);
    }

    section.append(&header_box);

    // Password field with reveal/copy buttons
    let password_box = Box::new(Orientation::Horizontal, 8);
    password_box.set_margin_bottom(20);
    password_box.set_margin_start(24);
    password_box.set_margin_end(24);
    password_box.set_halign(gtk4::Align::Center);

    let password_entry = Entry::new();
    let account = account_rc.borrow();

    // In edit mode, show the actual password; in view mode, show masked
    if edit_mode {
        account.password.with_exposed(|password| {
            password_entry.set_text(password);
        });
    } else {
        let masked_password = account.password.masked(8);
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
            account.password.update(text);
        });
    }

    // Connect generate button functionality (only in edit mode)
    if edit_mode {
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

    // Add copy functionality
    let account_rc_copy = account_rc.clone();
    copy_button.connect_clicked(move |button| {
        let account = account_rc_copy.borrow();
        let display = button.display();
        let clipboard = display.clipboard();

        // Use secure clipboard exposure with automatic cleanup
        let password_copy = account.password.expose_for_clipboard();
        clipboard.set_text(&password_copy);

        // The SecureClipboardString will be automatically zeroized when dropped

        // Schedule clipboard clearing after 30 seconds for security
        let clipboard_clone = clipboard.clone();
        glib::timeout_add_seconds_local(30, move || {
            clipboard_clone.set_text("");
            println!("Clipboard cleared for security");
            glib::ControlFlow::Break
        });

        println!("Password copied to clipboard (will be cleared in 30 seconds)");
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
fn create_additional_fields_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 16);
    section.add_css_class("account-section");
    section.set_margin_top(16);

    // Section title with add button
    let header_box = Box::new(Orientation::Vertical, 12);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

    let title = Label::new(Some("Additional Fields"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);

    let add_button = Button::new();
    add_button.set_label("Add Field");
    add_button.add_css_class("flat");

    header_box.append(&title);
    if edit_mode {
        header_box.append(&add_button);
    }

    section.append(&header_box);

    // Additional fields from account data
    let fields_box = Box::new(Orientation::Vertical, 12);
    fields_box.set_margin_bottom(20);
    fields_box.set_margin_start(24);
    fields_box.set_margin_end(24);
    fields_box.set_halign(gtk4::Align::Center);

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
    section.add_css_class("account-section");
    section.set_margin_top(16);

    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

    let title = Label::new(Some("Notes"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);

    header_box.append(&title);
    section.append(&header_box);

    let notes_box = Box::new(Orientation::Vertical, 12);
    notes_box.set_margin_bottom(20);
    notes_box.set_margin_start(24);
    notes_box.set_margin_end(24);
    notes_box.set_halign(gtk4::Align::Center);

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

    notes_box.append(&notes_entry);
    section.append(&notes_box);
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
                println!("Clipboard cleared for security");
                glib::ControlFlow::Break
            });

            println!(
                "Copied '{}' to clipboard (will be cleared in 60 seconds)",
                value_text_owned
            );
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

    // Generate button
    let generate_button = Button::new();
    generate_button.set_label("Generate");
    generate_button.add_css_class("flat");
    generate_button.set_tooltip_text(Some("Generate Password"));

    // Show/hide button
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

    // Connect generate button
    let entry_gen = entry.clone();
    let account_rc_gen = account_rc.clone();
    generate_button.connect_clicked(move |_| {
        show_password_generator_dialog(&entry_gen, &account_rc_gen);
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
    password_container.append(&generate_button);
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
            "password" => account.password.update(text),
            "notes" => account.notes = text,
            _ => {}
        }
    });

    row_box.append(&label);
    row_box.append(&entry);

    row_box
}

/// Creates the welcome section for the home view
fn create_welcome_section() -> Box {
    let section_box = Box::new(Orientation::Vertical, 16);
    section_box.set_halign(gtk4::Align::Center);

    // App title and description
    let title = Label::new(Some("Welcome to FMP"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Center);

    let subtitle = Label::new(Some("Secure Password Manager"));
    subtitle.add_css_class("title-3");
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Center);

    let description = Label::new(Some(
        "Manage your passwords securely with GPG encryption.\nCreate vaults to organize your accounts and keep your data safe.",
    ));
    description.add_css_class("body");
    description.set_halign(gtk4::Align::Center);
    description.set_wrap(true);
    description.set_max_width_chars(60);

    section_box.append(&title);
    section_box.append(&subtitle);
    section_box.append(&description);

    section_box
}

/// Creates the statistics section showing vault and account counts
fn create_statistics_section() -> Box {
    let section_container = Box::new(Orientation::Vertical, 0);
    section_container.add_css_class("home-section");
    section_container.set_margin_top(16);

    // Section header
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(16);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

    let title = Label::new(Some("📊 Overview"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);

    let subtitle = Label::new(Some("Your vault statistics at a glance"));
    subtitle.add_css_class("dim-label");
    subtitle.add_css_class("caption");
    subtitle.set_halign(gtk4::Align::Start);

    header_box.append(&title);
    header_box.append(&subtitle);
    section_container.append(&header_box);

    let stats_box = Box::new(Orientation::Horizontal, 32);
    stats_box.set_margin_bottom(20);
    stats_box.set_margin_start(24);
    stats_box.set_margin_end(24);
    stats_box.set_halign(gtk4::Align::Center);

    // Get vault statistics
    let vaults = crate::gui::sidebar::get_available_vaults();
    let vault_count = vaults.len();
    let total_accounts: usize = vaults
        .iter()
        .map(|vault_name| get_available_accounts(vault_name).len())
        .sum();

    // Vault count card
    let vault_card = create_stat_card("Vaults", &vault_count.to_string(), "🔐");
    stats_box.append(&vault_card);

    // Account count card
    let account_card = create_stat_card("Accounts", &total_accounts.to_string(), "👤");
    stats_box.append(&account_card);

    // Most used vault
    let most_used = get_most_used_vault();
    let most_used_card = create_stat_card("Most Used", &most_used, "⭐");
    stats_box.append(&most_used_card);

    section_container.append(&stats_box);
    section_container
}

/// Creates a statistics card
fn create_stat_card(title: &str, value: &str, icon: &str) -> Box {
    let card_box = Box::new(Orientation::Vertical, 8);
    card_box.set_size_request(120, 80);
    card_box.set_halign(gtk4::Align::Center);
    card_box.set_valign(gtk4::Align::Center);

    let icon_label = Label::new(Some(icon));
    icon_label.add_css_class("title-2");
    icon_label.set_halign(gtk4::Align::Center);

    let value_label = Label::new(Some(value));
    value_label.add_css_class("title-2");
    value_label.set_halign(gtk4::Align::Center);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("caption");
    title_label.add_css_class("dim-label");
    title_label.set_halign(gtk4::Align::Center);

    card_box.append(&icon_label);
    card_box.append(&value_label);
    card_box.append(&title_label);

    card_box
}

/// Creates the quick actions section
fn create_quick_actions_section(content_area: &Box) -> Box {
    let section_container = Box::new(Orientation::Vertical, 0);
    section_container.add_css_class("home-section");
    section_container.set_margin_top(16);

    // Section header
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(16);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

    let title = Label::new(Some("⚡ Quick Actions"));
    title.add_css_class("title-3");
    title.set_halign(gtk4::Align::Start);

    let subtitle = Label::new(Some("Get started with common tasks"));
    subtitle.add_css_class("dim-label");
    subtitle.add_css_class("caption");
    subtitle.set_halign(gtk4::Align::Start);

    header_box.append(&title);
    header_box.append(&subtitle);
    section_container.append(&header_box);

    let actions_box = Box::new(Orientation::Horizontal, 16);
    actions_box.set_margin_bottom(20);
    actions_box.set_margin_start(24);
    actions_box.set_margin_end(24);
    actions_box.set_halign(gtk4::Align::Center);

    // Create new vault button
    let create_vault_button = Button::new();
    create_vault_button.set_label("Create New Vault");
    create_vault_button.add_css_class("suggested-action");
    create_vault_button.set_size_request(160, 40);

    let content_area_clone = content_area.clone();
    create_vault_button.connect_clicked(move |_| {
        show_create_vault_view(&content_area_clone);
    });

    actions_box.append(&create_vault_button);

    section_container.append(&actions_box);
    section_container
}

/// Shows the create vault view
fn show_create_vault_view(content_area: &Box) {
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
                // Show the new vault with gate logic
                open_vault_with_gate(&content_area_clone, &vault_name);
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

/// Gets the usage count for a specific vault
fn get_vault_usage_count(vault_name: &str) -> u32 {
    let stats_file = get_vault_stats_file();

    if let Ok(content) = fs::read_to_string(&stats_file) {
        for line in content.lines() {
            if let Some((name, count_str)) = line.split_once(':') {
                if name == vault_name {
                    return count_str.parse().unwrap_or(0);
                }
            }
        }
    }

    0
}

/// Increments the usage count for a vault
pub fn increment_vault_usage(vault_name: &str) {
    let stats_file = get_vault_stats_file();
    let mut usage_counts = HashMap::new();

    // Read existing stats
    if let Ok(content) = fs::read_to_string(&stats_file) {
        for line in content.lines() {
            if let Some((name, count_str)) = line.split_once(':') {
                if let Ok(count) = count_str.parse::<u32>() {
                    usage_counts.insert(name.to_string(), count);
                }
            }
        }
    }

    // Increment count for this vault
    let current_count = usage_counts.get(vault_name).unwrap_or(&0);
    usage_counts.insert(vault_name.to_string(), current_count + 1);

    // Write back to file
    let mut content = String::new();
    for (name, count) in usage_counts {
        content.push_str(&format!("{}:{}\n", name, count));
    }

    if let Err(e) = fs::write(&stats_file, content) {
        eprintln!("Failed to write vault stats: {}", e);
    }
}

/// Gets the path to the vault statistics file
fn get_vault_stats_file() -> PathBuf {
    let locations = crate::vault::Locations::new("", "");
    locations.fmp.join("vault_stats.txt")
}

/// Gets the most used vault name
fn get_most_used_vault() -> String {
    let stats_file = get_vault_stats_file();
    let mut max_count = 0;
    let mut most_used = "None".to_string();

    if let Ok(content) = fs::read_to_string(&stats_file) {
        for line in content.lines() {
            if let Some((name, count_str)) = line.split_once(':') {
                if let Ok(count) = count_str.parse::<u32>() {
                    if count > max_count {
                        max_count = count;
                        most_used = name.to_string();
                    }
                }
            }
        }
    }

    if max_count == 0 {
        "None".to_string()
    } else {
        most_used
    }
}

/// Shows the password generator window
pub fn show_password_generator_window() {
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
    let password_display_section = create_password_display_section(&config, None, None, None);
    content_box.append(&password_display_section);

    dialog.set_child(Some(&content_box));

    // Add buttons
    dialog.add_button("Close", ResponseType::Close);

    // Set up response handling
    dialog.connect_response(move |dialog, response| match response {
        ResponseType::Close => {
            dialog.close();
        }
        _ => {}
    });

    dialog.present();
}

/// Shows the password generator dialog and updates the provided entry field and account
fn show_password_generator_dialog(entry: &Entry, account_rc: &Rc<RefCell<Account>>) {
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

/// Helper function to find the password text view in the display section
fn find_password_text_view(section: &Box) -> Option<TextView> {
    let mut child = section.first_child();
    while let Some(widget) = child {
        if let Ok(scrolled) = widget.clone().downcast::<ScrolledWindow>() {
            if let Some(text_view) = scrolled.child().and_then(|c| c.downcast::<TextView>().ok()) {
                return Some(text_view);
            }
        }
        child = widget.next_sibling();
    }
    None
}
