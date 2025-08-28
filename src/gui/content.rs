use adw::prelude::*;

use gtk4::{Box, Label};

/// Shows the home view in the content area
pub fn show_home_view(content_area: &Box) {
    clear_content(content_area);

    let title = Label::new(Some("Home"));
    title.add_css_class("title-1");
    title.set_margin_bottom(20);
    content_area.append(&title);

    let placeholder = Label::new(Some("Home panel will be implemented here"));
    placeholder.add_css_class("dim-label");
    content_area.append(&placeholder);
}

/// Shows the settings view in the content area
pub fn show_settings_view(content_area: &Box) {
    clear_content(content_area);

    let title = Label::new(Some("Settings"));
    title.add_css_class("title-1");
    title.set_margin_bottom(20);
    content_area.append(&title);

    let placeholder = Label::new(Some("Settings panel will be implemented here"));
    placeholder.add_css_class("dim-label");
    content_area.append(&placeholder);
}

/// Clears all content from the content area
fn clear_content(content_area: &Box) {
    while let Some(child) = content_area.first_child() {
        content_area.remove(&child);
    }
}

/// Shows a specific vault view (placeholder for now)
pub fn show_vault_view(content_area: &Box, vault_name: &str) {
    // Clear existing content
    while let Some(child) = content_area.first_child() {
        content_area.remove(&child);
    }

    // Add vault-specific content
    let title = Label::new(Some(vault_name));
    title.add_css_class("title-1");
    title.set_margin_bottom(20);
    content_area.append(&title);

    let placeholder = Label::new(Some("Vault content will be implemented here"));
    placeholder.add_css_class("dim-label");
    content_area.append(&placeholder);
}
