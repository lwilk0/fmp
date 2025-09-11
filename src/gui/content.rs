use crate::{
    gui::{
        dialogs::{
            show_add_field_dialog, show_backup_vault_dialog, show_confirmation_dialog,
            show_delete_backup_dialog, show_delete_field_dialog, show_delete_vault_dialog,
            show_edit_field_dialog, show_password_generator_dialog, show_rename_account_dialog,
            show_rename_vault_dialog, show_restore_vault_dialog,
            show_standalone_password_generator_dialog, show_totp_authentication_dialog,
            show_totp_management_dialog, show_totp_setup_dialog,
        },
        widgets::loading_spinner::{
            LoadingOverlay, create_loading_button, set_button_loading_state,
        },
    },
    storage::filesystem::{backup_exists, read_directory},
    totp::{is_totp_enabled, is_totp_required},
    vault::{
        Account, Locations, create_account, create_vault, delete_account, get_full_account_details,
        update_account, warm_up_gpg,
    },
};
use adw::{ActionRow, ButtonContent, Clamp, PreferencesGroup, prelude::*};

use gtk4::{
    Box, Button, Entry, Label, Orientation, PolicyType, ScrolledWindow, Separator,
    pango::EllipsizeMode,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

// Global counter for tracking vault loading operations
static VAULT_LOADING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Shows the home view in the content area
pub fn show_home_view(content_area: &Box) {
    clear_content(content_area);

    // Cancel any pending vault loading operations
    VAULT_LOADING_COUNTER.fetch_add(1, Ordering::SeqCst);

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let clamp = Clamp::new();
    clamp.set_maximum_size(800);
    clamp.set_tightening_threshold(600);

    let main_box = Box::new(Orientation::Vertical, 16);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    let welcome_section = create_welcome_section();
    main_box.append(&welcome_section);

    let stats_section = create_statistics_section();
    main_box.append(&stats_section);

    let actions_section = create_quick_actions_section(content_area);
    main_box.append(&actions_section);

    let recent_section = create_recent_vaults_section(content_area);
    main_box.append(&recent_section);

    clamp.set_child(Some(&main_box));
    scrolled_window.set_child(Some(&clamp));
    content_area.append(&scrolled_window);
}

pub fn show_vault_view(content_area: &Box, vault_name: &str) {
    clear_content(content_area);

    // Increment the loading counter to cancel any previous loading operations
    let current_loading_id = VAULT_LOADING_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

    increment_vault_usage(vault_name);
    record_recent_vault(vault_name);

    let loading_overlay = Rc::new(LoadingOverlay::new());
    content_area.append(loading_overlay.widget());

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let clamp = Clamp::new();
    clamp.set_maximum_size(800);
    clamp.set_tightening_threshold(600);

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    let header_group = PreferencesGroup::new();
    header_group.set_title(vault_name);
    header_group.set_description(Some(
        "Secure vault for managing your accounts and passwords",
    ));

    main_box.append(&header_group);

    let totp_section = create_totp_management_section(content_area, vault_name);
    main_box.append(&totp_section);

    let vault_management_section = create_vault_management_section(content_area, vault_name);
    main_box.append(&vault_management_section);

    loading_overlay.show("Loading accounts...");

    // Load accounts asynchronously
    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    let main_box_clone = main_box.clone();
    let scrolled_window_clone = scrolled_window.clone();
    let clamp_clone = clamp.clone();
    let loading_overlay_clone = loading_overlay.clone();

    glib::idle_add_local(move || {
        // Check if this loading operation is still the current one
        let current_counter = VAULT_LOADING_COUNTER.load(Ordering::SeqCst);
        if current_loading_id != current_counter {
            // This loading operation has been superseded, cancel it
            loading_overlay_clone.hide();
            return glib::ControlFlow::Break;
        }

        let accounts_section = create_accounts_grid(&content_area_clone, &vault_name_clone);
        main_box_clone.append(&accounts_section);

        clamp_clone.set_child(Some(&main_box_clone));
        scrolled_window_clone.set_child(Some(&clamp_clone));
        content_area_clone.append(&scrolled_window_clone);

        // Hide loading overlay
        loading_overlay_clone.hide();

        glib::ControlFlow::Break
    });
}

/// Shows the account view with edit mode option
pub fn show_account_view_with_mode(
    content_area: &Box,
    vault_name: &str,
    account_name: &str,
    edit_mode: bool,
) {
    clear_content(content_area);

    let account_data = match get_full_account_details(vault_name, account_name) {
        Ok(account) => Some(account),
        Err(e) => {
            eprintln!("Failed to load account data: {e}");
            None
        }
    };

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
        Account {
            name: account_name.to_string(),
            ..Default::default()
        }
    })));

    let header_section = create_account_header(
        &account_rc,
        content_area,
        vault_name,
        account_name,
        edit_mode,
    );

    main_box.append(&header_section);

    let details_section = create_account_details_section(&account_rc, edit_mode);
    main_box.append(&details_section);

    let password_section = create_password_section(&account_rc, edit_mode);
    main_box.append(&password_section);

    let additional_fields_section =
        create_additional_fields_section(&account_rc, edit_mode, content_area, vault_name);
    main_box.append(&additional_fields_section);

    let notes_section = create_notes_section(&account_rc, edit_mode);
    main_box.append(&notes_section);

    if edit_mode {
        let actions_section = create_account_actions_section(&account_rc, vault_name, content_area);
        main_box.append(&actions_section);
    }

    scrolled_window.set_child(Some(&main_box));
    content_area.append(&scrolled_window);
}

