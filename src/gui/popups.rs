use crate::{
    gui::FmpApp,
    totp::{disable_totp, verify_totp_code},
    vault::warm_up_gpg,
};
use egui::ColorImage;
use image::Luma;
use qrcode::QrCode;
use std::time::Duration;
use zeroize::Zeroize;

/// Displays the content quit popup.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn quit_popup(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::Window::new("Confirm Exit")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label("Are you sure you want to quit?");
            ui.horizontal(|ui| {
                if ui.button("Yes").clicked() {
                    app.userpass.password.zeroize();
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.button("No").clicked() {
                    app.quit = false;
                }
            });
        });
}

/// Displays a welcome popup for new users, featuring onboarding and GPG setup guidance.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn welcome_popup(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::Window::new("Welcome to FMP!")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.heading("Welcome!");

            ui.label("Thank you for installing FMP.\n\nGet started by creating your first vault and adding an account to it.");

            ui.add_space(12.0);

            ui.label("🔐 To keep your passwords safe, FMP uses GPG encryption. \
                   You'll need to have GPG available on your system and a valid GPG key pair for secure storage.");

            if ui.button("Learn about GPG requirements").clicked() {
        app.show_gpg_requirements_popup = true;
    }

    if app.show_gpg_requirements_popup {
        egui::Window::new("GPG Requirements")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("You'll need to install GPG (GNU Privacy Guard) if it's not already on your system.\n\
                                Generate a key with `gpg --full-generate-key`, or use an existing key.\n\
                                You can also import a key with `gpg --import your-key.asc`.");

                if ui.button("Close").clicked() {
                    app.show_gpg_requirements_popup = false;
                }
            });
    }

        ui.add_space(12.0);

        ui.hyperlink_to("Create your first vault (Walkthrough)", "https://codeberg.org/lwilko/fmp/wiki/Creating-Your-First-Vault").on_hover_text(
            "Follow a step-by-step guide to set up your first vault and add your account."
        );

        ui.add_space(16.0);

        if ui.button("Get Started").clicked() {
            app.show_welcome = false;
        }
    });
}

/// Displays a conformation popup for dangerous actions.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn confirmation_popup(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::Window::new("Dangerous Action!")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label("Are you sure?");
            ui.horizontal(|ui| {
                if ui.button("Yes").clicked() {
                    app.confirm_action = true;
                    app.show_confirm_action_popup = false;
                }
                if ui.button("No").clicked() {
                    app.show_confirm_action_popup = false;
                }
            });
        });
}

/// Render a full-screen translucent overlay that captures input to block underlying UI.
pub fn modal_blocker(ctx: &egui::Context) {
    egui::Area::new("modal_blocker".into())
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(160));
            let _ = ui.interact(
                rect,
                ui.id().with("modal_block"),
                egui::Sense::click_and_drag(),
            );
        });
}

/// Shows a TOTP verification popup when a vault requires 2FA. On success, triggers GPG unlock.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn totp_popup(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::Window::new("Two-Factor Authentication")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label("Enter your 2FA code to unlock this vault:");
            ui.add(egui::TextEdit::singleline(&mut app.totp_code_input).desired_width(120.0));
            ui.horizontal(|ui| {
                if ui.button("Verify").clicked() {
                    match verify_totp_code(&app.vault_name, &app.totp_code_input) {
                        Ok(true) => {
                            // Only consider the vault unlocked if GPG warm-up succeeds
                            match warm_up_gpg(&app.vault_name) {
                                Ok(()) => {
                                    app.totp_verified_until = Some(
                                        std::time::Instant::now()
                                            + std::time::Duration::from_secs(120),
                                    );
                                    app.totp_code_input.clear();
                                    app.show_totp_popup = false;
                                }
                                Err(e) => {
                                    app.toasts
                                        .error(format!("Unlock canceled or failed: {e}"))
                                        .duration(Some(std::time::Duration::from_secs(3)));
                                }
                            }
                        }
                        Ok(false) => {
                            app.toasts
                                .error("Invalid code.")
                                .duration(Some(std::time::Duration::from_secs(2)));
                        }
                        Err(e) => {
                            app.toasts
                                .error(format!("Verification failed: {e}"))
                                .duration(Some(std::time::Duration::from_secs(3)));
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    app.vault_name.clear();
                    app.show_totp_popup = false;
                }
            });
        });
}

