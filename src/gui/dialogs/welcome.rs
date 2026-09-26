use adw::{
    ActionRow, Clamp, EntryRow, HeaderBar, PreferencesGroup, Window as AdwWindow, prelude::*,
};
use anyhow::Error;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, glib};
use std::{
    fs::{File, create_dir_all},
    path::PathBuf,
    process::Stdio,
};

/// Checks if this is the first run of the application
pub fn is_first_run() -> bool {
    let config_path = get_config_file_path();
    !config_path.exists()
}

/// Creates the first-run marker file
pub fn mark_first_run_complete() -> Result<(), std::io::Error> {
    let config_path = get_config_file_path();

    // Create the config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        create_dir_all(parent)?;
    }

    // Create the marker file
    File::create(&config_path)?;

    Ok(())
}

/// Gets the path to the first-run marker file
fn get_config_file_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("fmp").join("fmp-ran")
}

pub fn show_welcome_dialog(parent: &adw::ApplicationWindow) {
    let welcome_window = AdwWindow::new();
    welcome_window.set_title(Some("Welcome to FMP"));
    welcome_window.set_modal(true);
    welcome_window.set_transient_for(Some(parent));
    welcome_window.set_default_size(560, -1);
    welcome_window.add_css_class("welcome-dialog");
    welcome_window.set_deletable(false);

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Welcome"))));

    let content_box = GtkBox::new(Orientation::Vertical, 24);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(18);
    content_box.set_margin_end(18);

    let clamp = Clamp::new();
    clamp.set_maximum_size(600);
    clamp.set_child(Some(&content_box));

    let dialog_title = Label::new(Some("FMP"));
    dialog_title.add_css_class("title-1");
    dialog_title.set_halign(Align::Center);
    content_box.append(&dialog_title);

    let intro_label = Label::new(Some(
        "FMP is a secure local password manager.\nCheck for an existing GPG key by email or create one.",
    ));
    intro_label.set_wrap(true);
    intro_label.set_justify(gtk4::Justification::Center);
    intro_label.add_css_class("body");
    content_box.append(&intro_label);

    let existing_group = PreferencesGroup::new();
    existing_group.set_title("Use an Existing Key");

    let email_row = EntryRow::new();
    email_row.set_title("Email address");

    let check_button = Button::with_label("Check");
    check_button.set_valign(Align::Center);
    email_row.add_suffix(&check_button);

    existing_group.add(&email_row);
    content_box.append(&existing_group);

    let check_result_label = Label::new(None);
    check_result_label.set_visible(false);
    check_result_label.set_wrap(true);
    check_result_label.set_halign(Align::Start);
    check_result_label.set_margin_top(6);
    content_box.append(&check_result_label);

    let create_group = PreferencesGroup::new();
    create_group.set_title("Create a New Key");
    create_group.set_description(Some(
        "It is recommended to use default settings.\nYou do not need to use a valid email, just remember it!",
    ));

    let terminal_row = ActionRow::new();
    terminal_row.set_title("Run `gpg --full-generate-key`");
    terminal_row.set_activatable(false);

    let terminal_button = Button::with_label("Open Terminal");
    terminal_button.set_valign(Align::Center);
    terminal_row.add_suffix(&terminal_button);

    create_group.add(&terminal_row);
    content_box.append(&create_group);

    let finish_button = Button::with_label("Finish");
    finish_button.add_css_class("suggested-action");
    finish_button.add_css_class("pill");
    finish_button.set_halign(Align::Center);
    finish_button.set_margin_top(6);
    content_box.append(&finish_button);

    let outer = GtkBox::new(Orientation::Vertical, 0);
    outer.append(&header_bar);
    outer.append(&clamp);
    welcome_window.set_content(Some(&outer));
    welcome_window.present();

    let check_result_label_clone = check_result_label.clone();
    let email_row_clone = email_row.clone();
    check_button.connect_clicked(move |_| {
        check_result_label_clone.set_visible(true);
        let email = email_row_clone.text().to_string();
        if email.trim().is_empty() {
            check_result_label_clone.set_text("Please enter an email.");
            return;
        }
        check_result_label_clone.set_text("Checking for GPG key...");

        let weak_label = check_result_label_clone.downgrade();

        // Move only the plain String into the worker task. Do NOT move the WeakRef into it.
        let email_for_task = email.clone();
        let task = gio::spawn_blocking(move || {
            std::process::Command::new("gpg")
                .arg("--list-keys")
                .arg("--with-colons")
                .arg(&email_for_task)
                .stderr(Stdio::piped())
                .output()
        });

        glib::spawn_future_local(async move {
            let text = match task.await {
                Ok(Ok(out)) => {
                    if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if stdout.contains("\nuid:")
                            || stdout.contains("\npub:")
                            || stdout.contains(":uid:")
                        {
                            format!("GPG key found for {email}.")
                        } else {
                            format!("No GPG key found for {email}.")
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        format!("gpg returned an error: {stderr}")
                    }
                }
                Ok(Err(e)) => format!("Failed to run gpg: {e}"),
                Err(e) => format!("Failed to run gpg task: {:?}", e),
            };

            if let Some(label) = weak_label.upgrade() {
                label.set_text(&text);
            }
        });
    });

    let check_for_enter = check_button.clone();
    email_row.connect_entry_activated(move |_| {
        check_for_enter.activate();
    });

    terminal_button.connect_clicked(move |_| {
        #[cfg(unix)]
        let _ = launch_terminal_unix("gpg --full-generate-key")
            .map_err(|e| anyhow::anyhow!("Failed to launch terminal. Error: {}", e));

        #[cfg(windows)]
        launch_terminal_windows("gpg --full-generate-key");
    });

    let welcome_window_clone = welcome_window.clone();
    finish_button.connect_clicked(move |_| {
        if let Err(err) = mark_first_run_complete() {
            log::error!("Failed to mark first run complete: {}", err);
        }
        welcome_window_clone.close();
    });
}

