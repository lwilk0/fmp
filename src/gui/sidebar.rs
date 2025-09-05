use crate::gui::content::{open_vault_with_gate, show_home_view};
use crate::gui::widgets::filtering::create_filter_bar;
use crate::vault::{Locations, read_directory};
use adw::prelude::*;
use adw::{ActionRow, ButtonContent, Clamp, HeaderBar, PreferencesGroup};
use gtk4::{
    Box, Button, Entry, Label, ListBox, ListBoxRow, Orientation, Paned, PolicyType, ScrolledWindow,
    SearchEntry, Separator,
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
    println!("Reading vaults from directory: {}", vaults_dir.display());

    let result = read_directory(&vaults_dir).unwrap_or_else(|e| {
        eprintln!(
            "Failed to read vaults directory: {} - Error: {}",
            vaults_dir.display(),
            e
        );
        Vec::new()
    });

    println!("Found {} vaults: {:?}", result.len(), result);
    result
}

/// Creates a responsive paned layout with sidebar that can update the main content
pub fn create_paned_layout_with_callbacks(main_content: &Box, content_area: &Box) -> Paned {
    let sidebar = create_sidebar_with_callbacks(content_area);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(main_content));
    paned.set_position(SIDEBAR_WIDTH);
    paned.set_shrink_start_child(false);
    paned.set_resize_start_child(true); // Allow sidebar to be resized
    paned.set_wide_handle(true); // Make the handle easier to grab

    paned
}

/// Creates a sidebar with navigation items that can update content
pub fn create_sidebar_with_callbacks(content_area: &Box) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 0);
    sidebar.add_css_class("sidebar");

    // Create header bar with navigation buttons
    let header_bar = create_header_bar(content_area);
    sidebar.append(&header_bar);

    // Create search entry
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search vaults..."));
    search_entry.set_margin_start(12);
    search_entry.set_margin_end(12);
    search_entry.set_margin_top(8);
    search_entry.set_margin_bottom(8);
    sidebar.append(&search_entry);

    // Create scrolled window for vault list
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    // Use Clamp for proper content width
    let clamp = Clamp::new();
    clamp.set_maximum_size(400);
    clamp.set_tightening_threshold(300);

    // Create vault list using PreferencesGroup for better libadwaita integration
    let vault_group = create_vault_preferences_group(content_area, "");
    clamp.set_child(Some(&vault_group));
    scrolled_window.set_child(Some(&clamp));

    // Connect search functionality
    connect_search_to_vault_group(&search_entry, &vault_group, content_area);

    sidebar.append(&scrolled_window);
    sidebar
}

/// Creates the header bar with navigation buttons
fn create_header_bar(content_area: &Box) -> HeaderBar {
    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Vaults"))));
    header_bar.add_css_class("flat");

    // Create home button
    let home_button = create_icon_button("go-home-symbolic");
    home_button.set_tooltip_text(Some("Home"));
    header_bar.pack_start(&home_button);

    // Connect home button callback
    let content_area_home = content_area.clone();
    home_button.connect_clicked(move |_| {
        show_home_view(&content_area_home);
    });

    header_bar
}

/// Creates a vault preferences group with filtering capability
fn create_vault_preferences_group(content_area: &Box, filter: &str) -> adw::PreferencesGroup {
    use adw::{ActionRow, PreferencesGroup};

    let group = PreferencesGroup::new();
    group.set_title("Your Vaults");
    group.set_description(Some("Select a vault to open"));

    // Get available vaults and apply filtering
    let all_vaults = get_available_vaults();
    let filtered_vaults = crate::gui::widgets::filtering::sort_vaults(&all_vaults, filter);

    if filtered_vaults.is_empty() {
        let empty_row = create_empty_state_action_row(filter);
        group.add(&empty_row);
    } else {
        for vault_name in filtered_vaults {
            let vault_row = create_vault_action_row(vault_name, content_area);
            group.add(&vault_row);
        }
    }

    group
}

