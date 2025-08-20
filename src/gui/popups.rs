use crate::gui::FmpApp;
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

/// Displays a welcome screen for new users.
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
                if ui.button("Get Started").clicked() {
                    app.show_welcome = false;
                }
            });
}

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
