use adw::prelude::*;
use gtk4::{Box, Label, Orientation, Spinner};
use std::cell::RefCell;
use std::rc::Rc;

/// A loading overlay widget that can be shown/hidden over content
pub struct LoadingOverlay {
    overlay_box: Box,
    spinner: Spinner,
    message_label: Label,
}

impl LoadingOverlay {
    /// Create a new loading overlay
    pub fn new() -> Self {
        let overlay_box = Box::new(Orientation::Vertical, 16);
        overlay_box.set_halign(gtk4::Align::Center);
        overlay_box.set_valign(gtk4::Align::Center);
        overlay_box.set_hexpand(true);
        overlay_box.set_vexpand(true);
        overlay_box.add_css_class("loading-overlay");
        overlay_box.set_visible(false);

        let spinner = Spinner::new();
        spinner.set_size_request(64, 64);
        spinner.add_css_class("loading-spinner");
        spinner.set_halign(gtk4::Align::Center);

        let message_label = Label::new(Some("Loading..."));
        message_label.add_css_class("title-4");
        message_label.add_css_class("dim-label");
        message_label.set_halign(gtk4::Align::Center);

        overlay_box.append(&spinner);
        overlay_box.append(&message_label);

        Self {
            overlay_box,
            spinner,
            message_label,
        }
    }

    /// Get the widget to add to the UI
    pub fn widget(&self) -> &Box {
        &self.overlay_box
    }

    /// Show the loading overlay with a custom message
    pub fn show(&self, message: &str) {
        self.message_label.set_text(message);
        self.spinner.start();
        self.overlay_box.set_visible(true);
    }

    /// Hide the loading overlay
    pub fn hide(&self) {
        self.spinner.stop();
        self.overlay_box.set_visible(false);
    }
}

/// Show a loading spinner for an async operation
pub fn show_loading_for_operation<F, R>(
    loading_overlay: &LoadingOverlay,
    message: &str,
    operation: F,
) -> R
where
    F: FnOnce() -> R,
{
    loading_overlay.show(message);

    // Force UI update
    while gtk4::glib::MainContext::default().pending() {
        gtk4::glib::MainContext::default().iteration(false);
    }

    let result = operation();
    loading_overlay.hide();
    result
}

/// A loading button that can show a spinner
pub struct LoadingButton {
    button: gtk4::Button,
    spinner: Spinner,
    label: Label,
    original_text: String,
    loading_text: String,
    is_loading: Rc<RefCell<bool>>,
}

impl LoadingButton {
    /// Create a new loading button
    pub fn new(label: &str, loading_text: &str) -> Self {
        let button = gtk4::Button::new();
        let is_loading = Rc::new(RefCell::new(false));

        // Create button content with label and spinner
        let button_box = Box::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Center);

        let button_label = Label::new(Some(label));
        let spinner = Spinner::new();
        spinner.set_size_request(16, 16);
        spinner.set_visible(false);

        button_box.append(&spinner);
        button_box.append(&button_label);
        button.set_child(Some(&button_box));

        Self {
            button,
            spinner,
            label: button_label,
            original_text: label.to_string(),
            loading_text: loading_text.to_string(),
            is_loading,
        }
    }

    /// Get the button widget
    pub fn button(&self) -> &gtk4::Button {
        &self.button
    }

    /// Set the loading state
    pub fn set_loading(&self, loading: bool) {
        *self.is_loading.borrow_mut() = loading;
        if loading {
            self.spinner.set_visible(true);
            self.spinner.start();
            self.label.set_text(&self.loading_text);
            self.button.set_sensitive(false);
        } else {
            self.spinner.stop();
            self.spinner.set_visible(false);
            self.label.set_text(&self.original_text);
            self.button.set_sensitive(true);
        }
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        *self.is_loading.borrow()
    }
}

/// Create a loading button that shows a spinner when clicked
pub fn create_loading_button(label: &str, loading_text: &str) -> (gtk4::Button, Rc<RefCell<bool>>) {
    let loading_button = LoadingButton::new(label, loading_text);
    let button = loading_button.button().clone();
    let is_loading = loading_button.is_loading.clone();

    // Store the LoadingButton in the button's data for later access
    let loading_button_rc = Rc::new(RefCell::new(loading_button));
    unsafe {
        button.set_data("loading_button", loading_button_rc);
    }

    (button, is_loading)
}

/// Set the loading state of a loading button
pub fn set_button_loading_state(button: &gtk4::Button, loading: bool) {
    unsafe {
        if let Some(loading_button_rc) = button.data::<Rc<RefCell<LoadingButton>>>("loading_button")
        {
            let loading_button_ref = loading_button_rc.as_ref().as_ref().borrow_mut();
            loading_button_ref.set_loading(loading);
        }
    }
}
