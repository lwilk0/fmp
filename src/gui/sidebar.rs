use crate::{
    gui::FmpApp,
    totp::{ensure_gate_exists, is_totp_enabled, is_totp_required},
    vault::{get_account_details, warm_up_gpg},
};
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

/// Draw a section header with title and a Refresh button.
/// Returns true if the Refresh button was clicked.
fn header_with_refresh(ui: &mut egui::Ui, title: &str, refresh_enabled: bool) -> bool {
    ui.horizontal(|ui| {
        ui.heading(title);
        let refresh_btn = egui::Button::new("Refresh");
        ui.add_enabled(refresh_enabled, refresh_btn).clicked()
    })
    .inner
}

/// Generic selectable list. Highlights the `selected` value and returns the
/// clicked item (owned `String`) if any.
fn selectable_list<'a, I>(ui: &mut egui::Ui, items: I, selected: &str) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut clicked: Option<String> = None;
    for item in items {
        if ui.selectable_label(selected == item, item).clicked() {
            clicked = Some(item.to_string());
            break;
        }
    }
    clicked
}

/// Optional "X matching" summary next to a total label.
fn render_count_row(ui: &mut egui::Ui, total_label: String, matching: Option<usize>) {
    ui.horizontal(|ui| {
        ui.label(total_label);
        if let Some(m) = matching {
            ui.label(format!("• {m} matching"));
        }
    });
}

/// Compute how many entries match a filter (using the same `make_view` as lists).
fn compute_filtered_len(names: &[String], filter: &str, case_sensitive: bool) -> usize {
    FmpApp::make_view(names, filter, true, case_sensitive).len()
}

/// Apply all side effects when a vault is selected (resets state and flags).
fn select_vault(app: &mut FmpApp, vault: String) {
    app.clear_account_data();
    app.vault_name = vault;
    app.account_names.clear();
    app.account_name.clear();
    app.account_name_create.clear();
    app.change_account_info = false;
    app.change_vault_name = false;
    app.random_password = false;

    app.needs_refresh_accounts = true;

    app.totp_enabled = is_totp_enabled(&app.vault_name);
    app.totp_required = is_totp_required(&app.vault_name);
    app.show_totp_setup_popup = false;
    app.totp_secret_b32.clear();
    app.totp_otpauth_uri.clear();
    app.totp_code_input.clear();
    app.totp_verified_until = None;

    if app.totp_required {
        app.show_totp_popup = true;
    }

    if let Err(e) = ensure_gate_exists(&app.vault_name) {
        log::error!("Failed to ensure gate file: {e}");
    }
    if !app.totp_required {
        if let Err(e) = warm_up_gpg(&app.vault_name) {
            app.toasts
                .error(format!("Unlock canceled or failed: {e}"))
                .duration(Some(std::time::Duration::from_secs(3)));
            app.vault_name.clear();
            app.account_names.clear();
            app.account_name.clear();
        }
    }
}

/// Apply side effects when an account is selected and load its details.
/// Returns Ok(()) on success, Err(()) on failure (already logged).
fn select_account(app: &mut FmpApp, account: &str) -> Result<(), ()> {
    app.change_vault_name = false;
    app.change_account_info = false;
    app.random_password = false;
    app.account_name = account.to_string();

    match get_account_details(&app.vault_name, account) {
        Ok(userpass) => {
            app.userpass = userpass;
            Ok(())
        }
        Err(e) => {
            error!("Failed to fetch account details. Error: {e}");
            Err(())
        }
    }
}

/// Render the `Vaults` section.
fn render_vaults_section(app: &mut FmpApp, ui: &mut egui::Ui) {
    if header_with_refresh(ui, "Vaults", true) {
        app.needs_refresh_vaults = true;
    }

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

    if let Some(vault) = selectable_list(ui, vault_view, &app.vault_name) {
        select_vault(app, vault);
    }

    let matching = if app.vault_filter.is_empty() {
        None
    } else {
        Some(compute_filtered_len(
            &app.vault_names,
            &app.vault_filter,
            app.sort_case_sensitive,
        ))
    };
    render_count_row(ui, format!("{} vault(s)", app.vault_names.len()), matching);
}

/// Render the `Accounts` section.
fn render_accounts_section(app: &mut FmpApp, ui: &mut egui::Ui) {
    let accounts_enabled = !app.vault_name.is_empty();

    if header_with_refresh(ui, "Accounts", accounts_enabled) {
        app.needs_refresh_accounts = true;
    }

    filter_bar(
        ui,
        &mut app.account_filter,
        &mut app.account_sort_asc,
        &mut app.sort_case_sensitive,
        "Filter accounts...",
        accounts_enabled,
        accounts_enabled,
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

    if let Some(account) = selectable_list(ui, account_view, &app.account_name) {
        if select_account(app, account.as_str()).is_err() {}
    }
}

/// Sidebar entry point: now very small, delegates to section helpers.
pub fn sidebar(app: &mut FmpApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(40.0);
        ui.separator();

        render_vaults_section(app, ui);

        ui.separator();

        render_accounts_section(app, ui);
    });
}