/// Shows the new account creation view
pub fn show_new_account_view(content_area: &Box, vault_name: &str) {
    clear_content(content_area);

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

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

    // Account type
    let type_row = create_editable_field_row(
        "Account Type",
        "Password Account",
        &new_account,
        "account_type",
    );    // Account name

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

    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_top(24);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    let content_area_clone2 = content_area.clone();
    let vault_name_clone2 = vault_name.to_string();
    cancel_button.connect_clicked(move |_| {
        show_vault_view(&content_area_clone2, &vault_name_clone2);
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
                show_vault_view(&content_area_clone, &vault_name_clone);
            }
            Err(e) => {
                eprintln!("Failed to create account: {e}");
            }
        }
    });

    actions_box.append(&cancel_button);
    actions_box.append(&create_button);
    main_box.append(&actions_box);

    scrolled_window.set_child(Some(&main_box));
    content_area.append(&scrolled_window);
}

/// Creates the welcome section for the home view
fn create_welcome_section() -> Box {
    let welcome_box = Box::new(Orientation::Vertical, 8);
    welcome_box.add_css_class("welcome-section");
    welcome_box.set_halign(gtk4::Align::Center);
    welcome_box.set_valign(gtk4::Align::Start);

    let title = Label::new(Some("Welcome to FMP"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Center);
    title.set_margin_top(8);
    title.set_margin_bottom(4);
    welcome_box.append(&title);

    welcome_box
}

/// Creates the recent vaults section for the home view (bottom panel)
fn create_recent_vaults_section(content_area: &Box) -> PreferencesGroup {
    use gtk4::Align;
    let group = PreferencesGroup::new();
    group.set_title("Recently Accessed Vaults");
    group.set_description(Some("Open a vault you used recently"));

    let recent = get_recent_vaults(5);
    if recent.is_empty() {
        let row = ActionRow::new();
        row.set_title("No recent vaults yet");
        row.set_subtitle("Open a vault to see it here");
        row.set_activatable(false);
        group.add(&row);
        return group;
    }

    for name in recent {
        let row = ActionRow::new();
        row.set_title(&name);
        row.set_subtitle("Click to open");
        row.set_activatable(true);

        let open_btn = Button::new();
        open_btn.set_label("Open");
        open_btn.add_css_class("suggested-action");
        open_btn.set_valign(Align::Center);
        row.add_suffix(&open_btn);
        row.set_activatable_widget(Some(&open_btn));

        let content_area_clone = content_area.clone();
        let name_clone = name.clone();
        open_btn.connect_clicked(move |_| {
            show_vault_view(&content_area_clone, &name_clone);
        });

        group.add(&row);
    }

    group
}

/// Creates the statistics section showing vault and account counts
fn create_statistics_section() -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Overview");
    group.set_description(Some("Your vault statistics at a glance"));

    let vaults = crate::gui::sidebar::get_available_vaults();
    let vault_count = vaults.len();
    let total_accounts: usize = vaults
        .iter()
        .map(|vault_name| get_available_accounts(vault_name).len())
        .sum();

    let vault_row = ActionRow::new();
    vault_row.set_title("Vaults");
    vault_row.set_subtitle("Total number of vaults");

    let vault_label = Label::new(Some(&vault_count.to_string()));
    vault_label.add_css_class("title-3");
    vault_row.add_suffix(&vault_label);
    group.add(&vault_row);

    let account_row = ActionRow::new();
    account_row.set_title("Accounts");
    account_row.set_subtitle("Total number of accounts");

    let account_label = Label::new(Some(&total_accounts.to_string()));
    account_label.add_css_class("title-3");
    account_row.add_suffix(&account_label);
    group.add(&account_row);

    let most_used = get_most_used_vault();

    let most_used_row = ActionRow::new();
    most_used_row.set_title("Most Used Vault");
    most_used_row.set_subtitle("Your frequently accessed vault");

    let most_used_label = Label::new(Some(&most_used));
    most_used_label.add_css_class("title-3");
    most_used_row.add_suffix(&most_used_label);
    group.add(&most_used_row);

    group
}

