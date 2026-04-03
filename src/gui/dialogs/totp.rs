/// All this code is awful, sorry future me. There must be a better way????
use crate::{
    gui::{dialogs::common::show_confirmation_dialog, views::vault_view::VaultView},
    totp::{
        confirm_totp_setup, disable_totp, get_totp_qr_info, prepare_totp_setup, verify_totp_code,
        verify_totp_code_with_secret,
    },
};

use adw::{ActionRow, PreferencesGroup, PreferencesWindow, prelude::*};
use gtk4::{
    Box as GtkBox, Button, Dialog, Entry, Image, Label, Orientation,
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::{self, Bytes},
};
use image::{DynamicImage, Luma};
use qrcode::QrCode;

/// Shows the TOTP setup dialog for enabling 2FA on a vault
pub fn show_totp_setup_dialog(vault_name: &str, content_area: &GtkBox) {
    let totp_window = PreferencesWindow::new();
    totp_window.set_title(Some("Enable Two-Factor Authentication"));
    totp_window.set_modal(true);
    totp_window.set_default_size(600, 650);
    totp_window.set_search_enabled(false);

    let page = adw::PreferencesPage::new();
    page.set_title("Two-Factor Authentication");
    page.set_icon_name(Some("security-high-symbolic"));

    let setup_group = PreferencesGroup::new();
    setup_group.set_title(&format!("Enable 2FA for \"{vault_name}\""));
    setup_group.set_description(Some("Secure your vault with two-factor authentication"));

    let instructions_row = ActionRow::new();
    instructions_row.set_title("Setup Instructions");
    instructions_row.set_subtitle(
        "Scan the QR code with your authenticator app, then enter the verification code",
    );
    setup_group.add(&instructions_row);

    // QR Code group (will be populated after setup)
    let qr_group = PreferencesGroup::new();
    qr_group.set_title("QR Code");
    qr_group.set_description(Some("Scan this code with your authenticator app"));

    // Show loading message initially
    let loading_row = ActionRow::new();
    loading_row.set_title("Generating QR code...");
    loading_row.set_subtitle("Please wait while the 2FA QR code is generated...");
    qr_group.add(&loading_row);

    // Verification group (will be populated after setup)
    let verification_group = PreferencesGroup::new();
    verification_group.set_title("Verification");
    verification_group.set_description(Some("Enter the code from your authenticator app"));

    page.add(&setup_group);
    page.add(&qr_group);
    page.add(&verification_group);

    totp_window.add(&page);
    totp_window.present();

    // Prepare TOTP secret and QR code after showing dialog (without enabling yet)
    match prepare_totp_setup(vault_name) {
        Ok((secret, secret_b32, otpauth_uri)) => {
            // Remove loading message
            qr_group.remove(&loading_row);

            let qr_row = ActionRow::new();
            qr_row.set_title("QR Code");
            qr_row.set_subtitle(&format!("Secret: {secret_b32}"));

            if let Ok(qr_image) = generate_qr_code_image(&otpauth_uri) {
                let qr_image_widget = Image::from_pixbuf(Some(&qr_image));
                qr_image_widget.set_pixel_size(200);
                qr_row.set_child(Some(&qr_image_widget));
            }

            qr_group.add(&qr_row);

            // Add verification input
            let verification_row = ActionRow::new();
            verification_row.set_title("Verification Code");
            verification_row.set_subtitle("Enter the 6-digit code from your authenticator app");

            let code_entry = Entry::new();
            code_entry.set_placeholder_text(Some("000000"));
            code_entry.set_max_length(6);
            code_entry.set_input_purpose(gtk4::InputPurpose::Digits);
            code_entry.set_valign(gtk4::Align::Center);
            code_entry.set_size_request(120, -1);

            let verify_button = Button::new();
            verify_button.set_label("Enable 2FA");
            verify_button.add_css_class("suggested-action");
            verify_button.set_valign(gtk4::Align::Center);

            let button_box = GtkBox::new(Orientation::Horizontal, 8);
            button_box.append(&code_entry);
            button_box.append(&verify_button);
            verification_row.add_suffix(&button_box);

            let vault_name_clone = vault_name.to_string();
            let secret_clone = secret.clone();
            let window_clone = totp_window.clone();
            let content_area_clone = content_area.clone();

            verify_button.connect_clicked(move |_| {
                let code = code_entry.text();
                if code.len() == 6 {
                    match verify_totp_code_with_secret(&secret_clone, &code) {
                        Ok(true) => {
                            if let Err(e) = confirm_totp_setup(&vault_name_clone, &secret_clone) {
                                log::error!("Failed to confirm TOTP setup: {e}");
                            } else {
                                VaultView::new(&content_area_clone, &vault_name_clone).create();
                                window_clone.close();
                            }
                        }
                        Ok(false) => {
                            code_entry.add_css_class("error");
                            // Remove error class after a delay
                            let entry_clone = code_entry.clone();
                            glib::timeout_add_local(std::time::Duration::from_secs(3), move || {
                                entry_clone.remove_css_class("error");
                                glib::ControlFlow::Break
                            });
                        }
                        Err(e) => {
                            log::error!("TOTP verification error: {e}");
                        }
                    }
                }
            });

            verification_group.add(&verification_row);
        }
        Err(e) => {
            // Remove loading message and show error
            qr_group.remove(&loading_row);

            let error_row = ActionRow::new();
            error_row.set_title("Setup Failed");
            error_row.set_subtitle(&format!("Failed to prepare 2FA: {e}"));
            qr_group.add(&error_row);
        }
    }
}

