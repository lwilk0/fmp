use crate::{
    gui::{
        content::{
            CreateActionRow, CreateBox, CreateScrollableView, VAULT_LOADING_COUNTER, clear_content,
            get_available_accounts, proceed_with_gate_warmup,
        },
        dialogs::show_standalone_password_generator_dialog,
        widgets::loading_spinner::{create_loading_button, set_button_loading_state},
    },
    storage::filesystem::{get_most_used_vault, get_recent_vaults},
    vault::create_vault,
};
use adw::{PreferencesGroup, prelude::*};
use gtk4::{Align, Box, Button, Entry, Label, Orientation};
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
        group.set_description(Some("Your vault statistics"));

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
                .subtitle("Your most frequently used vault")
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
                        move || proceed_with_gate_warmup(&content_area, &name)
                    })
                    .build(),
            );
        }

        group
    }
}

/// Shows the create vault view
pub fn show_create_vault_view(content_area: &Box) {
    clear_content(content_area);

    let main_box = CreateBox::new()
        .new_box(Box::new(Orientation::Vertical, 24))
        .margins(48, 48, 48, 48)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

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

    let actions_box = CreateBox::new()
        .new_box(Box::new(Orientation::Horizontal, 12))
        .halign(Align::Center)
        .build();

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.set_size_request(120, -1);

    let content_area_clone2 = content_area.clone();
    cancel_button.connect_clicked(move |_| HomeView::new(&content_area_clone2).create());

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