/// Creates the quick actions section
fn create_quick_actions_section(content_area: &Box) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Quick Actions");
    group.set_description(Some("Get started with common tasks"));

    // Create new vault row
    let create_vault_row = ActionRow::new();
    create_vault_row.set_title("Create New Vault");
    create_vault_row.set_subtitle("Set up a new secure vault for your passwords");
    create_vault_row.set_activatable(true);

    let create_vault_button = Button::new();
    create_vault_button.set_label("Create");
    create_vault_button.add_css_class("suggested-action");
    create_vault_button.set_valign(gtk4::Align::Center);
    create_vault_row.add_suffix(&create_vault_button);
    create_vault_row.set_activatable_widget(Some(&create_vault_button));

    let content_area_clone = content_area.clone();
    create_vault_button.connect_clicked(move |_| {
        show_create_vault_view(&content_area_clone);
    });

    group.add(&create_vault_row);

    let password_row = ActionRow::new();
    password_row.set_title("Generate Password");
    password_row.set_subtitle("Create a secure password with customizable options");
    password_row.set_activatable(true);

    let password_button = Button::new();
    password_button.set_label("Generate");
    password_button.add_css_class("suggested-action");
    password_button.set_valign(gtk4::Align::Center);
    password_row.add_suffix(&password_button);
    password_row.set_activatable_widget(Some(&password_button));

    password_button.connect_clicked(move |_| {
        show_standalone_password_generator_dialog();
    });

    group.add(&password_row);

    group
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

    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_top(24);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    let content_area_clone2 = content_area.clone();
    cancel_button.connect_clicked(move |_| {
        show_home_view(&content_area_clone2);
    });

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
fn create_accounts_grid(content_area: &Box, vault_name: &str) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Accounts");
    group.set_description(Some("Manage your account credentials"));

    let all_accounts = get_available_accounts(vault_name);

    if all_accounts.is_empty() {
        let empty_row = ActionRow::new();
        empty_row.set_title("No accounts yet");
        empty_row.set_subtitle("Create your first account to get started");
        empty_row.set_activatable(true);

        let add_button = Button::new();
        add_button.set_label("Add Account");
        add_button.add_css_class("suggested-action");
        add_button.set_valign(gtk4::Align::Center);
        empty_row.add_suffix(&add_button);
        empty_row.set_activatable_widget(Some(&add_button));

        let content_area_clone = content_area.clone();
        let vault_name_clone = vault_name.to_string();
        add_button.connect_clicked(move |_| {
            show_new_account_view(&content_area_clone, &vault_name_clone);
        });

        group.add(&empty_row);
    } else {
        let add_row = ActionRow::new();
        add_row.set_title("Add New Account");
        add_row.set_subtitle("Create a new account entry");
        add_row.set_activatable(true);

        let add_button = Button::new();
        add_button.set_label("Add");
        add_button.add_css_class("suggested-action");
        add_button.set_valign(gtk4::Align::Center);
        add_row.add_suffix(&add_button);
        add_row.set_activatable_widget(Some(&add_button));

        let content_area_clone = content_area.clone();
        let vault_name_clone = vault_name.to_string();
        add_button.connect_clicked(move |_| {
            show_new_account_view(&content_area_clone, &vault_name_clone);
        });

        group.add(&add_row);

        // Account rows
        for account_name in all_accounts {
            let account_row = create_account_row(account_name.as_str(), content_area, vault_name);
            group.add(&account_row);
        }
    }

    group
}

