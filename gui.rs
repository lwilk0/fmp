use crate::{
    content::{
        account_selected, alter_account_information, alter_vault_name, nothing_selected,
        random_password, vault_selected,
    },
    popups::{
        confirmation_popup, modal_blocker, quit_popup, totp_popup, totp_setup_popup, welcome_popup,
    },
    sidebar::sidebar,
    vault::{Locations, UserPass, read_directory},
};
use eframe::egui;
use egui_notify::Toasts;
use secrecy::SecretBox;
use std::time::Duration;
use zeroize::Zeroize;

/// Runs the FMP GUI application.
///
/// # Returns
/// * `Result<(), eframe::Error>` - Returns 'Ok(())' on success, or an error on failure.
///
/// # Errors
/// * If there is an error initializing the GUI, it will return an `eframe::Error`.
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "fmp",
        options,
        Box::new(|_cc| Ok(Box::new(FmpApp::default()))),
    )
}

#[allow(clippy::struct_excessive_bools)]
/// The main application state for the FMP GUI.
pub struct FmpApp {
    pub vault_name: String,
    pub account_name: String,

    pub vault_names: Vec<String>,
    pub account_names: Vec<String>,
    pub userpass: UserPass,
    pub recipient: String,
    pub vault_name_create: String,
    pub account_name_create: String,
    pub password_length: u8,

    /// Selections for password generation
    /// 0 - characters (lower)
    /// 1 - characters (upper)
    /// 2 - numbers
    /// 3 - symbols
    /// 4 - space
    /// 5 - accented characters
    pub selections: [bool; 6],
    pub consider_characters: String,
    pub ignore_characters: String,
    pub generated_password: SecretBox<Vec<u8>>,

    pub change_account_info: bool,
    pub change_vault_name: bool,

    pub random_password: bool,
    pub show_password_account: bool,
    pub show_password_retrieve: bool,

    pub show_welcome: bool,
    pub quit: bool,
    pub confirm_action: bool,
    pub show_confirm_action_popup: bool,
    pub show_gpg_requirements_popup: bool,

    pub needs_refresh_vaults: bool,
    pub needs_refresh_accounts: bool,

    pub initialized: bool,

    pub vault_filter: String,
    pub account_filter: String,
    pub vault_sort_asc: bool,
    pub account_sort_asc: bool,
    pub sort_case_sensitive: bool,

    pub toasts: Toasts,

    pub totp_enabled: bool,
    pub totp_required: bool,
    pub show_totp_setup_popup: bool,
    pub show_totp_popup: bool,
    pub show_totp_secret: bool,
    pub show_totp_qr: bool,
    pub totp_secret_b32: String,
    pub totp_otpauth_uri: String,
    pub totp_code_input: String,
    pub totp_verified_until: Option<std::time::Instant>,
}

impl Default for FmpApp {
    fn default() -> Self {
        Self {
            vault_name: String::new(),
            account_name: String::new(),

            vault_names: Vec::new(),
            account_names: Vec::new(),
            userpass: UserPass::default(),
            recipient: String::new(),
            vault_name_create: String::new(),
            account_name_create: String::new(),
            password_length: 16,

            selections: [true, true, true, true, false, false],
            consider_characters: String::new(),
            ignore_characters: String::new(),
            generated_password: SecretBox::new(Box::new(Vec::new())),

            change_account_info: false,
            change_vault_name: false,

            random_password: false,
            show_password_account: false,
            show_password_retrieve: false,
            show_confirm_action_popup: false,
            show_gpg_requirements_popup: false,

            show_welcome: false,
            quit: false,
            confirm_action: false,

            needs_refresh_vaults: true,
            needs_refresh_accounts: false,

            initialized: false,

            vault_filter: String::new(),
            account_filter: String::new(),
            vault_sort_asc: true,
            account_sort_asc: true,
            sort_case_sensitive: false,

            toasts: Toasts::default(),

            totp_enabled: false,
            totp_required: false,
            show_totp_setup_popup: false,
            show_totp_popup: false,
            show_totp_secret: false,
            show_totp_qr: true,
            totp_secret_b32: String::new(),
            totp_otpauth_uri: String::new(),
            totp_code_input: String::new(),
            totp_verified_until: None,
        }
    }
}

