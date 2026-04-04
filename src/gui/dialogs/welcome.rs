/// All this code is awful, sorry future me. There must be a better way????
use adw::{HeaderBar, Window as AdwWindow, prelude::*};
use gtk4::{Box as GtkBox, Button, Entry, Label, Orientation, glib};
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
    welcome_window.set_default_size(-1, -1);
    welcome_window.add_css_class("welcome-dialog");
    welcome_window.set_deletable(false);

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Welcome"))));
    header_bar.add_css_class("flat");

    let main_container = GtkBox::new(Orientation::Vertical, 0);
    main_container.append(&header_bar);

    let content_box = GtkBox::new(Orientation::Vertical, 20);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    let title_container = GtkBox::new(Orientation::Horizontal, 12);
    title_container.set_halign(gtk4::Align::Center);
    let dialog_title = Label::new(Some("FMP"));
    dialog_title.add_css_class("title-1");
    title_container.append(&dialog_title);
    content_box.append(&title_container);

    let intro_label = Label::new(Some(
        "FMP is a secure local password manager.\nCheck for an existing GPG key by email or create one.",
    ));
    intro_label.set_wrap(true);
    intro_label.add_css_class("body");
    content_box.append(&intro_label);

    let email_box = GtkBox::new(Orientation::Vertical, 8);
    email_box.set_halign(gtk4::Align::Center);
    email_box.add_css_class("group_background");

    let email_title_container = GtkBox::new(Orientation::Horizontal, 12);
    email_title_container.set_halign(gtk4::Align::Center);
    let email_title = Label::new(Some("Use an Existing Key:"));
    email_title.add_css_class("title-2");
    email_title_container.append(&email_title);
    email_box.append(&email_title_container);

    let email_entry = Entry::new();
    email_entry.set_placeholder_text(Some("Enter email associated with GPG key"));
    email_entry.set_width_chars(45);
    email_box.append(&email_entry);

    let check_button = Button::with_label("Check Key");
    email_box.append(&check_button);

    content_box.append(&email_box);

    let create_box = GtkBox::new(Orientation::Vertical, 8);
    create_box.set_halign(gtk4::Align::Center);
    create_box.add_css_class("group_background");

    let create_title_container = GtkBox::new(Orientation::Horizontal, 12);
    create_title_container.set_halign(gtk4::Align::Center);
    let create_title = Label::new(Some("Create a New Key:"));
    create_title.add_css_class("title-2");
    create_title_container.append(&create_title);
    create_box.append(&create_title_container);

    let terminal_button = Button::with_label("Run `gpg --full-generate-key`");
    create_box.append(&terminal_button);

    let create_label = Label::new(Some(
        "It is recommended to use default settings.\nYou do not need to use a valid email, just remember it!",
    ));
    create_label.set_wrap(true);
    create_label.add_css_class("body");
    create_box.append(&create_label);

    content_box.append(&create_box);

    let check_result_label = Label::new(None);
    check_result_label.set_visible(false);
    check_result_label.set_wrap(true);
    email_box.append(&check_result_label);

    let sep = gtk4::Separator::new(Orientation::Horizontal);
    content_box.append(&sep);

    let action_box = GtkBox::new(Orientation::Horizontal, 12);
    action_box.set_halign(gtk4::Align::Center);

    let finish_button = Button::new();
    finish_button.set_label("Finish");
    finish_button.add_css_class("suggested-action");
    action_box.append(&finish_button);

    content_box.append(&action_box);

    main_container.append(&content_box);
    welcome_window.set_content(Some(&main_container));
    welcome_window.present();

    let check_result_label_clone = check_result_label.clone();
    let email_entry_clone = email_entry.clone();
    check_button.connect_clicked(move |_| {
        check_result_label_clone.set_visible(true);
        let email = email_entry_clone.text().to_string();
        if email.trim().is_empty() {
            check_result_label_clone.set_text("Please enter an email.");
            return;
        }
        check_result_label_clone.set_text("Checking for GPG key...");

        let weak_label = check_result_label_clone.downgrade();

        let output = std::process::Command::new("gpg")
            .arg("--list-keys")
            .arg("--with-colons")
            .arg(&email)
            .stderr(Stdio::piped())
            .output();

        let text = match output {
            Ok(out) => {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if stdout.contains("\nuid:")
                        || stdout.contains("\npub:")
                        || stdout.contains(":uid:")
                    {
                        format!("GPG key found for {}.", email)
                    } else {
                        format!("No GPG key found for {}.", email)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    format!("gpg returned an error: {}", stderr)
                }
            }
            Err(e) => format!("Failed to run gpg: {}", e),
        };

        // Move only the plain String into the main-loop task. Do NOT move the WeakRef into the std thread.
        let text_for_main = text.clone();
        let weak_label_for_main = weak_label.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Some(label) = weak_label_for_main.upgrade() {
                label.set_text(&text_for_main);
            }
        });
    });

    terminal_button.connect_clicked(move |_| {
        #[cfg(unix)]
        launch_terminal_unix("gpg --full-generate-key");

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

fn launch_terminal_unix(inner_command: &str) {
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
            return;
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
            return;
        }

        // Try open -a Terminal as last resort
        if std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .spawn()
            .is_ok()
        {
            return;
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
        let mut cmd = std::process::Command::new(name);

        if use_double_dash {
            // This shouldnt work but it does so idk
            cmd.arg("--").arg("bash").arg("-c").arg(&full_command);
        } else {
            cmd.arg(flag).arg(&full_command);
        }

        if cmd.spawn().is_ok() {
            return; // Successful woohoo
        }
    }
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
