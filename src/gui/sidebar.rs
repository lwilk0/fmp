use crate::{
    gui::{content::proceed_with_gate_warmup, views::home_view::HomeView},
    storage::filesystem::read_directory,
    vault::Locations,
};
use adw::{ButtonContent, Clamp, HeaderBar, PreferencesGroup, prelude::*};
use gpgme::Context;
use gtk4::{Box, Button, Label, Orientation, Paned, PolicyType, ScrolledWindow, SearchEntry};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

const SIDEBAR_WIDTH: i32 = 250;

/// Gets the vaults directory path
fn get_vaults_directory() -> PathBuf {
    let locations = Locations::new("", "");
    locations.fmp.join("vaults")
}

/// Reads available vaults from the vaults directory
pub fn get_available_vaults() -> Vec<String> {
    let vaults_dir = get_vaults_directory();

    read_directory(&vaults_dir).unwrap_or_else(|e| {
        log::error!(
            "Failed to read vaults directory: {} - Error: {}",
            vaults_dir.display(),
            e
        );
        Vec::new()
    })
}

/// Creates a responsive paned layout with sidebar that can update the main content
pub fn create_paned_layout_with_callbacks(
    main_content: &Box,
    content_area: &Box,
    ctx: Rc<RefCell<Context>>,
) -> Paned {
    let sidebar = create_sidebar_with_callbacks(content_area, ctx);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(main_content));
    paned.set_position(SIDEBAR_WIDTH);
    paned.set_shrink_start_child(false);
    paned.set_resize_start_child(true);
    paned.set_wide_handle(false);

    paned
}

/// Creates a sidebar with navigation items that can update content
pub fn create_sidebar_with_callbacks(content_area: &Box, ctx: Rc<RefCell<Context>>) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 0);
    sidebar.add_css_class("sidebar");

    let header_bar = create_header_bar(content_area, ctx.clone());
    sidebar.append(&header_bar);

    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search vaults..."));
    search_entry.set_margin_start(8);
    search_entry.set_margin_end(8);
    search_entry.set_margin_top(8);
    search_entry.set_margin_bottom(8);
    sidebar.append(&search_entry);

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let clamp = Clamp::new();
    clamp.set_tightening_threshold(240);
    clamp.set_margin_start(8);
    clamp.set_margin_end(8);

    // --- Read filesystem ONCE, build all rows ONCE ---
    let all_vaults = get_available_vaults();
    let group = PreferencesGroup::new();

    // Store (vault_name, row) pairs so we can show/hide later
    let rows: Rc<Vec<(String, adw::ActionRow)>> = Rc::new(
        all_vaults
            .iter()
            .map(|name| {
                let row = create_vault_action_row(name, content_area, ctx.clone());
                group.add(&row);
                (name.clone(), row)
            })
            .collect(),
    );

    // Also keep an empty-state row, hidden by default
    let empty_row = create_empty_state_action_row("");
    empty_row.set_visible(false);
    group.add(&empty_row);

    clamp.set_child(Some(&group));
    scrolled_window.set_child(Some(&clamp));

    // --- Connect search: just toggle visibility ---
    let rows_clone = rows.clone();
    search_entry.connect_search_changed(move |entry| {
        let filter = entry.text().to_string();
        let filter_lc = filter.to_ascii_lowercase();

        let mut visible_count = 0;
        for (name, row) in rows_clone.iter() {
            let matches = filter_lc.is_empty() || name.to_ascii_lowercase().contains(&filter_lc);
            row.set_visible(matches);
            if matches {
                visible_count += 1;
            }
        }

        // Show/hide empty state
        if visible_count == 0 {
            if filter.is_empty() {
                empty_row.set_title("No vaults found");
                empty_row.set_subtitle("Create your first vault to get started");
            } else {
                empty_row.set_title("No vaults match your search");
                empty_row.set_subtitle("Try a different search term");
            }
            empty_row.set_visible(true);
        } else {
            empty_row.set_visible(false);
        }
    });

    sidebar.append(&scrolled_window);
    sidebar
}

/// Creates the header bar with navigation buttons
fn create_header_bar(content_area: &Box, ctx: Rc<RefCell<Context>>) -> HeaderBar {
    let header_bar = HeaderBar::new();
    let title_label = Label::new(None);
    title_label.add_css_class("heading");
    header_bar.set_title_widget(Some(&title_label));
    header_bar.add_css_class("flat");
    header_bar.set_show_end_title_buttons(false);

    let home_button = create_icon_button("go-home-symbolic");
    home_button.set_tooltip_text(Some("Home"));
    header_bar.pack_start(&home_button);

    let content_area_home = content_area.clone();
    home_button.connect_clicked(move |_| HomeView::new(&content_area_home).create(ctx.clone()));

    header_bar
}

/// Creates an empty state action row for when no vaults are found
fn create_empty_state_action_row(filter: &str) -> adw::ActionRow {
    use adw::ActionRow;

    let row = ActionRow::new();
    row.set_activatable(false);
    row.set_margin_start(8);
    row.set_margin_end(8);

    if filter.is_empty() {
        row.set_title("No vaults found");
        row.set_subtitle("Create your first vault to get started");
    } else {
        row.set_title("No vaults match your search");
        row.set_subtitle("Try a different search term");
    }

    row
}

/// Creates an action row for a specific vault
fn create_vault_action_row(
    vault_name: &str,
    content_area: &Box,
    ctx: Rc<RefCell<Context>>,
) -> adw::ActionRow {
    use adw::ActionRow;

    let row = ActionRow::new();
    row.set_title(vault_name);
    row.set_subtitle("Password vault");
    row.set_activatable(true);
    row.set_margin_start(8);
    row.set_margin_end(8);

    let open_button = Button::new();
    open_button.set_label("Open");
    open_button.add_css_class("flat");
    open_button.set_valign(gtk4::Align::Center);
    row.add_suffix(&open_button);
    row.set_activatable_widget(Some(&open_button));

    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    open_button.connect_clicked(move |_| {
        proceed_with_gate_warmup(&content_area_clone, &vault_name_clone, ctx.clone());
    });

    row
}

/// Creates a button with an icon
pub fn create_icon_button(icon_name: &str) -> Button {
    let button = Button::new();
    let button_content = ButtonContent::builder()
        .icon_name(icon_name)
        .use_underline(true)
        .build();
    button.set_child(Some(&button_content));
    button.add_css_class("flat");
    button.set_margin_start(2);
    button.set_margin_end(2);
    button
}

/// Refreshes the sidebar from the content area (convenience function)
pub fn refresh_sidebar_from_content_area(content_area: &Box, ctx: Rc<RefCell<Context>>) {
    if let Some(main_content) = content_area.parent().and_then(|p| p.downcast::<Box>().ok())
        && let Some(paned) = main_content
            .parent()
            .and_then(|p| p.downcast::<Paned>().ok())
    {
        let new_sidebar = create_sidebar_with_callbacks(content_area, ctx);
        paned.set_start_child(Some(&new_sidebar));
    }
}
