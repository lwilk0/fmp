use crate::{
    gui::{dialogs::totp::show_totp_authentication_dialog, views::vault_view::VaultView},
    storage::filesystem::read_directory,
    totp::is_totp_required,
    vault::{Account, Locations, warm_up_gpg},
};
use adw::{ActionRow, ButtonContent, Clamp, prelude::*};
use gtk4::{
    Align, Box, Button, Entry, Label, Orientation, PolicyType, ScrolledWindow, pango::EllipsizeMode,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::atomic::AtomicU64};

// Global counter for tracking vault loading operations
pub static VAULT_LOADING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Creates a field row with label and value
pub fn create_field_row(label_text: &str, value_text: &str, copyable: bool) -> Box {
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

/// Creates an editable field row for account creation/editing
pub fn create_editable_field_row(
    label_text: &str,
    initial_value: &str,
    account_rc: &Rc<RefCell<Account>>,
    field_name: &str,
) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 16);
    row_box.set_halign(gtk4::Align::Fill);
    row_box.set_margin_top(2);
    row_box.set_margin_bottom(2);

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
    //entry.add_css_class("password-field");

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
pub fn clear_content(content_area: &Box) {
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
            VaultView::new(content_area, vault_name).create();
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

    let main_box = CreateBox::new()
        .new_box(Box::new(Orientation::Vertical, 24))
        .margins(48, 48, 48, 48)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

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

// TODO move to filesystem
/// Reads available vaults from the vaults directory
pub fn get_available_accounts(vault_name: &str) -> Vec<String> {
    let account_dir = get_account_directory(vault_name);
    read_directory(&account_dir).unwrap_or_else(|_| {
        log::error!(
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

pub struct CreateActionRow<F: Fn() + 'static> {
    title: Option<String>,
    subtitle: Option<String>,
    button_label: Option<String>,
    css_class: Option<String>,
    callback: Option<F>,
    activatable: bool,
    margin_start: i32,
    margin_end: i32,
    add_button: bool,
    suffix_widget: Option<gtk4::Widget>,
}

impl<F: Fn() + 'static> Default for CreateActionRow<F> {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            button_label: None,
            css_class: None,
            activatable: false,
            margin_start: 8,
            margin_end: 8,
            callback: None,
            add_button: true,
            suffix_widget: None,
        }
    }
}

impl<F: Fn() + 'static> CreateActionRow<F> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    pub fn button_label(mut self, bl: impl Into<String>) -> Self {
        self.button_label = Some(bl.into());
        self
    }
    pub fn css_class(mut self, cc: impl Into<String>) -> Self {
        self.css_class = Some(cc.into());
        self
    }
    pub fn activatable(mut self, a: bool) -> Self {
        self.activatable = a;
        self
    }
    pub fn callback(mut self, c: F) -> Self {
        self.callback = Some(c);
        self
    }
    pub fn add_button(mut self, ab: bool) -> Self {
        self.add_button = ab;
        self
    }
    pub fn suffix(mut self, widget: &impl gtk4::prelude::IsA<gtk4::Widget>) -> Self {
        self.suffix_widget = Some(widget.clone().upcast());
        self
    }

    pub fn build(self) -> ActionRow {
        let row = ActionRow::new();
        let button = Button::new();

        if let Some(t) = self.title {
            row.set_title(&t);
        }
        if let Some(s) = self.subtitle {
            row.set_subtitle(&s);
        }

        row.set_margin_start(self.margin_start);
        row.set_margin_end(self.margin_end);

        if let Some(custom_widget) = self.suffix_widget {
            row.add_suffix(&custom_widget);
        }

        if self.add_button {
            if let Some(bl) = self.button_label {
                button.set_label(&bl)
            }
            if let Some(cc) = self.css_class {
                button.add_css_class(&cc)
            }

            button.set_valign(gtk4::Align::Center);
            row.add_suffix(&button);
            row.set_activatable_widget(Some(&button));

            if let Some(c) = self.callback {
                button.connect_clicked(move |_| c());
            }
        }

        row.set_activatable(self.activatable);

        row
    }
}

pub struct CreateBox {
    new_box: Box,
    top: i32,
    bottom: i32,
    start: i32,
    end: i32,
    halign: Option<Align>,
    valign: Option<Align>,
}

impl Default for CreateBox {
    fn default() -> Self {
        Self {
            new_box: Box::new(Orientation::Vertical, 0),
            top: 24,
            bottom: 24,
            start: 24,
            end: 24,
            halign: None,
            valign: None,
        }
    }
}

impl CreateBox {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_box(mut self, nb: Box) -> Self {
        self.new_box = nb;
        self
    }
    pub fn margins(mut self, top: i32, bottom: i32, start: i32, end: i32) -> Self {
        self.top = top;
        self.bottom = bottom;
        self.start = start;
        self.end = end;
        self
    }
    pub fn halign(mut self, h: impl Into<Align>) -> Self {
        self.halign = Some(h.into());
        self
    }
    pub fn valign(mut self, v: impl Into<Align>) -> Self {
        self.valign = Some(v.into());
        self
    }

    pub fn build(self) -> Box {
        let content_box = self.new_box;

        content_box.set_margin_top(self.top);
        content_box.set_margin_bottom(self.bottom);
        content_box.set_margin_start(self.start);
        content_box.set_margin_end(self.end);

        if let Some(h) = self.halign {
            content_box.set_halign(h)
        }
        if let Some(v) = self.valign {
            content_box.set_valign(v)
        }

        content_box
    }
}

pub struct CreateScrollableView {
    max_width: i32,
    tighten_threshold: i32,
}

impl Default for CreateScrollableView {
    fn default() -> Self {
        Self {
            max_width: 800,
            tighten_threshold: 600,
        }
    }
}

impl CreateScrollableView {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn max_width(mut self, width: i32) -> Self {
        self.max_width = width;
        self
    }
    pub fn tighten_threshold(mut self, threshold: i32) -> Self {
        self.tighten_threshold = threshold;
        self
    }

    pub fn build(self) -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let clamp = Clamp::new();
        clamp.set_maximum_size(self.max_width);
        clamp.set_tightening_threshold(self.tighten_threshold);

        scrolled.set_child(Some(&clamp));
        scrolled
    }
}
