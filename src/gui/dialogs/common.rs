use adw::prelude::*;

/// Gets the parent window from any widget (avoids changing all signatures).
pub fn root_window(widget: &impl IsA<gtk4::Widget>) -> Option<adw::ApplicationWindow> {
    widget.root()?.downcast::<adw::ApplicationWindow>().ok()
}

pub fn show_confirmation_dialog<F>(
    title: &str,
    message: &str,
    confirm_label: &str,
    parent: Option<&impl IsA<gtk4::Window>>,
    on_confirm: F,
) where
    F: Fn() + 'static,
{
    let dialog = adw::MessageDialog::new(parent, Some(title), Some(message));
    dialog.add_response("cancel", "Cancel");
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");
    dialog.add_response("confirm", confirm_label);
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
    dialog.connect_response(Some("confirm"), move |_, _| on_confirm());
    dialog.present();
}

pub fn show_error_dialog(parent: Option<&impl IsA<gtk4::Window>>, title: &str, message: &str) {
    let dialog = adw::MessageDialog::new(parent, Some(title), Some(message));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present();
}

pub fn show_update_dialog(parent: &adw::ApplicationWindow, latest: check_latest::Version) {
    let dialog = adw::MessageDialog::new(
        Some(parent),
        Some(&format!("Update {latest} Available")),
        Some(&format!(
            "Version {latest} is available from Codeberg.org and Crates.io"
        )),
    );
    dialog.add_response("close", "Close");
    dialog.set_default_response(Some("close"));
    dialog.present();
}

/// for dialogs that need custom content:
/// header bar with Cancel/Confirm, content area below.
pub struct DialogParts {
    pub window: adw::Window,
    pub content: gtk4::Box,
    pub confirm: gtk4::Button,
}

pub fn dialog_skeleton(
    parent: &adw::ApplicationWindow,
    title: &str,
    confirm_label: &str,
    confirm_css: &str, // "suggested-action" or "destructive-action"
) -> DialogParts {
    let window = adw::Window::builder()
        .title(title)
        .modal(true)
        .transient_for(parent)
        .resizable(false)
        .build();

    let cancel = gtk4::Button::with_label("Cancel");
    let window_for_cancel = window.clone();
    cancel.connect_clicked(move |_| window_for_cancel.close());

    let confirm = gtk4::Button::with_label(confirm_label);
    confirm.add_css_class(confirm_css);
    confirm.set_receives_default(true);

    let header = adw::HeaderBar::new();
    header.pack_start(&cancel);
    header.pack_end(&confirm);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(18);
    content.set_margin_end(18);

    let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    outer.append(&header);
    outer.append(&content);
    window.set_content(Some(&outer));

    DialogParts {
        window,
        content,
        confirm,
    }
}
