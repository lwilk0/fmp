use crate::gui::state::{SharedAppState, SidebarItem};
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Example of how to use the state management system
pub fn create_example_with_state(state: &SharedAppState) -> Box {
    let container = Box::new(Orientation::Vertical, 12);
    container.add_css_class("main-content");

    // Display current state
    let state_label = Label::new(None);
    update_state_display(&state_label, state);
    container.append(&state_label);

    // Button to change view
    let change_view_btn = Button::with_label("Switch to Passwords");
    let state_clone = state.clone();
    let label_clone = state_label.clone();
    change_view_btn.connect_clicked(move |_| {
        {
            let mut state_mut = state_clone.borrow_mut();
            state_mut.set_current_view("passwords");
        }
        update_state_display(&label_clone, &state_clone);
    });
    container.append(&change_view_btn);

    // Button to add new sidebar item
    let add_item_btn = Button::with_label("Add Custom Item");
    let state_clone2 = state.clone();
    let label_clone2 = state_label.clone();
    add_item_btn.connect_clicked(move |_| {
        {
            let mut state_mut = state_clone2.borrow_mut();
            let new_item = SidebarItem {
                id: "custom".to_string(),
                label: "Custom Item".to_string(),
                icon: None,
                is_active: false,
            };
            state_mut.add_sidebar_item(new_item);
        }
        update_state_display(&label_clone2, &state_clone2);
    });
    container.append(&add_item_btn);

    // Button to toggle lock state
    let lock_btn = Button::with_label("Toggle Lock");
    let state_clone3 = state.clone();
    let label_clone3 = state_label.clone();
    lock_btn.connect_clicked(move |_| {
        {
            let mut state_mut = state_clone3.borrow_mut();
            state_mut.toggle_lock();
        }
        update_state_display(&label_clone3, &state_clone3);
    });
    container.append(&lock_btn);

    // Search functionality
    let search_entry = gtk4::Entry::new();
    search_entry.set_placeholder_text(Some("Search..."));
    let state_clone4 = state.clone();
    let label_clone4 = state_label.clone();
    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_string();
        {
            let mut state_mut = state_clone4.borrow_mut();
            state_mut.set_search_query(query);
        }
        update_state_display(&label_clone4, &state_clone4);
    });
    container.append(&search_entry);

    container
}

fn update_state_display(label: &Label, state: &SharedAppState) {
    let state_ref = state.borrow();
    let display_text = format!(
        "Current View: {}\nSidebar Items: {}\nSearch Query: '{}'\nLocked: {}\nActive Item: {}",
        state_ref.current_view,
        state_ref.sidebar_items.len(),
        state_ref.search_query,
        state_ref.is_locked,
        state_ref
            .get_active_item()
            .map(|item| item.label.as_str())
            .unwrap_or("None")
    );
    label.set_text(&display_text);
}

/// Advanced example: State with callbacks and event handling
pub struct StateManager {
    state: SharedAppState,
    callbacks: Vec<Box<dyn Fn(&SharedAppState)>>,
}

impl StateManager {
    pub fn new(state: SharedAppState) -> Self {
        Self {
            state,
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: Fn(&SharedAppState) + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn update_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut crate::gui::state::AppState),
    {
        {
            let mut state_mut = self.state.borrow_mut();
            updater(&mut *state_mut);
        }

        // Notify all callbacks
        for callback in &self.callbacks {
            callback(&self.state);
        }
    }

    pub fn get_state(&self) -> &SharedAppState {
        &self.state
    }
}

/// Example of reactive UI updates
pub fn create_reactive_example() -> (Box, StateManager) {
    let state = crate::gui::state::create_app_state();
    let mut manager = StateManager::new(state.clone());

    let container = Box::new(Orientation::Vertical, 12);

    // Create UI elements
    let status_label = Label::new(Some("Ready"));
    let counter_label = Label::new(Some("Items: 0"));

    // Add reactive callbacks
    let status_clone = status_label.clone();
    manager.add_callback(move |state| {
        let state_ref = state.borrow();
        let status = if state_ref.is_locked {
            "🔒 Locked"
        } else {
            "🔓 Unlocked"
        };
        status_clone.set_text(status);
    });

    let counter_clone = counter_label.clone();
    manager.add_callback(move |state| {
        let state_ref = state.borrow();
        counter_clone.set_text(&format!("Items: {}", state_ref.sidebar_items.len()));
    });

    // Create buttons that trigger state changes
    let lock_button = Button::with_label("Toggle Lock");
    let manager_rc = Rc::new(RefCell::new(manager));
    let manager_clone = manager_rc.clone();
    lock_button.connect_clicked(move |_| {
        let manager_ref = manager_clone.borrow();
        manager_ref.update_state(|state| {
            state.toggle_lock();
        });
    });

    let add_button = Button::with_label("Add Item");
    let manager_clone2 = manager_rc.clone();
    add_button.connect_clicked(move |_| {
        let manager_ref = manager_clone2.borrow();
        manager_ref.update_state(|state| {
            let item = SidebarItem {
                id: format!("item_{}", state.sidebar_items.len()),
                label: format!("Item {}", state.sidebar_items.len() + 1),
                icon: None,
                is_active: false,
            };
            state.add_sidebar_item(item);
        });
    });

    container.append(&status_label);
    container.append(&counter_label);
    container.append(&lock_button);
    container.append(&add_button);

    // Extract manager from Rc<RefCell<_>>
    let final_manager = manager_rc.try_borrow_mut().unwrap();
    // This is a bit awkward, but demonstrates the pattern
    // In real usage, you'd structure this differently

    (container, StateManager::new(state))
}
