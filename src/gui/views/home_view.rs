use crate::{
    gui::{
        content::{
            CreateActionRow, CreateBox, CreateScrollableView, VAULT_LOADING_COUNTER, clear_content,
            get_available_accounts, show_create_vault_view,
        },
        dialogs::show_standalone_password_generator_dialog,
        views::vault_view::VaultView,
    },
    storage::filesystem::{get_most_used_vault, get_recent_vaults},
};
use adw::{PreferencesGroup, prelude::*};
use gtk4::{Box, Label, Orientation};
use std::sync::atomic::Ordering;

pub struct HomeView<'a> {
    content_area: &'a Box,
}

impl<'a> HomeView<'a> {
    pub fn new(content_area: &'a Box) -> Self {
        Self { content_area }
    }

    /// Creates the home view for the content area
    pub fn create(&self) {
        self.clear();

        let main_box = CreateBox::new()
            .new_box(Box::new(Orientation::Vertical, 16))
            .build();

        let scrollable = CreateScrollableView::new()
            .max_width(800)
            .tighten_threshold(600)
            .build();

        //main_box.append(&self.welcome_section());
        main_box.append(&self.statistics_section());
        main_box.append(&self.quick_actions_section());
        main_box.append(&self.recent_vaults_section());

        scrollable.set_child(Some(&main_box));
        self.content_area.append(&scrollable);
    }

    pub fn clear(&self) {
        clear_content(self.content_area);
        VAULT_LOADING_COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    /// Creates the statistics section showing vault and account counts
    pub fn statistics_section(&self) -> PreferencesGroup {
        let group = PreferencesGroup::new();
        group.set_title("Overview");
        group.set_description(Some("Your vault statistics at a glance"));

        let vaults = crate::gui::sidebar::get_available_vaults();
        let vault_count = vaults.len();
        let most_used = get_most_used_vault();
        let total_accounts: usize = vaults
            .iter()
            .map(|vault_name| get_available_accounts(vault_name).len())
            .sum();

        let vault_label = Label::new(Some(&vault_count.to_string()));
        vault_label.add_css_class("title-3");

        let account_label = Label::new(Some(&total_accounts.to_string()));
        account_label.add_css_class("title-3");

        let most_used_label = Label::new(Some(&most_used));
        most_used_label.add_css_class("title-3");

        group.add(
            &CreateActionRow::<fn()>::new()
                .title("Vaults")
                .subtitle("Total number of vaults")
                .suffix(&vault_label)
                .activatable(false)
                .add_button(false)
                .build(),
        );

        group.add(
            &CreateActionRow::<fn()>::new()
                .title("Accounts")
                .subtitle("Total number of accounts")
                .suffix(&account_label)
                .activatable(false)
                .add_button(false)
                .build(),
        );

        group.add(
            &CreateActionRow::<fn()>::new()
                .title("Most Used Vault")
                .subtitle("Your frequently used vaults")
                .suffix(&most_used_label)
                .activatable(false)
                .add_button(false)
                .build(),
        );

        group
    }

    /// Creates the quick actions section
    fn quick_actions_section(&self) -> PreferencesGroup {
        let group = PreferencesGroup::new();
        group.set_title("Quick Actions");
        group.set_description(Some("Get started with common tasks"));

        let content_area = self.content_area.clone();

        group.add(
            &CreateActionRow::new()
                .title("Create New Vault")
                .subtitle("Set up a vault for your passwords")
                .button_label("Create")
                .css_class("suggested-action")
                .callback({
                    let content_area = content_area.clone();
                    move || show_create_vault_view(&content_area)
                })
                .build(),
        );

        group.add(
            &CreateActionRow::new()
                .title("Generate Password")
                .subtitle("Create a secure password with customisable options")
                .button_label("Generate")
                .css_class("suggested-action")
                .callback(show_standalone_password_generator_dialog)
                .build(),
        );

        group
    }

    /// Creates the recent vaults section for the home view (bottom panel)
    fn recent_vaults_section(&self) -> PreferencesGroup {
        let group = PreferencesGroup::new();

        let content_area = &self.content_area.clone();

        group.set_title("Recently Accessed Vaults");
        group.set_description(Some("Open a vault you used recently"));

        let recent = get_recent_vaults(5);
        if recent.is_empty() {
            group.add(
                &CreateActionRow::<fn()>::new()
                    .title("No Recent Vaults Yet")
                    .subtitle("Open a vault to see it here")
                    .activatable(false)
                    .add_button(false)
                    .build(),
            );

            return group;
        }

        for name in recent {
            let name_clone = name.clone();

            group.add(
                &CreateActionRow::new()
                    .title(&name)
                    .subtitle("Click to open")
                    .button_label("Open")
                    .css_class("suggested-action")
                    .callback({
                        let content_area = content_area.clone();
                        let name = name_clone.clone();
                        move || VaultView::new(&content_area, &name).create()
                    })
                    .build(),
            );
        }

        group
    }
}
