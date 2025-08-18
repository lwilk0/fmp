use crate::{gui::FmpApp, vault::get_account_details};
use log::error;

///
/// # Arguments
/// * "app" - A mutable reference to the "FmpApp" instance containing the application state.
/// * "ui" - A mutable reference to the "egui::Ui" instance for rendering the user interface.
pub fn sidebar(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(40.0);
        ui.separator();

        ui.horizontal(|ui| {
            ui.heading("Vaults");
            if ui.button("Refresh").clicked() {
                app.needs_refresh_vaults = true;
            }
        });

        ui.horizontal(|ui| {
            let te = egui::TextEdit::singleline(&mut app.vault_filter)
                .hint_text("Filter vaults...")
                .desired_width(160.0);

            ui.add(te);
            if ui
                .add_enabled(!app.vault_filter.is_empty(), egui::Button::new("×"))
                .on_hover_text("Clear")
                .clicked()
            {
                app.vault_filter.clear();
            }

            let sort_label = if app.vault_sort_asc { "A>Z" } else { "Z>A" };
            if ui
                .button(sort_label)
                .on_hover_text("Toggle sort order")
                .clicked()
            {
                app.vault_sort_asc = !app.vault_sort_asc;
            }

            let cs_label = if app.sort_case_sensitive { "Aa" } else { "aA" };
            if ui
                .button(cs_label)
                .on_hover_text("Toggle case sensitivity")
                .clicked()
            {
                app.sort_case_sensitive = !app.sort_case_sensitive;
            }
        });

        let vault_view = FmpApp::make_view(
            &app.vault_names,
            &app.vault_filter,
            app.vault_sort_asc,
            app.sort_case_sensitive,
        );

        let mut clicked_vault: Option<String> = None;
        for vault in vault_view {
            if ui
                .selectable_label(app.vault_name == vault, vault)
                .clicked()
            {
                clicked_vault = Some(vault.to_string());
                break;
            }
        }

        if let Some(vault) = clicked_vault {
            app.clear_account_data();
            app.vault_name = vault;
            app.account_names.clear();
            app.account_name.clear();
            app.account_name_create.clear();
            app.change_account_info = false;
            app.change_vault_name = false;
            app.random_password = false;

            app.needs_refresh_accounts = true;
        }

        ui.horizontal(|ui| {
            ui.label(format!("{} vault(s)", app.vault_names.len()));
            if !app.vault_filter.is_empty() {
                let filtered_count = FmpApp::make_view(
                    &app.vault_names,
                    &app.vault_filter,
                    true,
                    app.sort_case_sensitive,
                )
                .len();
                ui.label(format!("• {filtered_count} matching"));
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.heading("Accounts");
            let refresh_btn = egui::Button::new("Refresh");
            if ui
                .add_enabled(!app.vault_name.is_empty(), refresh_btn)
                .clicked()
            {
                app.needs_refresh_accounts = true;
            }
        });

        ui.horizontal(|ui| {
            let te = egui::TextEdit::singleline(&mut app.account_filter)
                .hint_text("Filter accounts...")
                .desired_width(160.0);

            ui.add_enabled(!app.vault_name.is_empty(), te);

            if ui
                .add_enabled(!app.account_filter.is_empty(), egui::Button::new("×"))
                .on_hover_text("Clear")
                .clicked()
            {
                app.account_filter.clear();
            }

            let sort_label = if app.account_sort_asc { "A>Z" } else { "Z>A" };
            if ui
                .add_enabled(!app.vault_name.is_empty(), egui::Button::new(sort_label))
                .on_hover_text("Toggle sort order")
                .clicked()
            {
                app.account_sort_asc = !app.account_sort_asc;
            }

            let cs_label = if app.sort_case_sensitive { "Aa" } else { "aA" };
            if ui
                .add_enabled(!app.vault_name.is_empty(), egui::Button::new(cs_label))
                .on_hover_text("Toggle case sensitivity")
                .clicked()
            {
                app.sort_case_sensitive = !app.sort_case_sensitive;
            }
        });

        let account_view = if app.vault_name.is_empty() {
            Vec::<&str>::new()
        } else {
            FmpApp::make_view(
                &app.account_names,
                &app.account_filter,
                app.account_sort_asc,
                app.sort_case_sensitive,
            )
        };

        let mut clicked_account: Option<String> = None;
        for account in account_view {
            if ui
                .selectable_label(app.account_name == account, account)
                .clicked()
            {
                clicked_account = Some(account.to_string());
                break;
            }
        }

        if let Some(account) = clicked_account {
            app.change_vault_name = false;
            app.change_account_info = false;
            app.random_password = false;
            app.account_name = account.clone();
            app.userpass = match get_account_details(&app.vault_name, &account) {
                Ok(userpass) => userpass,
                Err(e) => {
                    error!("Failed to fetch account details. Error: {e}");
                    return;
                }
            };
        }

        if let Some(msg) = &app.output {
            ui.separator();
            match msg {
                Ok(info) => ui.label(info),
                Err(err_msg) => ui.colored_label(egui::Color32::RED, err_msg),
            };
        }
    });
}