/// Implementation of methods for the `FmpApp` struct to handle fetching vault and account names.
impl FmpApp {
    /// Find all the vault names.
    pub fn fetch_vault_names(&mut self) {
        let locations = Locations::new("", "");
        if let Ok(names) = read_directory(&locations.fmp.join("vaults")) {
            self.vault_names = names;
        } else {
            self.toasts
                .error("Failed to fetch vault names.")
                .duration(Some(Duration::from_secs(3)));
        }
    }

    /// Find all the account names in a vault.
    pub fn fetch_account_names(&mut self) {
        let locations = Locations::new(&self.vault_name, "");
        if let Ok(names) = read_directory(&locations.vault) {
            self.account_names = names;
        } else {
            self.toasts
                .error("Failed to fetch account names.")
                .duration(Some(Duration::from_secs(3)));
        }
    }

    /// Clear the data in userpass
    pub fn clear_account_data(&mut self) {
        self.userpass.username.clear();
        self.userpass.password.zeroize();
        self.userpass.password = SecretBox::new(Box::new(vec![]));
    }

    /// Check if this is FMPs first run.
    pub fn check_first_run(&mut self) {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("fmp_ran");

        if config_path.exists() {
            self.show_welcome = false;
        } else {
            let _ = std::fs::write(&config_path, "shown");
            self.show_welcome = true;
        }
    }

    /// Make a filtered and sorted view of provided names without mutating the source.
    pub fn make_view<'a>(
        names: &'a [String],
        filter: &str,
        asc: bool,
        case_sensitive: bool,
    ) -> Vec<&'a str> {
        let mut view: Vec<&str> = if filter.is_empty() {
            names.iter().map(std::string::String::as_str).collect()
        } else if case_sensitive {
            names
                .iter()
                .filter_map(|s| {
                    if s.contains(filter) {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            let filter_lc = filter.to_ascii_lowercase();
            names
                .iter()
                .filter_map(|s| {
                    if s.to_ascii_lowercase().contains(&filter_lc) {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect()
        };

        if case_sensitive {
            if asc {
                view.sort_unstable();
            } else {
                view.sort_unstable_by(|a, b| b.cmp(a));
            }
        } else {
            use std::cmp::Reverse;
            if asc {
                view.sort_by_cached_key(|s| s.to_ascii_lowercase());
            } else {
                view.sort_by_cached_key(|s| Reverse(s.to_ascii_lowercase()));
            }
        }

        view
    }
}

/// Implementation of the `eframe::App` trait for the `FmpApp` struct to handle GUI updates.
impl eframe::App for FmpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);

        if !self.initialized {
            self.check_first_run();
            self.needs_refresh_vaults = true;
            self.initialized = true;
        }

        if self.needs_refresh_vaults {
            self.fetch_vault_names();
            self.needs_refresh_vaults = false;
        }

        if self.needs_refresh_accounts && !self.vault_name.is_empty() {
            self.fetch_account_names();
            self.needs_refresh_accounts = false;
        }

        if let Some(until) = self.totp_verified_until {
            if std::time::Instant::now() >= until {
                self.totp_verified_until = None;
            }
        }

        egui::SidePanel::left("sidebar")
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                let modal_active = self.quit
                    || self.show_welcome
                    || self.show_confirm_action_popup
                    || self.show_totp_popup;
                if modal_active {
                    ui.disable();
                }
                sidebar(self, ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Forgot My Password").size(32.0));
                });

                ui.separator();
                ui.add_space(8.0);

                let modal_active = self.quit
                    || self.show_welcome
                    || self.show_confirm_action_popup
                    || self.show_totp_popup
                    || self.show_totp_setup_popup;
                if modal_active {
                    ui.disable();
                }

                if self.change_vault_name {
                    alter_vault_name(self, ui);
                } else if self.random_password {
                    random_password(self, ui);
                } else if self.change_account_info {
                    alter_account_information(self, ui);
                } else if self.vault_name.is_empty() {
                    nothing_selected(self, ui);
                } else if self.account_name.is_empty() {
                    vault_selected(self, ui);
                } else {
                    account_selected(self, ui);
                }

                if modal_active {
                    modal_blocker(ctx);
                }

                if self.quit {
                    quit_popup(self, ui);
                } else if self.show_welcome {
                    welcome_popup(self, ui);
                } else if self.show_confirm_action_popup {
                    confirmation_popup(self, ui);
                } else if self.show_totp_popup {
                    totp_popup(self, ui);
                } else if self.show_totp_setup_popup {
                    totp_setup_popup(self, ui);
                }
            });

        self.toasts.show(ctx);
    }
}