/// Creates an account row for the preferences group
fn create_account_row(account_name: &str, content_area: &Box, vault_name: &str) -> ActionRow {
    let row = ActionRow::new();
    row.set_title(account_name);
    row.set_subtitle("Password Account");
    row.set_activatable(true);

    let view_button = Button::new();
    view_button.set_label("View");
    view_button.add_css_class("flat");
    view_button.set_valign(gtk4::Align::Center);
    row.add_suffix(&view_button);
    row.set_activatable_widget(Some(&view_button));

    let account_name_clone = account_name.to_string();
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    view_button.connect_clicked(move |_| {
        show_account_view_with_mode(&content_area_clone, &vault_name_clone, &account_name_clone, false);
    });

    row
}

/// Creates the TOTP (2FA) management section for the vault view
fn create_totp_management_section(content_area: &Box, vault_name: &str) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Two-Factor Authentication");
    group.set_description(Some("Secure your vault with TOTP authentication"));

    let is_enabled = is_totp_enabled(vault_name);

    if is_enabled {
        // 2FA is enabled - show status row
        let status_row = ActionRow::new();
        status_row.set_title("TOTP Status");
        status_row.set_subtitle("Two-factor authentication is enabled and active");
        status_row.set_activatable(true);

        let status_label = Label::new(Some("✅ Enabled"));
        status_label.add_css_class("success");
        status_row.add_suffix(&status_label);

        let manage_button = Button::new();
        manage_button.set_label("Manage");
        manage_button.add_css_class("flat");
        manage_button.set_valign(gtk4::Align::Center);
        status_row.add_suffix(&manage_button);
        status_row.set_activatable_widget(Some(&manage_button));

        // Connect manage button
        let content_area_clone = content_area.clone();
        let vault_name_clone = vault_name.to_string();
        manage_button.connect_clicked(move |_| {
            show_totp_management_dialog(&vault_name_clone, &content_area_clone);
        });

        group.add(&status_row);
    } else {
        // 2FA is not enabled - show setup row
        let setup_row = ActionRow::new();
        setup_row.set_title("Enable Two-Factor Authentication");
        setup_row.set_subtitle("Add an extra layer of security to your vault");
        setup_row.set_activatable(true);

        let enable_button = Button::new();
        enable_button.set_label("Enable 2FA");
        enable_button.add_css_class("suggested-action");
        enable_button.set_valign(gtk4::Align::Center);
        setup_row.add_suffix(&enable_button);
        setup_row.set_activatable_widget(Some(&enable_button));

        let vault_name_clone = vault_name.to_string();
        enable_button.connect_clicked(move |_| {
            show_totp_setup_dialog(&vault_name_clone);
        });

        group.add(&setup_row);
    }

    group
}

