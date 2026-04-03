use crate::{
    gui::{
        content::{
            CreateActionRow, CreateBox, CreateScrollableView, VAULT_LOADING_COUNTER, clear_content,
        },
        dialogs::{
            totp::{show_totp_management_dialog, show_totp_setup_dialog},
            vault_management::{
                show_backup_vault_dialog, show_delete_backup_dialog, show_delete_vault_dialog,
                show_rename_vault_dialog, show_restore_vault_dialog,
            },
        },
        views::{account_view::AccountView, account_view::show_new_account_view},
        widgets::loading_spinner::LoadingOverlay,
    },
    storage::filesystem::{
        backup_exists, get_available_accounts, increment_vault_usage, record_recent_vault,
    },
    totp::is_totp_enabled,
};
use adw::{ActionRow, PreferencesGroup, prelude::*};
use gtk4::{Box, Label, Orientation};
use std::{rc::Rc, sync::atomic::Ordering};

pub struct VaultView<'a> {
    content_area: &'a Box,
    vault_name: String,
}

impl<'a> VaultView<'a> {
    pub fn new(content_area: &'a Box, vault_name: &str) -> Self {
        Self {
            content_area,
            vault_name: vault_name.to_string(),
        }
    }

    pub fn create(&self) {
        self.clear();

        increment_vault_usage(&self.vault_name);
        record_recent_vault(&self.vault_name);

        let loading_overlay = Rc::new(LoadingOverlay::new());
        self.content_area.append(loading_overlay.widget());
        loading_overlay.show("Loading accounts...");

        let main_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 16))
            .build();

        let scrollable = CreateScrollableView::new()
            .max_width(800)
            .tighten_threshold(600)
            .build();

        main_box.append(&self.header_section());

        let content_area_clone = self.content_area.clone();
        let vault_name_clone = self.vault_name.clone();
        let scrollable_clone = scrollable.clone();
        let loading_overlay_clone = loading_overlay.clone();

        let current_id = VAULT_LOADING_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

        glib::idle_add_local(move || {
            let counter = VAULT_LOADING_COUNTER.load(Ordering::SeqCst);
            if counter != current_id {
                loading_overlay_clone.hide();
                return glib::ControlFlow::Break;
            }

            main_box.append(&create_accounts_grid(
                &content_area_clone,
                &vault_name_clone,
            ));
            main_box.append(&create_totp_management_section(
                &content_area_clone,
                &vault_name_clone,
            ));
            main_box.append(&create_vault_management_section(
                &content_area_clone,
                &vault_name_clone,
            ));

            scrollable_clone.set_child(Some(&main_box));
            content_area_clone.append(&scrollable_clone);

            loading_overlay_clone.hide();
            glib::ControlFlow::Break
        });
    }

    fn clear(&self) {
        clear_content(self.content_area);
        VAULT_LOADING_COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    fn header_section(&self) -> Box {
        let header_box = Box::new(Orientation::Horizontal, 16);
        header_box.set_halign(gtk4::Align::Fill);

        let info_box = Box::new(Orientation::Vertical, 4);
        info_box.set_hexpand(true);

        let title = Label::new(Some(&self.vault_name));
        title.add_css_class("title-1");
        title.set_halign(gtk4::Align::Start);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title.set_max_width_chars(50);
        title.set_wrap(true);
        title.set_lines(2);

        let account_count = get_available_accounts(&self.vault_name).len();
        let subtitle_text = match account_count {
            1 => "Vault • 1 account".to_string(),
            n => format!("Vault • {n} accounts"),
        };

        let subtitle = Label::new(Some(&subtitle_text));
        subtitle.add_css_class("dim-label");
        subtitle.add_css_class("caption");
        subtitle.set_halign(gtk4::Align::Start);
        subtitle.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        subtitle.set_max_width_chars(55);
        subtitle.set_tooltip_text(Some(&subtitle_text));

        info_box.append(&title);
        info_box.append(&subtitle);

        header_box.append(&info_box);
        header_box
    }
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
            .css_class("suggested-action")
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
            .css_class("suggested-action")
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
            .title("Rename Vault")
            .subtitle("Change the name of this vault")
            .button_label("Rename")
            .css_class("suggested-action")
            .callback({
                let vault_name = vault_name_clone.clone();
                let content_area = content_area_clone.clone();
                move || show_rename_vault_dialog(&vault_name, &content_area)
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
        .css_class("suggested-action")
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
                    move || show_totp_setup_dialog(&vault_name, &content_area_clone)
                })
                .build(),
        );
    }

    group
}
