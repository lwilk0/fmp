use adw::prelude::*;

use crate::gui::{
    dialogs::{
        common::show_update_dialog,
        welcome::{is_first_run, show_welcome_dialog},
    },
    sidebar::create_paned_layout_with_callbacks,
    views::home_view::HomeView,
};
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk4::{Box, CssProvider, Label, Orientation, gdk, style_context_add_provider_for_display};

pub fn run_gui() {
    let application = adw::Application::builder()
        .application_id("com.fmp")
        .build();
    application.connect_activate(|app| {
        run_ui(app);
    });

    application.run();
}

fn run_ui(app: &Application) {
    load_css();

    let main_content = Box::new(Orientation::Vertical, 0);

    let title_label = Label::new(None);
    title_label.set_markup("<b>Forgot My Password</b>");

    let header = HeaderBar::builder().title_widget(&title_label).build();
    main_content.append(&header);

    // Create a separate content area for dynamic content
    let content_area = Box::new(Orientation::Vertical, 12);
    content_area.add_css_class("main-content");

    HomeView::new(&content_area).create();

    main_content.append(&content_area);

    let paned_layout = create_paned_layout_with_callbacks(&main_content, &content_area);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Forgot My Password")
        .default_width(900)
        .default_height(600)
        .build();

    window.set_content(Some(&paned_layout));
    window.present();

    if is_first_run() {
        show_welcome_dialog(&window);
    }

    let window_clone = window.clone();
    glib::MainContext::default().spawn_local(async move {
        let (updateable, latest) = can_update().await;
        if updateable {
            // latest is Some(...) here
            show_update_dialog(&window_clone, latest.unwrap());
        }
    });
}

async fn can_update() -> (bool, Option<check_latest::Version>) {
    // run blocking check in background thread
    let join_res = tokio::task::spawn_blocking(|| {
        let target_crate = "forgot-my-password";
        check_latest::new_versions!(
            crate_name = target_crate,
            user_agent = check_latest::user_agent!()
        )
    })
    .await;

    let mut latest_version = None;
    let mut updatable = false;

    match join_res {
        Ok(Ok(versions)) => {
            let current = env!("CARGO_PKG_VERSION");
            if let Some(latest) = versions.max_version() {
                if latest > current {
                    updatable = true;
                    latest_version = Some(latest.clone());
                }
            }
        }
        _ => { /* treat as no update available */ }
    }

    (updatable, latest_version)
}
fn load_css() {
    let provider = CssProvider::new();
    let css_data = include_str!("style.css");

    provider.load_from_data(css_data);

    style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