/// Shows the TOTP management dialog for an already enabled vault
pub fn show_totp_management_dialog(vault_name: &str, content_area: &GtkBox) {
    let totp_window = PreferencesWindow::new();
    totp_window.set_title(Some("Manage Two-Factor Authentication"));
    totp_window.set_modal(true);
    totp_window.set_default_size(600, 500);
    totp_window.set_search_enabled(false);

    let page = adw::PreferencesPage::new();
    page.set_title("Two-Factor Authentication");
    page.set_icon_name(Some("security-high-symbolic"));

    let status_group = PreferencesGroup::new();
    status_group.set_title(&format!("2FA Settings for \"{vault_name}\""));
    status_group.set_description(Some("Manage your two-factor authentication settings"));

    let status_row = ActionRow::new();
    status_row.set_title("Status");
    status_row.set_subtitle("Two-factor authentication is currently enabled");

    let status_icon = Image::from_icon_name("emblem-ok-symbolic");
    status_icon.add_css_class("success");
    status_row.add_prefix(&status_icon);
    status_group.add(&status_row);

    match get_totp_qr_info(vault_name) {
        Ok((secret, otpauth_uri)) => {
            let qr_group: PreferencesGroup = PreferencesGroup::new();
            qr_group.set_title("Backup QR Code");
            qr_group.set_description(Some("Use this QR code to set up 2FA on additional devices"));

            let qr_row = ActionRow::new();
            qr_row.set_title("QR Code");
            qr_row.set_subtitle(&format!("Secret: {secret}"));

            if let Ok(qr_image) = generate_qr_code_image(&otpauth_uri) {
                let qr_image_widget = Image::from_pixbuf(Some(&qr_image));
                qr_image_widget.set_pixel_size(200);
                qr_row.set_child(Some(&qr_image_widget));
            }

            qr_group.add(&qr_row);

            let actions_group = PreferencesGroup::new();
            actions_group.set_title("Actions");
            actions_group.set_description(Some("Manage your two-factor authentication"));

            let disable_row = ActionRow::new();
            disable_row.set_title("Disable Two-Factor Authentication");
            disable_row.set_subtitle("Remove 2FA protection from this vault");

            let disable_button = Button::new();
            disable_button.set_label("Disable");
            disable_button.add_css_class("destructive-action");
            disable_button.set_valign(gtk4::Align::Center);

            let vault_name_clone = vault_name.to_string();
            let content_area_clone = content_area.clone();
            let window_clone = totp_window.clone();
            disable_button.connect_clicked(move |_| {
                show_confirmation_dialog(
                    "Disable Two-Factor Authentication",
                    &format!("Are you sure you want to disable 2FA for vault \"{vault_name_clone}\"?\n\nThis will make your vault less secure."),
                    "Disable 2FA",
                    Some(&window_clone),
                    {
                        let vault_name_clone2 = vault_name_clone.clone();
                        let content_area_clone2 = content_area_clone.clone();
                        let window_clone2 = window_clone.clone();
                        move || {
                            match disable_totp(&vault_name_clone2) {
                                Ok(()) => {
                                    window_clone2.close();
                                    crate::gui::sidebar::refresh_sidebar_from_content_area(&content_area_clone2);
                                    crate::gui::views::vault_view::VaultView::new(&content_area_clone2, &vault_name_clone2).create();
                                }
                                Err(e) => {
                                    log::error!("Failed to disable 2FA: {e}");
                                }
                            }
                        }
                    }
                );
            });

            disable_row.add_suffix(&disable_button);
            disable_row.set_activatable_widget(Some(&disable_button));
            actions_group.add(&disable_row);

            page.add(&status_group);
            page.add(&qr_group);
            page.add(&actions_group);
        }
        Err(e) => {
            let error_row = ActionRow::new();
            error_row.set_title("Error Loading 2FA Information");
            error_row.set_subtitle(&format!("Failed to load 2FA info: {e}"));
            status_group.add(&error_row);

            page.add(&status_group);
        }
    }

    totp_window.add(&page);
    totp_window.present();
}