/// Creates a vault list using ListBox for better libadwaita integration
fn create_vault_list(content_area: &Box, filter: &str) -> ListBox {
    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);
    list_box.add_css_class("navigation-sidebar");

    // Get available vaults and apply filtering
    let all_vaults = get_available_vaults();
    let filtered_vaults = crate::gui::widgets::filtering::sort_vaults(&all_vaults, filter);

    if filtered_vaults.is_empty() {
        let empty_row = create_empty_state_row(filter);
        list_box.append(&empty_row);
    } else {
        for vault_name in filtered_vaults {
            let vault_row = create_vault_row(vault_name, content_area);
            list_box.append(&vault_row);
        }
    }

    list_box
}

/// Creates an empty state row for when no vaults are found
fn create_empty_state_row(filter: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);

    let label = if filter.is_empty() {
        Label::new(Some("No vaults found"))
    } else {
        Label::new(Some("No vaults match your search"))
    };
    label.add_css_class("dim-label");
    label.set_margin_top(16);
    label.set_margin_bottom(16);
    label.set_margin_start(16);
    label.set_margin_end(16);

    row.set_child(Some(&label));
    row
}

/// Creates a row for a specific vault
fn create_vault_row(vault_name: &str, content_area: &Box) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_activatable(true);

    let label = Label::new(Some(vault_name));
    label.set_halign(gtk4::Align::Start);
    label.set_margin_top(12);
    label.set_margin_bottom(12);
    label.set_margin_start(16);
    label.set_margin_end(16);

    row.set_child(Some(&label));

    // Connect click handler using a button approach since ListBoxRow doesn't have connect_activated
    let button = Button::new();
    button.set_child(Some(&label));
    button.add_css_class("flat");
    button.set_hexpand(true);
    button.set_halign(gtk4::Align::Fill);

    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    button.connect_clicked(move |_| {
        open_vault_with_gate(&content_area_clone, &vault_name_clone);
    });

    row.set_child(Some(&button));
    row
}

