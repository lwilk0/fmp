use gtk4::prelude::*;
use gtk4::{Box, Entry, EntryIconPosition, Orientation};
use std::cmp::Reverse;

pub fn sort_vaults<'a>(names: &'a [String], filter: &str) -> Vec<&'a str> {
    let mut view: Vec<&str> = if filter.is_empty() {
        names.iter().map(std::string::String::as_str).collect()
    } else {
        let filter_lc = filter.to_ascii_lowercase();
        names
            .iter()
            .filter_map(|s| {
                if s.to_ascii_lowercase().contains(&filter_lc) {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect()
    };

    view.sort_by_cached_key(|s| Reverse(s.to_ascii_lowercase()));

    view
}

/// Creates a responsive filter bar with search entry that adapts to sidebar width
pub fn create_filter_bar(filter_text: &str, hint: &str, input_enabled: bool) -> Box {
    let container = Box::new(Orientation::Horizontal, 6);
    container.set_halign(gtk4::Align::Fill);
    container.add_css_class("sidebar-filter-bar");

    // Search entry - make it expand to fill available space
    let entry = Entry::new();
    entry.set_placeholder_text(Some(hint));
    entry.set_hexpand(true);
    entry.set_sensitive(input_enabled);
    entry.set_text(filter_text);
    entry.add_css_class("sidebar-search-entry");

    // Add search icon
    entry.set_icon_from_icon_name(
        gtk4::EntryIconPosition::Primary,
        Some("system-search-symbolic"),
    );

    container.append(&entry);
    container
}
