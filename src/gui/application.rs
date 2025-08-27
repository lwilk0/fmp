use adw::prelude::*;

use crate::gui::sidebar::create_paned_layout;
use crate::gui::state::{create_app_state, ui_helpers};
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk4::{Box, Label, Orientation};

pub fn run_gui() {
    let application = Application::builder().application_id("com.fmp").build();
    application.connect_activate(|app| {
        run_ui(app);
    });

    application.run();
}

fn run_ui(app: &Application) {
    // Create shared application state
    let app_state = create_app_state();

    let main_content = Box::new(Orientation::Vertical, 0);

    let title_label = Label::new(None);
    title_label.set_markup("<b>Forgot My Password</b>");
    let header = HeaderBar::builder().title_widget(&title_label).build();
    main_content.append(&header);

    // Create a separate content area for dynamic content
    let content_area = Box::new(Orientation::Vertical, 12);
    content_area.add_css_class("main-content");

    // Initialize main content with current state
    ui_helpers::update_main_content(&app_state, &content_area);

    main_content.append(&content_area);

    let paned = create_paned_layout(&main_content, &app_state);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Forgot My Password")
        .default_width(800)
        .default_height(600)
        .content(&paned)
        .build();

    window.present();
}