/// Creates an empty state action row for when no vaults are found
fn create_empty_state_action_row(filter: &str) -> adw::ActionRow {
    use adw::ActionRow;

    let row = ActionRow::new();
    row.set_activatable(false);

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
fn create_vault_action_row(vault_name: &str, content_area: &Box) -> adw::ActionRow {
    use adw::ActionRow;

    let row = ActionRow::new();
    row.set_title(vault_name);
    row.set_subtitle("Password vault");
    row.set_activatable(true);

    // Add vault icon
    let open_button = Button::new();
    open_button.set_label("Open");
    open_button.add_css_class("flat");
    open_button.set_valign(gtk4::Align::Center);
    row.add_suffix(&open_button);
    row.set_activatable_widget(Some(&open_button));

    // Connect click handler
    let vault_name_clone = vault_name.to_string();
    let content_area_clone = content_area.clone();
    open_button.connect_clicked(move |_| {
        open_vault_with_gate(&content_area_clone, &vault_name_clone);
    });

    row
}

/// Connects the search entry to update the vault list when text changes
fn connect_search_to_vault_list(
    search_entry: &SearchEntry,
    vault_list: &ListBox,
    content_area: &Box,
) {
    let vault_list_clone = vault_list.clone();
    let content_area_clone = content_area.clone();

    search_entry.connect_search_changed(move |entry| {
        let filter_text = entry.text();

        // Clear existing vault rows
        while let Some(child) = vault_list_clone.first_child() {
            vault_list_clone.remove(&child);
        }

        // Create new filtered vault list
        let all_vaults = get_available_vaults();
        let filtered_vaults =
            crate::gui::widgets::filtering::sort_vaults(&all_vaults, &filter_text);

        if filtered_vaults.is_empty() {
            let empty_row = create_empty_state_row(&filter_text);
            vault_list_clone.append(&empty_row);
        } else {
            for vault_name in filtered_vaults {
                let vault_row = create_vault_row(vault_name, &content_area_clone);
                vault_list_clone.append(&vault_row);
            }
        }
    });
}

/// Connects the search entry to update the vault preferences group when text changes
fn connect_search_to_vault_group(
    search_entry: &SearchEntry,
    vault_group: &adw::PreferencesGroup,
    content_area: &Box,
) {
    let vault_group_clone = vault_group.clone();
    let content_area_clone = content_area.clone();

    search_entry.connect_search_changed(move |entry| {
        let filter_text = entry.text().to_string();

        // Clear existing vault rows
        while let Some(child) = vault_group_clone.first_child() {
            vault_group_clone.remove(&child);
        }

        // Create new filtered vault list
        let all_vaults = get_available_vaults();
        let filtered_vaults =
            crate::gui::widgets::filtering::sort_vaults(&all_vaults, &filter_text);

        if filtered_vaults.is_empty() {
            let empty_row = create_empty_state_action_row(&filter_text);
            vault_group_clone.add(&empty_row);
        } else {
            for vault_name in filtered_vaults {
                let vault_row = create_vault_action_row(vault_name, &content_area_clone);
                vault_group_clone.add(&vault_row);
            }
        }
    });
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

/// Finds the sidebar widget from the content area by traversing up the widget hierarchy
pub fn find_sidebar_from_content_area(content_area: &Box) -> Option<Box> {
    // Traverse up: content_area -> main_content -> paned -> get start_child (sidebar)
    if let Some(main_content) = content_area.parent().and_then(|p| p.downcast::<Box>().ok()) {
        if let Some(paned) = main_content
            .parent()
            .and_then(|p| p.downcast::<Paned>().ok())
        {
            if let Some(sidebar) = paned.start_child().and_then(|c| c.downcast::<Box>().ok()) {
                return Some(sidebar);
            }
        }
    }
    None
}

/// Refreshes the sidebar from the content area (convenience function)
pub fn refresh_sidebar_from_content_area(content_area: &Box) {
    // Find the paned widget and replace the sidebar entirely
    if let Some(main_content) = content_area.parent().and_then(|p| p.downcast::<Box>().ok()) {
        if let Some(paned) = main_content
            .parent()
            .and_then(|p| p.downcast::<Paned>().ok())
        {
            // Create a new sidebar
            let new_sidebar = create_sidebar_with_callbacks(content_area);

            // Replace the old sidebar with the new one
            paned.set_start_child(Some(&new_sidebar));
        }
    }
}

/// Refreshes the vault list in the sidebar
pub fn refresh_vault_list(sidebar: &Box, content_area: &Box) {
    // Find the scrolled window in the sidebar
    if let Some(scrolled_window) = sidebar
        .last_child()
        .and_then(|child| child.downcast::<ScrolledWindow>().ok())
    {
        if let Some(vault_list) = scrolled_window
            .child()
            .and_then(|child| child.downcast::<ListBox>().ok())
        {
            // Find the search entry to get current filter text
            let mut current_filter = String::new();
            if let Some(search_entry) = sidebar
                .first_child()
                .and_then(|header| header.next_sibling())
                .and_then(|child| child.downcast::<SearchEntry>().ok())
            {
                current_filter = search_entry.text().to_string();
            }

            // Clear existing vault rows
            while let Some(child) = vault_list.first_child() {
                vault_list.remove(&child);
            }

            // Recreate the vault list with updated vault list, preserving filter
            let all_vaults = get_available_vaults();
            let filtered_vaults =
                crate::gui::widgets::filtering::sort_vaults(&all_vaults, &current_filter);

            if filtered_vaults.is_empty() {
                let empty_row = create_empty_state_row(&current_filter);
                vault_list.append(&empty_row);
            } else {
                for vault_name in filtered_vaults {
                    let vault_row = create_vault_row(vault_name, content_area);
                    vault_list.append(&vault_row);
                }
            }
        }
    }
}