/// Creates the vault management section with backup and vault operations
fn create_vault_management_section(content_area: &Box, vault_name: &str) -> PreferencesGroup {
    let group = PreferencesGroup::new();
    group.set_title("Vault Management");
    group.set_description(Some("Backup, restore, rename, and delete vault operations"));

    let backup_row = ActionRow::new();
    backup_row.set_title("Create Backup");
    backup_row.set_subtitle("Create a backup of this vault");
    backup_row.set_activatable(true);

    let backup_button = Button::new();
    backup_button.set_label("Backup");
    backup_button.add_css_class("flat");
    backup_button.set_valign(gtk4::Align::Center);
    backup_row.add_suffix(&backup_button);
    backup_row.set_activatable_widget(Some(&backup_button));

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    backup_button.connect_clicked(move |_| {
        show_backup_vault_dialog(&vault_name_clone, &content_area_clone);
    });

    group.add(&backup_row);

    let restore_row = ActionRow::new();
    restore_row.set_title("Restore Backup");
    restore_row.set_subtitle("Restore vault from backup");
    restore_row.set_activatable(true);

    let restore_button = Button::new();
    restore_button.set_label("Restore");
    restore_button.add_css_class("flat");
    restore_button.set_valign(gtk4::Align::Center);

    // Check if backup exists to enable/disable button
    let has_backup = backup_exists(vault_name);
    restore_button.set_sensitive(has_backup);
    restore_row.set_sensitive(has_backup);

    restore_row.add_suffix(&restore_button);
    restore_row.set_activatable_widget(Some(&restore_button));

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    restore_button.connect_clicked(move |_| {
        show_restore_vault_dialog(&vault_name_clone, &content_area_clone);
    });

    group.add(&restore_row);

    let delete_backup_row = ActionRow::new();
    delete_backup_row.set_title("Delete Backup");
    delete_backup_row.set_subtitle("Delete vault backup");
    delete_backup_row.set_activatable(true);

    let delete_backup_button = Button::new();
    delete_backup_button.set_label("Delete");
    delete_backup_button.add_css_class("destructive-action");
    delete_backup_button.set_valign(gtk4::Align::Center);
    delete_backup_button.set_sensitive(has_backup);
    delete_backup_row.set_sensitive(has_backup);

    delete_backup_row.add_suffix(&delete_backup_button);
    delete_backup_row.set_activatable_widget(Some(&delete_backup_button));

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    delete_backup_button.connect_clicked(move |_| {
        show_delete_backup_dialog(&vault_name_clone, &content_area_clone);
    });

    group.add(&delete_backup_row);

    let rename_row = ActionRow::new();
    rename_row.set_title("Rename Vault");
    rename_row.set_subtitle("Change the name of this vault");
    rename_row.set_activatable(true);

    let rename_button = Button::new();
    rename_button.set_label("Rename");
    rename_button.add_css_class("flat");
    rename_button.set_valign(gtk4::Align::Center);
    rename_row.add_suffix(&rename_button);
    rename_row.set_activatable_widget(Some(&rename_button));

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    rename_button.connect_clicked(move |_| {
        show_rename_vault_dialog(&vault_name_clone, &content_area_clone);
    });

    group.add(&rename_row);

    let delete_row = ActionRow::new();
    delete_row.set_title("Delete Vault");
    delete_row.set_subtitle("Permanently delete this vault and all its data");
    delete_row.set_activatable(true);

    let delete_button = Button::new();
    delete_button.set_label("Delete");
    delete_button.add_css_class("destructive-action");
    delete_button.set_valign(gtk4::Align::Center);
    delete_row.add_suffix(&delete_button);
    delete_row.set_activatable_widget(Some(&delete_button));

    let content_area_clone = content_area.clone();
    let vault_name_clone = vault_name.to_string();
    delete_button.connect_clicked(move |_| {
        show_delete_vault_dialog(&vault_name_clone, &content_area_clone);
    });

    group.add(&delete_row);

    group
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

    let info_box = Box::new(Orientation::Vertical, 4);
    info_box.set_hexpand(true);

    let account = account_rc.borrow();
    let title = Label::new(Some(&account.name));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);
    title.set_ellipsize(EllipsizeMode::End);
    title.set_max_width_chars(50); // Increased from 30
    title.set_tooltip_text(Some(&account.name));
    title.set_wrap(true);
    title.set_lines(2);

    let subtitle = Label::new(Some(&account.account_type));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Start);
    subtitle.set_ellipsize(EllipsizeMode::End);
    subtitle.set_max_width_chars(55); // Increased from 35
    subtitle.set_tooltip_text(Some(&account.account_type));

    info_box.append(&title);
    info_box.append(&subtitle);

    let actions_box = Box::new(Orientation::Horizontal, 8);

    if !edit_mode {
        let edit_button = Button::new();
        edit_button.set_label("Edit");
        edit_button.set_tooltip_text(Some("Edit account"));
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

        let rename_button = Button::new();
        rename_button.set_label("Rename");
        rename_button.set_tooltip_text(Some("Rename account"));
        rename_button.add_css_class("suggested-action");

        let delete_button = Button::new();
        delete_button.set_label("Delete");
        delete_button.set_tooltip_text(Some("Delete account"));
        delete_button.add_css_class("destructive-action");

        // Connect delete functionality with confirmation dialog
        let content_area_delete = content_area.clone();
        let vault_name_delete = vault_name.to_string();
        let account_name_delete = account_name.to_string();
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
                            show_home_view(&content_area_confirm);
                        }
                        Err(e) => {
                            eprintln!("Failed to delete account '{account_name_confirm}': {e}");
                        }
                    }
                },
            );
        });

        let content_area_rename = content_area.clone();
        let vault_name_rename = vault_name.to_string();
        let account_name_rename = account_name.to_string();
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

