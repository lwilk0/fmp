use crate::{gui::FmpApp, vault::get_account_details};
use log::error;

/// Small reusable UI helper that renders:
/// - a single-line text input with a hint,
/// - a Clear ('×') button,
/// - a sort order toggle ("A>Z"/"Z>A"),
/// - a case sensitivity toggle ("Aa"/"aA").
///
/// Mutates `filter`, `sort_asc`, and `case_sensitive` in-place based on user interactions.
///
/// Arguments:
/// - `ui` - egui UI handle
/// - `filter` - bound filter string
/// - `sort_asc` - whether sorting is ascending (A>Z)
/// - `case_sensitive` - whether filtering/sorting is case sensitive
/// - `hint` - placeholder/hint text for the input
/// - `input_enabled` - whether the text input is enabled
/// - `controls_enabled` - whether the sort and case-sensitivity buttons are enabled
fn filter_bar(
    ui: &mut egui::Ui,
    filter: &mut String,
    sort_asc: &mut bool,
    case_sensitive: &mut bool,
    hint: &str,
    input_enabled: bool,
    controls_enabled: bool,
) {
    ui.horizontal(|ui| {
        let text_box = egui::TextEdit::singleline(filter)
            .hint_text(hint)
            .desired_width(160.0);
        ui.add_enabled(input_enabled, text_box);

        let clear_button = egui::Button::new("×");
        if ui.add_enabled(!filter.is_empty(), clear_button).clicked() {
            filter.clear();
        }

        let sort_label = if *sort_asc { "A>Z" } else { "Z>A" };
        let sort_button = egui::Button::new(sort_label);
        if ui.add_enabled(controls_enabled, sort_button).clicked() {
            *sort_asc = !*sort_asc;
        }

        let cs_label = if *case_sensitive { "Aa" } else { "aA" };
        let cs_button = egui::Button::new(cs_label);
        if ui.add_enabled(controls_enabled, cs_button).clicked() {
            *case_sensitive = !*case_sensitive;
        }
    });
}

///
/// # Arguments
/// * `app` - A mutable reference to the `FmpApp` instance containing the application state.
/// * `ui` - A mutable reference to the `egui::Ui` instance for rendering the user interface.
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

        filter_bar(
            ui,
            &mut app.vault_filter,
            &mut app.vault_sort_asc,
            &mut app.sort_case_sensitive,
            "Filter vaults...",
            true,
            true,
        );

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

        // Replaces the previous accounts filter UI block
        let accounts_enabled = !app.vault_name.is_empty();
        filter_bar(
            ui,
            &mut app.account_filter,
            &mut app.account_sort_asc,
            &mut app.sort_case_sensitive,
            "Filter accounts...",
            accounts_enabled, // input_enabled
            accounts_enabled, // controls_enabled
        );

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
