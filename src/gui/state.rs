use gtk4::prelude::*;
use gtk4::{Label, ListBox, ListBoxRow};
use std::cell::RefCell;
use std::rc::Rc;

/// Application state that can be shared across UI components
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_view: String,
    pub sidebar_items: Vec<SidebarItem>,
    pub search_query: String,
    pub is_locked: bool,
}

#[derive(Debug, Clone)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub is_active: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_view: "dashboard".to_string(),
            sidebar_items: vec![
                SidebarItem {
                    id: "dashboard".to_string(),
                    label: "Dashboard".to_string(),
                    icon: Some("view-grid-symbolic".to_string()),
                    is_active: true,
                },
                SidebarItem {
                    id: "passwords".to_string(),
                    label: "Passwords".to_string(),
                    icon: Some("dialog-password-symbolic".to_string()),
                    is_active: false,
                },
                SidebarItem {
                    id: "secure_notes".to_string(),
                    label: "Secure Notes".to_string(),
                    icon: Some("text-x-generic-symbolic".to_string()),
                    is_active: false,
                },
                SidebarItem {
                    id: "settings".to_string(),
                    label: "Settings".to_string(),
                    icon: Some("preferences-system-symbolic".to_string()),
                    is_active: false,
                },
            ],
            search_query: String::new(),
            is_locked: false,
        }
    }
}

/// Shared application state using Rc<RefCell<T>>
pub type SharedAppState = Rc<RefCell<AppState>>;

/// Creates a new shared application state
pub fn create_app_state() -> SharedAppState {
    Rc::new(RefCell::new(AppState::default()))
}

/// State management methods
impl AppState {
    /// Set the current active view
    pub fn set_current_view(&mut self, view_id: &str) {
        self.current_view = view_id.to_string();

        // Update sidebar items active state
        for item in &mut self.sidebar_items {
            item.is_active = item.id == view_id;
        }
    }

    /// Get the currently active sidebar item
    pub fn get_active_item(&self) -> Option<&SidebarItem> {
        self.sidebar_items.iter().find(|item| item.is_active)
    }

    /// Update search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// Toggle lock state
    pub fn toggle_lock(&mut self) {
        self.is_locked = !self.is_locked;
    }

    /// Add a new sidebar item
    pub fn add_sidebar_item(&mut self, item: SidebarItem) {
        self.sidebar_items.push(item);
    }

    /// Remove a sidebar item by id
    pub fn remove_sidebar_item(&mut self, id: &str) {
        self.sidebar_items.retain(|item| item.id != id);
    }
}

/// Helper functions for UI updates
pub mod ui_helpers {
    use super::*;

    /// Update sidebar UI based on current state
    pub fn update_sidebar_ui(state: &SharedAppState, sidebar_list: &ListBox) {
        // Clear existing items
        while let Some(child) = sidebar_list.first_child() {
            sidebar_list.remove(&child);
        }

        let state_ref = state.borrow();

        // Add items from state
        for item in &state_ref.sidebar_items {
            let row = create_sidebar_row(item);

            // Clone state for the closure
            let state_clone = state.clone();
            let item_id = item.id.clone();

            row.connect_activate(move |_| {
                let mut state_mut = state_clone.borrow_mut();
                state_mut.set_current_view(&item_id);
                println!("Switched to view: {}", item_id);
                // Here you would trigger UI updates for the main content area
            });

            sidebar_list.append(&row);
        }
    }

    /// Create a sidebar row widget from a SidebarItem
    fn create_sidebar_row(item: &SidebarItem) -> ListBoxRow {
        let row = ListBoxRow::new();
        let label = Label::new(Some(&item.label));

        // Add styling based on active state
        if item.is_active {
            row.add_css_class("sidebar-item-active");
        } else {
            row.add_css_class("sidebar-item");
        }

        // TODO: Add icon support if needed
        // if let Some(icon_name) = &item.icon {
        //     let icon = gtk4::Image::from_icon_name(icon_name);
        //     let box_widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        //     box_widget.append(&icon);
        //     box_widget.append(&label);
        //     row.set_child(Some(&box_widget));
        // } else {
        row.set_child(Some(&label));
        // }

        row
    }

    /// Update main content based on current view
    pub fn update_main_content(state: &SharedAppState, content_area: &gtk4::Box) {
        let state_ref = state.borrow();
        let current_view = &state_ref.current_view;

        // Clear existing content
        while let Some(child) = content_area.first_child() {
            if child.type_().name() != "AdwHeaderBar" {
                // Keep the header
                content_area.remove(&child);
            }
        }

        // Add content based on current view
        let content_label = Label::new(Some(&format!("Content for: {}", current_view)));
        content_label.add_css_class("title-1");
        content_area.append(&content_label);

        // TODO: Add actual content widgets based on the view
        match current_view.as_str() {
            "dashboard" => {
                let dashboard_content = Label::new(Some("Dashboard content goes here"));
                content_area.append(&dashboard_content);
            }
            "passwords" => {
                let passwords_content = Label::new(Some("Password list goes here"));
                content_area.append(&passwords_content);
            }
            "secure_notes" => {
                let notes_content = Label::new(Some("Secure notes go here"));
                content_area.append(&notes_content);
            }
            "settings" => {
                let settings_content = Label::new(Some("Settings panel goes here"));
                content_area.append(&settings_content);
            }
            _ => {
                let default_content = Label::new(Some("Unknown view"));
                content_area.append(&default_content);
            }
        }
    }
}

/// Event system for state changes
pub mod events {
    use super::*;
    use std::collections::HashMap;

    pub type EventCallback = Box<dyn Fn(&SharedAppState)>;

    pub struct EventManager {
        callbacks: HashMap<String, Vec<EventCallback>>,
    }

    impl EventManager {
        pub fn new() -> Self {
            Self {
                callbacks: HashMap::new(),
            }
        }

        pub fn subscribe(&mut self, event: &str, callback: EventCallback) {
            self.callbacks
                .entry(event.to_string())
                .or_insert_with(Vec::new)
                .push(callback);
        }

        pub fn emit(&self, event: &str, state: &SharedAppState) {
            if let Some(callbacks) = self.callbacks.get(event) {
                for callback in callbacks {
                    callback(state);
                }
            }
        }
    }
}