/// Generates a QR code image from an otpauth URI
fn generate_qr_code_image(otpauth_uri: &str) -> Result<Pixbuf, Box<dyn std::error::Error>> {
    // Generate QR code with larger module size for better performance and visibility
    let qr_code = QrCode::new(otpauth_uri)?;

    // Use the QR code renderer's built-in scaling for much better performance
    let image = qr_code
        .render::<Luma<u8>>()
        .min_dimensions(300, 300)
        .module_dimensions(15, 15)
        .build();

    // Convert to RGB for GTK
    let rgb_image = DynamicImage::ImageLuma8(image).to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let data = rgb_image.into_raw();

    // Create GTK Pixbuf
    let bytes = Bytes::from(&data);
    let pixbuf = Pixbuf::from_bytes(
        &bytes,
        Colorspace::Rgb,
        false,
        8,
        width as i32,
        height as i32,
        (width * 3) as i32, // rowstride (3 bytes per pixel for RGB)
    );

    Ok(pixbuf)
}

/// Shows the TOTP authentication dialog for vault access
pub fn show_totp_authentication_dialog<F>(vault_name: &str, on_success: F)
where
    F: Fn() + 'static + Clone,
{
    let dialog = Dialog::new();
    dialog.set_title(Some("Two-Factor Authentication"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 250);

    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title = Label::new(Some(&format!("Access Vault '{vault_name}'")));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Center);
    content_box.append(&title);

    let instructions = Label::new(Some("Enter the 6-digit code from your authenticator app"));
    instructions.add_css_class("body");
    instructions.set_halign(gtk4::Align::Center);
    instructions.set_margin_bottom(12);
    content_box.append(&instructions);

    let code_container = GtkBox::new(Orientation::Vertical, 8);
    code_container.set_halign(gtk4::Align::Center);

    let code_entry = Entry::new();
    code_entry.set_placeholder_text(Some("000000"));
    code_entry.set_max_length(6);
    code_entry.set_width_chars(10);
    code_entry.set_halign(gtk4::Align::Center);
    code_entry.add_css_class("totp-code-entry");

    code_container.append(&code_entry);
    content_box.append(&code_container);

    let status_label = Label::new(None);
    status_label.set_halign(gtk4::Align::Center);
    content_box.append(&status_label);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::Center);
    button_box.set_margin_top(16);

    let cancel_button = Button::new();
    cancel_button.set_label("Cancel");
    cancel_button.add_css_class("flat");

    let verify_button = Button::new();
    verify_button.set_label("Enter");
    verify_button.add_css_class("suggested-action");

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    let vault_name_clone = vault_name.to_string();
    let dialog_clone = dialog.clone();
    let code_entry_clone = code_entry.clone();
    let status_label_clone = status_label.clone();
    let on_success_clone = on_success.clone();
    verify_button.connect_clicked(move |_| {
        let code = code_entry_clone.text();

        match verify_totp_code(&vault_name_clone, &code) {
            Ok(true) => {
                status_label_clone.set_text("Authentication successful!");
                status_label_clone.add_css_class("success");

                let dialog_clone2 = dialog_clone.clone();
                let on_success_clone2 = on_success_clone.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                    dialog_clone2.close();
                    on_success_clone2();
                    glib::ControlFlow::Break
                });
            }
            Ok(false) => {
                status_label_clone.set_text("Invalid code. Please try again.");
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
                code_entry_clone.select_region(0, -1);
            }
            Err(e) => {
                status_label_clone.set_text(&format!("Error: {e}"));
                status_label_clone.remove_css_class("success");
                status_label_clone.add_css_class("error");
            }
        }
    });

    let verify_button_clone = verify_button.clone();
    code_entry.connect_activate(move |_| {
        verify_button_clone.emit_clicked();
    });

    button_box.append(&cancel_button);
    button_box.append(&verify_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));
    dialog.present();
}
