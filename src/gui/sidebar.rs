use adw::prelude::*;

use crate::gui::state::{SharedAppState, ui_helpers};
use adw::HeaderBar;
use gtk4::{Box, Label, ListBox, Orientation, Paned};

// Constants for better maintainability
const SIDEBAR_WIDTH: i32 = 250;

/// Creates a sidebar with navigation items
pub fn create_sidebar(state: &SharedAppState) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 0);

    let title_label = Label::new(Some("Navigation"));
    title_label.add_css_class("title-3");

    sidebar.append(
        &HeaderBar::builder()
            .title_widget(&title_label)
            .show_end_title_buttons(false)
            .build(),
    );

    let sidebar_list = ListBox::new();
    sidebar_list.add_css_class("navigation-sidebar");

    // Initialize sidebar with state
    ui_helpers::update_sidebar_ui(state, &sidebar_list);

    sidebar.append(&sidebar_list);
    sidebar
}

/// Creates a paned layout with sidebar and main content
pub fn create_paned_layout(main_content: &Box, state: &SharedAppState) -> Paned {
    let sidebar = create_sidebar(state);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(main_content));
    paned.set_position(SIDEBAR_WIDTH);
    paned.set_shrink_start_child(false);
    paned.set_resize_start_child(false);

    paned
}
