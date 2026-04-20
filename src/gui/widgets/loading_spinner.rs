use adw::prelude::*;
use gtk4::{Box, Label, Orientation, Spinner};
use std::cell::RefCell;
use std::rc::Rc;

pub struct LoadingOverlay {
    overlay_box: Box,
    spinner: Spinner,
    message_label: Label,
}

impl LoadingOverlay {
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

    pub fn widget(&self) -> &Box {
        &self.overlay_box
    }

    pub fn show(&self, message: &str) {
        self.message_label.set_text(message);
        self.spinner.start();
        self.overlay_box.set_visible(true);
    }

    pub fn hide(&self) {
        self.spinner.stop();
        self.overlay_box.set_visible(false);
    }
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
    pub fn new(label: &str, loading_text: &str) -> Self {
        let button = gtk4::Button::new();
        let is_loading = Rc::new(RefCell::new(false));

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

    pub fn button(&self) -> &gtk4::Button {
        &self.button
    }
}

const LOADING_BUTTON_DATA_KEY: &str = "fmp_loading_button_data";

struct LoadingButtonData {
    spinner: Spinner,
    label: Label,
    original_text: String,
    loading_text: String,
}

pub fn create_loading_button(label: &str, loading_text: &str) -> (gtk4::Button, Rc<RefCell<bool>>) {
    let loading_button = LoadingButton::new(label, loading_text);
    let button = loading_button.button().clone();
    let is_loading = loading_button.is_loading.clone();

    let data = LoadingButtonData {
        spinner: loading_button.spinner.clone(),
        label: loading_button.label.clone(),
        original_text: loading_button.original_text.clone(),
        loading_text: loading_button.loading_text.clone(),
    };

    unsafe {
        button.set_data(LOADING_BUTTON_DATA_KEY, data);
    }

    (button, is_loading)
}

pub fn set_button_loading_state(button: &gtk4::Button, loading: bool) {
    unsafe {
        let Some(data) = button.data::<LoadingButtonData>(LOADING_BUTTON_DATA_KEY) else {
            return;
        };
        let data = data.as_ref();

        if loading {
            data.spinner.set_visible(true);
            data.spinner.start();
            data.label.set_text(&data.loading_text);
            button.set_sensitive(false);
        } else {
            data.spinner.stop();
            data.spinner.set_visible(false);
            data.label.set_text(&data.original_text);
            button.set_sensitive(true);
        }
    }
}
