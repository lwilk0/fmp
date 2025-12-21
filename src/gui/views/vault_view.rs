use crate::{
    gui::{
        content::{
            CreateBox, CreateScrollableView, VAULT_LOADING_COUNTER, clear_content,
            create_accounts_grid, create_totp_management_section, create_vault_management_section,
            get_available_accounts,
        },
        widgets::loading_spinner::LoadingOverlay,
    },
    storage::filesystem::{increment_vault_usage, record_recent_vault},
};
use adw::prelude::*;
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
        main_box.append(&create_totp_management_section(
            self.content_area,
            &self.vault_name,
        ));
        main_box.append(&create_vault_management_section(
            self.content_area,
            &self.vault_name,
        ));

        let content_area_clone = self.content_area.clone();
        let vault_name_clone = self.vault_name.clone();
        let main_box_clone = main_box.clone();
        let scrollable_clone = scrollable.clone();
        let loading_overlay_clone = loading_overlay.clone();

        let current_id = VAULT_LOADING_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

        glib::idle_add_local(move || {
            let counter = VAULT_LOADING_COUNTER.load(Ordering::SeqCst);
            if counter != current_id {
                loading_overlay_clone.hide();
                return glib::ControlFlow::Break;
            }

            let accounts_section = create_accounts_grid(&content_area_clone, &vault_name_clone);

            main_box_clone.append(&accounts_section);
            scrollable_clone.set_child(Some(&main_box_clone));
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
