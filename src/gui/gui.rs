use crate::{
    content::*,
    vault::{Locations, UserPass, get_account_details, read_directory},
};
use eframe::egui;
use log::error;
use secrecy::SecretBox;
use zeroize::Zeroize;

/// Runs the Forgot-My-Password GUI application.
///
/// # Returns
/// * `Result<(), eframe::Error>` - Returns `Ok(())` on success, or an error on failure.
///
/// # Errors
/// * If there is an error initializing the GUI, it will return an `eframe::Error`.
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Forgot-My-Password",
        options,
        Box::new(|_cc| Ok(Box::new(FmpApp::default()))),
    )
}

/// The main application state for the Forgot-My-Password GUI.
pub struct FmpApp {
    pub vault_name: String,
    pub account_name: String,

    /// Message to show to the user.
    /// - None: show nothing
    /// - Some(Ok(msg)): informational/success message
    /// - Some(Err(msg)): error message (drawn in red)
    pub output: Option<Result<String, String>>,

    pub vault_names: Vec<String>,
    pub account_names: Vec<String>,
    pub userpass: UserPass,
    pub recipient: String,
    pub vault_name_create: String,
    pub account_name_create: String,
    pub password_length: u8,

    pub change_account_info: bool,
    pub change_vault_name: bool,
    pub quit: bool,
    pub show_password: bool,
    pub show_welcome: bool,

    pub needs_refresh_vaults: bool,
    pub needs_refresh_accounts: bool,

    pub initialized: bool,

    pub vault_filter: String,
    pub account_filter: String,
    pub vault_sort_asc: bool,
    pub account_sort_asc: bool,
    pub sort_case_sensitive: bool,
}

impl Default for FmpApp {
    fn default() -> Self {
        Self {
            vault_name: String::new(),
            account_name: String::new(),
            output: None,
            vault_names: Vec::new(),
            account_names: Vec::new(),
            userpass: UserPass::default(),
            recipient: String::new(),
            vault_name_create: String::new(),
            account_name_create: String::new(),
            password_length: 0,

            change_account_info: false,
            change_vault_name: false,
            quit: false,
            show_password: false,
            show_welcome: false,

            needs_refresh_vaults: true,
            needs_refresh_accounts: false,
            initialized: false,

            vault_filter: String::new(),
            account_filter: String::new(),
            vault_sort_asc: true,
            account_sort_asc: true,
            sort_case_sensitive: false,
        }
    }
}

/// Implementation of methods for the `FmpApp` struct to handle fetching vault and account names.
impl FmpApp {
    /// Find all the vault names.
    pub fn fetch_vault_names(&mut self) {
        if let Ok(locations) = Locations::new("", "") {
            if let Ok(names) = read_directory(&locations.fmp_location.join("vaults")) {
                self.vault_names = names;
                self.output = None;
            } else {
                self.output = Some(Err("Failed to fetch vault names.".to_string()));
            }
        }
    }

    /// Find all the account names in a vault.
    pub fn fetch_account_names(&mut self) {
        if let Ok(locations) = Locations::new(&self.vault_name, "") {
            if let Ok(names) = read_directory(&locations.vault_location) {
                self.account_names = names;
                self.output = None;
            } else {
                self.output = Some(Err("Failed to fetch account names.".to_string()));
            }
        }
    }

