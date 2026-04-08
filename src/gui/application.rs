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
use gpgme::{Context, Protocol};
use gtk4::{
    Box, ButtonsType, CssProvider, DialogFlags, Label, MessageDialog, MessageType, Orientation,
    gdk, gio, glib, style_context_add_provider_for_display,
};
use std::cell::RefCell;
use std::rc::Rc;

pub fn run_gui() {
    crate::crypto::disable_core_dumps();

    let application = adw::Application::builder()
        .application_id("org.codeberg.fmp")
        .build();
    application.connect_activate(|app| {
        run_ui(app);
    });

    application.run();
}

fn run_ui(app: &Application) {
    load_css();

    let ctx = match Context::from_protocol(Protocol::OpenPgp) {
        Ok(ctx) => Rc::new(RefCell::new(ctx)),
        Err(e) => {
            let dialog = MessageDialog::new(
                None::<&gtk4::Window>,
                DialogFlags::MODAL,
                MessageType::Error,
                ButtonsType::Close,
                &format!(
                    "FMP requires GPG (GnuPG) to be installed and the OpenPGP backend to be available.\n\nError: {e}\n\nPlease install GnuPG and restart the application."
                ),
            );
            dialog.set_title(Some("GPG Not Available"));
            dialog.connect_response(move |_, _| {
                std::process::exit(1);
            });
            dialog.present();
            return;
        }
    };

    let main_content = Box::new(Orientation::Vertical, 0);

    let title_label = Label::new(None);
    title_label.set_markup("<b>Forgot My Password</b>");

    let header = HeaderBar::builder().title_widget(&title_label).build();
    main_content.append(&header);

    // Create a separate content area for dynamic content
    let content_area = Box::new(Orientation::Vertical, 12);
    content_area.add_css_class("main-content");

    HomeView::new(&content_area).create(ctx.clone());

    main_content.append(&content_area);

    let paned_layout =
        create_paned_layout_with_callbacks(&main_content, &content_area, ctx.clone());

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

    // I will not be happy if I get another *mut c_void` cannot be sent between threads safely, I hate async!!!! IT WORKS WOOOO
    glib::spawn_future_local(glib::clone!(
        #[weak]
        window,
        async move {
            // Spawn the blocking task on GLib's thread pool
            let result = gio::spawn_blocking(move || can_update_blocking()).await;

            // Match on the result in case the thread panicked
            if let Ok((updatable, latest_version)) = result {
                if updatable {
                    if let Some(version) = latest_version {
                        show_update_dialog(&window, version);
                    }
                }
            }
        }
    ));
}

fn can_update_blocking() -> (bool, Option<check_latest::Version>) {
    let target_crate = "forgot-my-password";
    let mut latest_version = None;
    let mut updatable = false;

    if let Ok(versions) = check_latest::new_versions!(
        crate_name = target_crate,
        user_agent = check_latest::user_agent!()
    ) {
        let current = env!("CARGO_PKG_VERSION");
        if let Some(latest) = versions.max_version() {
            if latest > current {
                updatable = true;
                latest_version = Some(latest.clone());
            }
        }
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
