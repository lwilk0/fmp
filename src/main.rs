mod crypto;
mod gui;
mod models;
mod password;
mod security;
mod storage;
mod totp;
mod vault;

use crate::gui::application::run_gui;
fn main() {
    run_gui();
}