#[allow(clippy::too_many_lines)]
/// Displays a popup for totp 2FA.
///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
pub fn totp_setup_popup(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::Window::new("2FA Setup")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label("Scan or add this secret in an authenticator app:");
            ui.horizontal(|ui| {
                let masked = "•".repeat(app.totp_secret_b32.len().max(6));
                let shown = if app.show_totp_secret {
                    app.totp_secret_b32.as_str()
                } else {
                    masked.as_str()
                };
                ui.label(format!("Secret (Base32): {shown}"));
                if ui
                    .button(if app.show_totp_secret { "Hide" } else { "Show" })
                    .clicked()
                {
                    app.show_totp_secret = !app.show_totp_secret;
                }
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(app.totp_secret_b32.clone());
                }
            });
            ui.horizontal(|ui| {
                ui.label("otpauth URI:");
                ui.monospace(&app.totp_otpauth_uri);
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(app.totp_otpauth_uri.clone());
                }
            });
            ui.add_space(8.0);
            ui.checkbox(&mut app.show_totp_qr, "Show QR code");
            if app.show_totp_qr {
                if let Ok(code) = QrCode::new(app.totp_otpauth_uri.as_bytes()) {
                    let size = 192; // logical pixels for display
                    let img = code
                        .render::<Luma<u8>>()
                        .min_dimensions(size, size)
                        .quiet_zone(true)
                        .build();
                    let (w, h) = (img.width() as usize, img.height() as usize);
                    let mut rgba = vec![0u8; w * h * 4];
                    for y in 0..h {
                        for x in 0..w {
                            #[allow(clippy::cast_possible_truncation)]
                            let lum = img.get_pixel(x as u32, y as u32)[0];
                            let idx = (y * w + x) * 4;
                            rgba[idx] = lum;
                            rgba[idx + 1] = lum;
                            rgba[idx + 2] = lum;
                            rgba[idx + 3] = 255;
                        }
                    }
                    let color = ColorImage::from_rgba_unmultiplied([w, h], &rgba);
                    let tex =
                        ui.ctx()
                            .load_texture("totp_qr", color, egui::TextureOptions::NEAREST);
                    #[allow(clippy::cast_precision_loss)]
                    ui.image((tex.id(), egui::vec2(w as f32, h as f32)));
                } else {
                    ui.colored_label(egui::Color32::LIGHT_RED, "Failed to generate QR");
                }
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Enter code to verify:");
                ui.add(egui::TextEdit::singleline(&mut app.totp_code_input).desired_width(100.0));
                if ui.button("Verify").clicked() {
                    match verify_totp_code(&app.vault_name, &app.totp_code_input) {
                        Ok(true) => {
                            app.totp_verified_until = Some(
                                std::time::Instant::now() + std::time::Duration::from_secs(120),
                            );
                            app.totp_code_input.clear();
                            app.show_totp_setup_popup = false;
                            app.toasts
                                .success("2FA verified.")
                                .duration(Some(Duration::from_secs(2)));
                        }
                        Ok(false) => {
                            app.toasts
                                .error("Invalid code.")
                                .duration(Some(Duration::from_secs(2)));
                        }
                        Err(e) => {
                            app.toasts
                                .error(format!("Verification failed: {e}"))
                                .duration(Some(Duration::from_secs(3)));
                        }
                    }
                }
            });

            if ui.button("Cancel").clicked() {
                app.show_totp_setup_popup = false;
                match disable_totp(&app.vault_name) {
                    Ok(()) => {
                        app.totp_enabled = false;
                        app.show_totp_setup_popup = false;
                        app.totp_secret_b32.clear();
                        app.totp_otpauth_uri.clear();
                        app.totp_code_input.clear();
                        app.totp_verified_until = None;
                        app.toasts
                            .success("2FA disabled for this vault.")
                            .duration(Some(Duration::from_secs(2)));
                    }
                    Err(e) => {
                        app.toasts
                            .error(format!("Failed to disable 2FA: {e}"))
                            .duration(Some(Duration::from_secs(3)));
                    }
                }
            }
        });
}
