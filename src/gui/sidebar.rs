use crate::gui::content::{
    show_home_view, show_new_vault_view, show_settings_view, show_vault_view,
};
use crate::gui::widgets::filtering::create_filter_bar;
use crate::vault::{Locations, read_directory};
use adw::prelude::*;
use adw::{ButtonContent, HeaderBar};
use gtk4::{
    Box, Button, Entry, Label, ListBoxRow, Orientation, Paned, PolicyType, ScrolledWindow,
    Separator,
};
use std::path::PathBuf;

// Constants for better maintainability
const SIDEBAR_WIDTH: i32 = 250;

/// Gets the vaults directory path
fn get_vaults_directory() -> PathBuf {
    let locations = Locations::new("", "");
    locations.fmp.join("vaults")
}

/// Reads available vaults from the vaults directory
pub fn get_available_vaults() -> Vec<String> {
    let vaults_dir = get_vaults_directory();
    read_directory(&vaults_dir).unwrap_or_else(|_| {
        eprintln!("Failed to read vaults directory: {}", vaults_dir.display());
        Vec::new()
    })
}

/// Creates a paned layout with sidebar that can update the main content
pub fn create_paned_layout_with_callbacks(main_content: &Box, content_area: &Box) -> Paned {
    let sidebar = create_sidebar_with_callbacks(content_area);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(main_content));
    paned.set_position(SIDEBAR_WIDTH);
    paned.set_shrink_start_child(false);
    paned.set_resize_start_child(false);

    paned
}

/// Creates a sidebar with navigation items that can update content
pub fn create_sidebar_with_callbacks(content_area: &Box) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 0);

    // Add dark background styling
    sidebar.add_css_class("sidebar");
    sidebar.set_css_classes(&["sidebar", "background"]);

    // Create vault buttons section (initially unfiltered)
    let vaults_section = create_vaults_section(content_area, "");

    // Create filter bar (visible by default)
    let filter_bar = create_filter_bar("", "Search vaults...", true);
    filter_bar.set_visible(true);
    filter_bar.set_margin_start(16);
    filter_bar.set_margin_end(16);
    filter_bar.set_margin_top(8);
    filter_bar.set_margin_bottom(8);
    filter_bar.set_halign(gtk4::Align::Fill);

    // Connect filter bar to update vaults section
    connect_filter_to_vaults(&filter_bar, &vaults_section, content_area);

    // Create header bar with navigation buttons
    let header_bar = create_header_bar(content_area, &filter_bar);
    sidebar.append(&header_bar);

    // Create a scrollable container for the content below the header
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    // Create a box to hold the scrollable content
    let scrollable_content = Box::new(Orientation::Vertical, 0);

    // Add sidebar title and divider to scrollable content
    let title_section = create_title_section(content_area);
    scrollable_content.append(&title_section);

    // Add filter bar to scrollable content
    scrollable_content.append(&filter_bar);

    // Add vaults section to scrollable content
    scrollable_content.append(&vaults_section);

    // Set the scrollable content as the child of the scrolled window
    scrolled_window.set_child(Some(&scrollable_content));

    // Add the scrolled window to the sidebar
    sidebar.append(&scrolled_window);

    sidebar
}

/// Creates the header bar with navigation buttons
fn create_header_bar(content_area: &Box, filter_bar: &Box) -> HeaderBar {
    let header_bar = HeaderBar::builder()
        .title_widget(&Label::new(Some("")))
        .show_end_title_buttons(false)
        .build();

    // Create and pack navigation buttons
    let home_button = create_icon_button("go-home");
    let settings_button = create_icon_button("settings");
    let search_button = create_icon_button("system-search-symbolic");

    header_bar.pack_start(&home_button);
    header_bar.pack_start(&settings_button);
    header_bar.pack_end(&search_button);

    // Connect button callbacks
    let content_area_home = content_area.clone();
    home_button.connect_clicked(move |_| {
        show_home_view(&content_area_home);
    });

    let content_area_settings = content_area.clone();
    settings_button.connect_clicked(move |_| {
        show_settings_view(&content_area_settings);
    });

    // Search button toggles filter bar visibility
    let filter_bar_clone = filter_bar.clone();
    search_button.connect_clicked(move |_| {
        let is_visible = filter_bar_clone.is_visible();
        filter_bar_clone.set_visible(!is_visible);
    });

    header_bar
}

/// Creates a title section with divider and add vault button
fn create_title_section(content_area: &Box) -> Box {
    let title_box = Box::new(Orientation::Vertical, 0);

    // Header with title and add button
    let header_box = Box::new(Orientation::Horizontal, 8);
    header_box.set_margin_start(16);
    header_box.set_margin_end(16);
    header_box.set_margin_top(12);
    header_box.set_margin_bottom(8);

    // Title label
    let title_label = Label::new(Some("Vaults"));
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_hexpand(true);
    title_label.add_css_class("heading");
    title_label.add_css_class("sidebar-title");

    // Add vault button
    let add_vault_button = Button::new();
    add_vault_button.set_label("+");
    add_vault_button.add_css_class("circular");
    add_vault_button.add_css_class("suggested-action");
    add_vault_button.set_tooltip_text(Some("Add New Vault"));
    add_vault_button.set_size_request(32, 32);

    // Connect add vault functionality
    let content_area_clone = content_area.clone();
    add_vault_button.connect_clicked(move |_| {
        show_new_vault_view(&content_area_clone);
    });

    header_box.append(&title_label);
    header_box.append(&add_vault_button);

    // Divider
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    separator.set_margin_start(16);
    separator.set_margin_end(16);
    separator.set_margin_bottom(8);
    separator.add_css_class("sidebar-divider");

    title_box.append(&header_box);
    title_box.append(&separator);

    title_box
}

