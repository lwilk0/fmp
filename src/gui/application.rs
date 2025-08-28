use adw::prelude::*;

use crate::gui::{content::show_home_view, sidebar::create_paned_layout_with_callbacks};
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk4::{Box, CssProvider, Label, Orientation, StyleContext, gdk};

pub fn run_gui() {
    let application = Application::builder().application_id("com.fmp").build();
    application.connect_activate(|app| {
        run_ui(app);
    });

    application.run();
}

fn run_ui(app: &Application) {
    // Load CSS styles
    load_css();

    let main_content = Box::new(Orientation::Vertical, 0);

    let title_label = Label::new(None);
    title_label.set_markup("<b>Forgot My Password</b>");
    let header = HeaderBar::builder().title_widget(&title_label).build();
    main_content.append(&header);

    // Create a separate content area for dynamic content
    let content_area = Box::new(Orientation::Vertical, 12);
    content_area.add_css_class("main-content");

    // Initialize with home view
    show_home_view(&content_area);

    main_content.append(&content_area);

    // Create sidebar with content area reference for updating
    let paned = create_paned_layout_with_callbacks(&main_content, &content_area);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Forgot My Password")
        .default_width(800)
        .default_height(600)
        .content(&paned)
        .build();

    window.present();
}

fn load_css() {
    let provider = CssProvider::new();
    let css_data = include_str!("style.css");

    provider.load_from_data(css_data);

    StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
