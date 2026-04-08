mod crypto;
mod gui;
mod models;
mod password;
mod security;
mod storage;
mod totp;
mod vault;

#[cfg(test)]
mod tests;

use crate::gui::application::run_gui;

fn main() {
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }

    run_gui();
}
