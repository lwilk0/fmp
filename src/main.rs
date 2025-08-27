mod crypto;
mod flags;
mod gui;
mod password;
mod totp;
mod vault;

use crate::gui::application::run_gui;
fn main() {
    run_gui();
}
