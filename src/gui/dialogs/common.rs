use adw::prelude::*;
use gtk4::{
    Box as GtkBox, Button, ButtonsType, Dialog, Label, MessageDialog, MessageType, Orientation,
    ResponseType,
};

/// Shows a confirmation dialog for dangerous actions
/// Takes a callback that will be executed if the user confirms the action
pub fn show_confirmation_dialog<F>(
    title: &str,
    message: &str,
    confirm_label: &str,
    parent: Option<&impl IsA<gtk4::Window>>,
    on_confirm: F,
) where
    F: Fn() + 'static,
{
    let dialog = MessageDialog::new(
        parent,
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Warning,
        ButtonsType::None,
        message,
    );

    dialog.set_title(Some(title));
    dialog.add_css_class("confirmation-dialog");

    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button(confirm_label, ResponseType::Accept);

    if let Some(confirm_button) = dialog.widget_for_response(ResponseType::Accept) {
        confirm_button.add_css_class("destructive-action");
    }

    dialog.set_default_response(ResponseType::Cancel);

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            on_confirm();
        }
        dialog.close();
    });

    dialog.present();
}

/// Shows a simple error dialog
pub fn show_error_dialog(title: &str, message: &str) {
    let dialog = Dialog::new();
    dialog.set_title(Some(title));
    dialog.set_modal(true);
    dialog.set_default_size(300, 150);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let message_label = Label::new(Some(message));
    message_label.set_wrap(true);
    message_label.set_halign(gtk4::Align::Center);
    content_box.append(&message_label);

    let ok_button = Button::new();
    ok_button.set_label("OK");
    ok_button.add_css_class("suggested-action");
    ok_button.set_halign(gtk4::Align::Center);

    let dialog_clone = dialog.clone();
    ok_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    content_box.append(&ok_button);
    dialog.set_child(Some(&content_box));
    dialog.present();
}

/// Shows the update dialog when update is available
pub fn show_update_dialog(parent: &adw::ApplicationWindow, latest: check_latest::Version) {
    let dialog = Dialog::new();

    dialog.set_title(Some("Update Available"));
    dialog.set_modal(true);
    dialog.set_transient_for(Some(parent));
    dialog.set_default_size(300, 0);
    dialog.set_resizable(true);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Update {} Available", latest)));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let warning = Label::new(Some(&format!(
        "Version {} is available from Codeberg.org and Crates.io",
        latest
    )));
    warning.add_css_class("error");
    warning.set_wrap(true);
    warning.set_halign(gtk4::Align::Center);
    content_box.append(&warning);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);

    let cancel_button = Button::new();
    cancel_button.set_label("Close");
    cancel_button.add_css_class("flat");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    button_box.append(&cancel_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}