    /// Clear the data in userpass
    pub fn clear_account_data(&mut self) {
        self.userpass.username.clear();
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
    fn make_view<'a>(
        names: &'a [String],
        filter: &str,
        asc: bool,
        case_sensitive: bool,
    ) -> Vec<&'a str> {
        let mut view: Vec<&str> = if filter.is_empty() {
            names.iter().map(|s| s.as_str()).collect()
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

/// Clears sensitive info on exit
impl Drop for FmpApp {
    fn drop(&mut self) {
        self.userpass.password.zeroize();
    }
}

/// Implementation of the `eframe::App` trait for the `FmpApp` struct to handle GUI updates.
impl eframe::App for FmpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Vaults");
                    if ui.button("Refresh").clicked() {
                        self.needs_refresh_vaults = true;
                    }
                });

                ui.horizontal(|ui| {
                    let te = egui::TextEdit::singleline(&mut self.vault_filter)
                        .hint_text("Filter vaults...");
                    ui.add(te);
                    if ui
                        .add_enabled(!self.vault_filter.is_empty(), egui::Button::new("×"))
                        .on_hover_text("Clear")
                        .clicked()
                    {
                        self.vault_filter.clear();
                    }

                    let sort_label = if self.vault_sort_asc { "A>Z" } else { "Z>A" };
                    if ui
                        .button(sort_label)
                        .on_hover_text("Toggle sort order")
                        .clicked()
                    {
                        self.vault_sort_asc = !self.vault_sort_asc;
                    }

                    let cs_label = if self.sort_case_sensitive { "Aa" } else { "aA" };
                    if ui
                        .button(cs_label)
                        .on_hover_text("Toggle case sensitivity")
                        .clicked()
                    {
                        self.sort_case_sensitive = !self.sort_case_sensitive;
                    }
                });

                let vault_view = Self::make_view(
                    &self.vault_names,
                    &self.vault_filter,
                    self.vault_sort_asc,
                    self.sort_case_sensitive,
                );

                let mut clicked_vault: Option<String> = None;
                for vault in vault_view {
                    if ui
                        .selectable_label(self.vault_name == vault, vault)
                        .clicked()
                    {
                        clicked_vault = Some(vault.to_string());
                        break;
                    }
                }

                if let Some(vault) = clicked_vault {
                    self.clear_account_data();
                    self.vault_name = vault;
                    self.account_names.clear();
                    self.account_name.clear();
                    self.account_name_create.clear();
                    self.change_account_info = false;
                    self.change_vault_name = false;

                    self.needs_refresh_accounts = true;
                }

                ui.horizontal(|ui| {
                    ui.label(format!("{} vault(s)", self.vault_names.len()));
                    if !self.vault_filter.is_empty() {
                        let filtered_count = Self::make_view(
                            &self.vault_names,
                            &self.vault_filter,
                            true,
                            self.sort_case_sensitive,
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
                        .add_enabled(!self.vault_name.is_empty(), refresh_btn)
                        .clicked()
                    {
                        self.needs_refresh_accounts = true;
                    }
                });

                ui.horizontal(|ui| {
                    let te = egui::TextEdit::singleline(&mut self.account_filter)
                        .hint_text("Filter accounts...");
                    ui.add_enabled(!self.vault_name.is_empty(), te);

                    if ui
                        .add_enabled(!self.account_filter.is_empty(), egui::Button::new("×"))
                        .on_hover_text("Clear")
                        .clicked()
                    {
                        self.account_filter.clear();
                    }

                    let sort_label = if self.account_sort_asc { "A>Z" } else { "Z>A" };
                    if ui
                        .add_enabled(!self.vault_name.is_empty(), egui::Button::new(sort_label))
                        .on_hover_text("Toggle sort order")
                        .clicked()
                    {
                        self.account_sort_asc = !self.account_sort_asc;
                    }

                    let cs_label = if self.sort_case_sensitive { "Aa" } else { "aA" };
                    if ui
                        .add_enabled(!self.vault_name.is_empty(), egui::Button::new(cs_label))
                        .on_hover_text("Toggle case sensitivity")
                        .clicked()
                    {
                        self.sort_case_sensitive = !self.sort_case_sensitive;
                    }
                });

                let account_view = if self.vault_name.is_empty() {
                    Vec::<&str>::new()
                } else {
                    Self::make_view(
                        &self.account_names,
                        &self.account_filter,
                        self.account_sort_asc,
                        self.sort_case_sensitive,
                    )
                };

                let mut clicked_account: Option<String> = None;
                for account in account_view {
                    if ui
                        .selectable_label(self.account_name == account, account)
                        .clicked()
                    {
                        clicked_account = Some(account.to_string());
                        break;
                    }
                }

                if let Some(account) = clicked_account {
                    self.change_vault_name = false;
                    self.change_account_info = false;
                    self.account_name = account.clone();
                    self.userpass = match get_account_details(&self.vault_name, &account) {
                        Ok(userpass) => userpass,
                        Err(e) => {
                            error!("Failed to fetch account details. Error: {e}");
                            return;
                        }
                    };
                }

                if let Some(msg) = &self.output {
                    ui.separator();
                    match msg {
                        Ok(info) => ui.label(info),
                        Err(err_msg) => ui.colored_label(egui::Color32::RED, err_msg),
                    };
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Forgot-My-Password").size(32.0));
            });

            ui.add_space(8.0);

            if self.change_vault_name {
                alter_vault_name(self, ui);
            } else if self.change_account_info {
                alter_account_information(self, ui);
            } else if self.vault_name.is_empty() {
                nothing_selected(self, ui);
            } else if self.account_name.is_empty() {
                vault_selected(self, ui);
            } else {
                account_selected(self, ui);
            }
        });

        if self.quit {
            egui::Window::new("Confirm Exit")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to quit?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            self.userpass.password.zeroize();
                            std::process::exit(0);
                        }
                        if ui.button("No").clicked() {
                            self.quit = false;
                        }
                    });
                });
        }

        if self.show_welcome {
            egui::Window::new("Welcome to Forgot-My-Password!")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Welcome!");
                    ui.label("Thank you for installing Forgot-My-Password.\n\nGet started by creating your first vault and adding an account to it.");
                    if ui.button("Get Started").clicked() {
                        self.show_welcome = false;
                    }
                });
        }
    }
}
