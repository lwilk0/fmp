use crate::{
    gui::{content::proceed_with_gate_warmup, views::home_view::HomeView},
    storage::filesystem::read_directory,
    vault::Locations,
};
use adw::{ButtonContent, Clamp, HeaderBar, PreferencesGroup, prelude::*};
use gtk4::{Box, Button, Label, Orientation, Paned, PolicyType, ScrolledWindow, SearchEntry};
use std::path::PathBuf;

const SIDEBAR_WIDTH: i32 = 270;

/// Gets the vaults directory path
fn get_vaults_directory() -> PathBuf {
    let locations = Locations::new("", "");
    locations.fmp.join("vaults")
}

/// Reads available vaults from the vaults directory
pub fn get_available_vaults() -> Vec<String> {
    let vaults_dir = get_vaults_directory();

    read_directory(&vaults_dir).unwrap_or_else(|e| {
        eprintln!(
            "Failed to read vaults directory: {} - Error: {}",
            vaults_dir.display(),
            e
        );
        Vec::new()
    })
}

/// Creates a responsive paned layout with sidebar that can update the main content
pub fn create_paned_layout_with_callbacks(main_content: &Box, content_area: &Box) -> Paned {
    let sidebar = create_sidebar_with_callbacks(content_area);

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
pub fn create_sidebar_with_callbacks(content_area: &Box) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 0);
    sidebar.add_css_class("sidebar");

    let header_bar = create_header_bar(content_area);
    sidebar.append(&header_bar);

    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search vaults..."));
    search_entry.set_margin_start(8);
    search_entry.set_margin_end(8);
    search_entry.set_margin_top(8);
    search_entry.set_margin_bottom(4);
    sidebar.append(&search_entry);

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_policy(PolicyType::Never, PolicyType::Automatic);
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let clamp = Clamp::new();
    //clamp.set_maximum_size(320);
    clamp.set_tightening_threshold(240);
    clamp.set_margin_start(8);
    clamp.set_margin_end(8);

    let vault_group = create_vault_preferences_group(content_area, "");
    clamp.set_child(Some(&vault_group));
    scrolled_window.set_child(Some(&clamp));

    connect_search_to_vault_group(&search_entry, &vault_group, content_area);

    sidebar.append(&scrolled_window);
    sidebar
}

/// Creates the header bar with navigation buttons
fn create_header_bar(content_area: &Box) -> HeaderBar {
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
    home_button.connect_clicked(move |_| HomeView::new(&content_area_home).create());

    header_bar
}

/// Creates a vault preferences group with filtering capability
fn create_vault_preferences_group(content_area: &Box, filter: &str) -> adw::PreferencesGroup {
    let group = PreferencesGroup::new();

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
fn create_vault_action_row(vault_name: &str, content_area: &Box) -> adw::ActionRow {
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
        proceed_with_gate_warmup(&content_area_clone, &vault_name_clone);
    });

    row
}

/// Connects the search entry to update the vault preferences group when text changes
fn connect_search_to_vault_group(
    search_entry: &SearchEntry,
    vault_group: &adw::PreferencesGroup,
    content_area: &Box,
) {
    use std::cell::RefCell;
    use std::rc::Rc;

    let content_area_clone = content_area.clone();
    let current_group: Rc<RefCell<adw::PreferencesGroup>> =
        Rc::new(RefCell::new(vault_group.clone()));

    let cg = current_group.clone();
    search_entry.connect_search_changed(move |entry| {
        let filter_text = entry.text().to_string();

        let new_group = create_vault_preferences_group(&content_area_clone, &filter_text);

        if let Some(clamp) = cg
            .borrow()
            .parent()
            .and_then(|p| p.downcast::<Clamp>().ok())
        {
            clamp.set_child(Some(&new_group));
        } else if let Some(scrolled) = cg
            .borrow()
            .parent()
            .and_then(|p| p.downcast::<ScrolledWindow>().ok())
        {
            scrolled.set_child(Some(&new_group));
        } else {
            if let Some(parent) = cg.borrow().parent() {
                parent.downcast::<gtk4::Widget>().ok().map(|w| {
                    w.hide();
                });
            }
        }

        *cg.borrow_mut() = new_group;
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
    button.add_css_class("flat");
    button.set_margin_start(2);
    button.set_margin_end(2);
    button
}

/// Refreshes the sidebar from the content area (convenience function)
pub fn refresh_sidebar_from_content_area(content_area: &Box) {
    if let Some(main_content) = content_area.parent().and_then(|p| p.downcast::<Box>().ok())
        && let Some(paned) = main_content
            .parent()
            .and_then(|p| p.downcast::<Paned>().ok())
    {
        let new_sidebar = create_sidebar_with_callbacks(content_area);
        paned.set_start_child(Some(&new_sidebar));
    }
}