fn launch_terminal_unix(inner_command: &str) -> Result<(), Error> {
    let full_command = format!("bash -c \"{}\"", inner_command);

    #[cfg(target_os = "macos")]
    {
        // Why on earth do MacOS terms require this stupid string???
        // Try Terminal.app
        let apple_script_term = format!(
            "tell application \"Terminal\" to do script {}",
            shell_escape_posix(&full_command)
        );
        if std::process::Command::new("osascript")
            .arg("-e")
            .arg(&apple_script_term)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // Try iTerm2 (fallback)
        let apple_script_iterm = format!(
            "tell application \"iTerm\" to create window with default profile; tell current session of current window to write text {}",
            shell_escape_posix(&full_command)
        );
        if std::process::Command::new("osascript")
            .arg("-e")
            .arg(&apple_script_iterm)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // Try open -a Terminal as last resort
        if std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }

    // List of terminals and their preferred execution flags
    // (Terminal Name, Execution Flag, Use Double-Dash?)
    let terminal_configs = [
        ("gnome-terminal", "--", true),       // GNOME
        ("cosmic-term", "--", true),          // Cosmic
        ("konsole", "-e", false),             // KDE
        ("xfce4-terminal", "-x", false),      // XFCE
        ("alacritty", "-e", false),           //
        ("kitty", "-e", false),               //
        ("xterm", "-e", false),               // Universal fallback
        ("x-terminal-emulator", "-e", false), // Generic system default
    ];

    for (name, flag, use_double_dash) in terminal_configs {
        match which::which(name) {
            Ok(_) => {
                let mut cmd = std::process::Command::new(name);

                if use_double_dash {
                    // This shouldnt work but it does so idk
                    cmd.arg("--").arg("bash").arg("-c").arg(&full_command);
                } else {
                    cmd.arg(flag).arg(&full_command);
                }

                let mut child = cmd.spawn()?;
                let status = child.wait()?;
                if status.success() {
                    return Ok(());
                } else {
                    return Err(anyhow::anyhow!("Child Failed!"));
                }
            }
            Err(_) => {}
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn shell_escape_posix(s: &str) -> String {
    if s.is_empty() {
        "''".into()
    } else if !s.contains('\'') {
        format!("'{}'", s)
    } else {
        let replaced = s.replace('\'', r#"'"'"'"#);
        format!("'{}'", replaced)
    }
}

#[cfg(windows)]
fn launch_terminal_windows(inner_command: &str) {
    // Try Windows Terminal (wt)
    if std::process::Command::new("wt")
        .arg("--")
        .arg("powershell")
        .arg("-NoExit")
        .arg("-Command")
        .arg(inner_command)
        .spawn()
        .is_ok()
    {
        return;
    }

    // Try PowerShell directly
    if std::process::Command::new("powershell")
        .arg("-NoExit")
        .arg("-Command")
        .arg(inner_command)
        .spawn()
        .is_ok()
    {
        return;
    }

    // Fallback to cmd.exe (start a new window with /k to keep it open)
    if std::process::Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("cmd")
        .arg("/k")
        .arg(inner_command)
        .spawn()
        .is_ok()
    {
        return;
    }
}
