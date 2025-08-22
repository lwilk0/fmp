use crate::{gui::FmpApp, totp::verify_totp_code, vault::warm_up_gpg};
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
                            app.totp_verified_until = Some(
                                std::time::Instant::now() + std::time::Duration::from_secs(120),
                            );
                            app.totp_code_input.clear();
                            app.show_totp_popup = false;

                            if let Err(e) = warm_up_gpg(&app.vault_name) {
                                app.toasts
                                    .error(format!("GPG unlock failed: {e}"))
                                    .duration(Some(std::time::Duration::from_secs(3)));
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
                ui.label(format!("Secret (Base32): {}", app.totp_secret_b32));
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
        });
}