/// Creates the account details section
fn create_account_details_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 20);
    section.add_css_class("account-section");
    section.set_margin_top(20);

    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(16);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

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

    let details_box = Box::new(Orientation::Vertical, 18);
    details_box.set_margin_bottom(20);
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
    let section = Box::new(Orientation::Vertical, 20);
    section.add_css_class("account-section");
    section.set_margin_top(20);

    let header_box = Box::new(Orientation::Horizontal, 12);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

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
    if edit_mode {
        header_box.append(&generate_button);
    }

    section.append(&header_box);

    // Password field with reveal/copy buttons - bigger spacing
    let password_box = Box::new(Orientation::Horizontal, 12);
    password_box.set_margin_bottom(24);
    password_box.set_margin_start(24);
    password_box.set_margin_end(24);
    password_box.set_halign(gtk4::Align::Center);

    let password_entry = Entry::new();
    let account = account_rc.borrow();

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
    password_entry.set_size_request(250, -1); // Reduced width for better responsiveness
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
    if !edit_mode {
        password_box.append(&reveal_button);
    }

    password_box.append(&copy_button);
    section.append(&password_box);

    section
}

#[allow(clippy::too_many_lines)]
/// Creates the additional fields section
fn create_additional_fields_section(
    account_rc: &Rc<RefCell<Account>>,
    edit_mode: bool,
    content_area: &Box,
    vault_name: &str,
) -> Box {
    let section = Box::new(Orientation::Vertical, 16);
    section.add_css_class("account-section");
    section.set_margin_top(16);

    let header_box = Box::new(Orientation::Horizontal, 12);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

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

    if edit_mode {
        let account_rc_clone = account_rc.clone();
        let content_area_clone = content_area.clone();
        let vault_name_clone = vault_name.to_string();
        add_button.connect_clicked(move |_| {
            show_add_field_dialog(&account_rc_clone, &content_area_clone, &vault_name_clone);
        });
    }

    header_box.append(&title_box);
    if edit_mode {
        header_box.append(&add_button);
    }

    section.append(&header_box);

    let fields_box = Box::new(Orientation::Vertical, 8);
    fields_box.set_margin_bottom(20);
    fields_box.set_margin_start(24);
    fields_box.set_margin_end(24);
    fields_box.set_halign(gtk4::Align::Center);

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
        field_entry.set_editable(edit_mode);
        field_entry.set_hexpand(true);
        field_entry.set_size_request(350, -1); // Make textbox longer
        field_entry.add_css_class("password-field");
        field_entry.set_placeholder_text(Some(&format!("Enter {}", field_name.to_lowercase())));

        // Connect field changes in edit mode
        if edit_mode {
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
        if edit_mode {
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
            let content_area_edit = content_area.clone();
            let vault_name_edit = vault_name.to_string();
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
            let content_area_delete = content_area.clone();
            let vault_name_delete = vault_name.to_string();
            let field_name_delete = field_name.clone();
            delete_button.connect_clicked(move |_| {
                show_delete_field_dialog(
                    &account_rc_delete,
                    &content_area_delete,
                    &field_name_delete,
                    &vault_name_delete,
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

/// Creates the notes section
fn create_notes_section(account_rc: &Rc<RefCell<Account>>, edit_mode: bool) -> Box {
    let section = Box::new(Orientation::Vertical, 20);
    section.add_css_class("account-section");
    section.set_margin_top(20);

    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(20);
    header_box.set_margin_bottom(0);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);

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

    let notes_box = Box::new(Orientation::Vertical, 16);
    notes_box.set_margin_bottom(24);
    notes_box.set_margin_start(24);
    notes_box.set_margin_end(24);
    notes_box.set_halign(gtk4::Align::Center);

    // Notes text area (using Entry for now, could be TextView for multiline)
    let notes_entry = Entry::new();
    let account = account_rc.borrow();
    notes_entry.set_text(&account.notes);
    notes_entry.set_hexpand(true);
    notes_entry.set_size_request(250, -1); // Reduced width for better responsiveness
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
    let vault_name_clone = vault_name.to_string();
    let content_area_clone3 = content_area.clone();
    save_button.connect_clicked(move |_| {
        let mut account = account_rc_clone.borrow_mut();
        account.update_modified_time();
        let account_name = account.name.clone();

        match update_account(&vault_name_clone, &account) {
            Ok(()) => {
                drop(account); // Release the borrow
                // Exit edit mode and show the updated account
                show_account_view_with_mode(&content_area_clone3, &vault_name_clone, &account_name, false);
            }
            Err(e) => {
                eprintln!("Failed to save account: {e}");
            }
        }
    });

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    let vault_name_clone2 = vault_name.to_string();
    let content_area_clone2 = content_area.clone();
    let account_rc_clone3 = account_rc.clone();
    cancel_button.connect_clicked(move |_| {
        let account = account_rc_clone3.borrow();
        show_account_view_with_mode(&content_area_clone2, &vault_name_clone2, &account.name, false);
    });

    actions_box.append(&cancel_button);
    actions_box.append(&save_button);

    section.append(&actions_box);
    section
}

/// Creates a field row with label and value
fn create_field_row(label_text: &str, value_text: &str, copyable: bool) -> Box {
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
fn create_editable_field_row(
    label_text: &str,
    initial_value: &str,
    account_rc: &Rc<RefCell<Account>>,
    field_name: &str,
) -> Box {
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

    let entry = Entry::new();
    entry.set_text(initial_value);
    entry.set_hexpand(true);
    entry.set_size_request(250, -1);
    entry.add_css_class("password-field");

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
fn clear_content(content_area: &Box) {
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
            show_vault_view(content_area, vault_name);
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
    error_message.set_max_width_chars(80);
    error_message.set_halign(gtk4::Align::Center);
    error_message.add_css_class("dim-label");
    error_message.set_justify(gtk4::Justification::Center);

    main_box.append(&error_title);
    main_box.append(&error_message);

    content_area.append(&main_box);
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

/// Increments the usage count for a vault
pub fn increment_vault_usage(vault_name: &str) {
    let stats_file = get_vault_stats_file();
    let mut usage_counts = HashMap::new();

    if let Ok(content) = fs::read_to_string(&stats_file) {
        for line in content.lines() {
            if let Some((name, count_str)) = line.split_once(':') {
                if let Ok(count) = count_str.parse::<u32>() {
                    usage_counts.insert(name.to_string(), count);
                }
            }
        }
    }

    let current_count = usage_counts.get(vault_name).unwrap_or(&0);
    usage_counts.insert(vault_name.to_string(), current_count + 1);

    let mut content = String::new();
    for (name, count) in usage_counts {
        use std::fmt::Write;
        writeln!(content, "{name}:{count}").unwrap();
    }

    if let Err(e) = fs::write(&stats_file, content) {
        eprintln!("Failed to write vault stats: {e}");
    }
}

fn get_vault_stats_file() -> PathBuf {
    let locations = crate::vault::Locations::new("", "");
    locations.fmp.join("vault_stats.txt")
}

/// Append the given vault name to the recent list (most recent first, unique)
pub fn record_recent_vault(vault_name: &str) {
    let file = get_recent_vaults_file();
    let mut items: Vec<String> = Vec::new();

    if let Ok(content) = fs::read_to_string(&file) {
        for line in content.lines() {
            let name = line.trim();
            if !name.is_empty() && name != vault_name {
                items.push(name.to_string());
            }
        }
    }

    // Prepend current vault
    items.insert(0, vault_name.to_string());

    // Cap to 10 items
    if items.len() > 10 {
        items.truncate(10);
    }

    let mut out = String::new();
    for name in items {
        use std::fmt::Write;
        writeln!(out, "{name}").ok();
    }

    if let Err(e) = fs::write(&file, out) {
        eprintln!("Failed to write recent vaults: {e}");
    }
}

/// Read recent vaults (most recent first), limited to `limit`
pub fn get_recent_vaults(limit: usize) -> Vec<String> {
    let file = get_recent_vaults_file();
    if let Ok(content) = fs::read_to_string(&file) {
        let mut lines: Vec<String> = content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if lines.len() > limit {
            lines.truncate(limit);
        }
        lines
    } else {
        Vec::new()
    }
}

fn get_recent_vaults_file() -> PathBuf {
    let locations = crate::vault::Locations::new("", "");
    locations.fmp.join("recent_vaults.txt")
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