/// Connects the filter bar to update the vaults section when text changes
fn connect_filter_to_vaults(filter_bar: &Box, vaults_section: &Box, content_area: &Box) {
    // Get the entry widget from the filter bar
    if let Some(entry) = filter_bar
        .first_child()
        .and_then(|child| child.downcast::<Entry>().ok())
    {
        let vaults_section_clone = vaults_section.clone();
        let content_area_clone = content_area.clone();

        entry.connect_changed(move |entry| {
            let filter_text = entry.text();

            // Clear existing vault buttons
            let mut child = vaults_section_clone.first_child();
            while let Some(widget) = child {
                let next_child = widget.next_sibling();
                vaults_section_clone.remove(&widget);
                child = next_child;
            }

            // Create new filtered vault section content
            let new_content = create_vaults_section(&content_area_clone, &filter_text);

            // Move all children from new_content to vaults_section_clone
            let mut new_child = new_content.first_child();
            while let Some(widget) = new_child {
                let next_child = widget.next_sibling();
                new_content.remove(&widget);
                vaults_section_clone.append(&widget);
                new_child = next_child;
            }
        });
    }
}

/// Creates a button with an icon
pub fn create_icon_button(icon_name: &str) -> Button {
    let button = Button::new();
    let button_content = ButtonContent::builder()
        .icon_name(icon_name)
        .use_underline(true)
        .build();
    button.set_child(Some(&button_content));
    button
}

/// Creates a section with buttons for each available vault
fn create_vaults_section(content_area: &Box, filter: &str) -> Box {
    let vaults_box = Box::new(Orientation::Vertical, 4);
    vaults_box.set_margin_top(8);
    vaults_box.set_margin_start(16);
    vaults_box.set_margin_end(16);
    vaults_box.set_margin_bottom(16);

    // Get available vaults and apply filtering
    let all_vaults = get_available_vaults();
    let filtered_vaults = crate::gui::widgets::filtering::sort_vaults(&all_vaults, filter);

    if filtered_vaults.is_empty() {
        let message = if filter.is_empty() {
            "No vaults found"
        } else {
            "No vaults match your search"
        };
        let no_vaults_label = Label::new(Some(message));
        no_vaults_label.add_css_class("dim-label");
        no_vaults_label.add_css_class("sidebar-empty-message");
        no_vaults_label.set_halign(gtk4::Align::Start);
        no_vaults_label.set_margin_top(16);
        vaults_box.append(&no_vaults_label);
    } else {
        for vault_name in filtered_vaults {
            let vault_button = create_vault_button(vault_name, content_area);
            vaults_box.append(&vault_button);
        }
    }

    vaults_box
}

/// Creates a button for a specific vault
fn create_vault_button(vault_name: &str, content_area: &Box) -> Button {
    let button = Button::new();
    button.add_css_class("flat");
    button.add_css_class("sidebar-vault-button");
    button.set_halign(gtk4::Align::Fill);
    button.set_hexpand(true);

    // Create a box to hold the label with proper alignment
    let button_box = Box::new(Orientation::Horizontal, 0);
    let label = Label::new(Some(vault_name));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    label.add_css_class("sidebar-vault-label");

    button_box.append(&label);
    button.set_child(Some(&button_box));

    // Connect click handler
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    button.connect_clicked(move |_| {
        show_vault_view(&content_area_clone, &vault_name_clone);
    });

    button
}

/// Refreshes the vaults section in the sidebar
pub fn refresh_vaults_section(sidebar: &Box, content_area: &Box) {
    // Find the scrolled window in the sidebar
    if let Some(scrolled_window) = sidebar
        .last_child()
        .and_then(|child| child.downcast::<ScrolledWindow>().ok())
    {
        if let Some(scrollable_content) = scrolled_window
            .child()
            .and_then(|child| child.downcast::<Box>().ok())
        {
            // Find the vaults section (should be the last child)
            if let Some(vaults_section) = scrollable_content
                .last_child()
                .and_then(|child| child.downcast::<Box>().ok())
            {
                // Clear existing vault buttons
                let mut child = vaults_section.first_child();
                while let Some(widget) = child {
                    let next_child = widget.next_sibling();
                    vaults_section.remove(&widget);
                    child = next_child;
                }

                // Recreate the vault buttons with updated vault list
                let new_vaults_section = create_vaults_section(content_area, "");

                // Move all children from new_vaults_section to vaults_section
                let mut new_child = new_vaults_section.first_child();
                while let Some(widget) = new_child {
                    let next_child = widget.next_sibling();
                    new_vaults_section.remove(&widget);
                    vaults_section.append(&widget);
                    new_child = next_child;
                }
            }
        }
    }
}

/// Creates a clickable navigation row
fn create_clickable_row(label: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_activatable(true);
    row.set_selectable(true);

    let box_container = Box::new(Orientation::Horizontal, 8);
    box_container.set_margin_top(8);
    box_container.set_margin_bottom(8);
    box_container.set_margin_start(12);
    box_container.set_margin_end(12);

    let text_label = Label::new(Some(label));
    text_label.set_halign(gtk4::Align::Start);
    box_container.append(&text_label);

    row.set_child(Some(&box_container));
    row
}
