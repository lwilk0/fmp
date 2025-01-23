mod vault;
mod json;
mod account;

fn main() {
    //vault::encrypt_fmp_vault();
    vault::decrypt_fmp_vault();
    vault::read_vault();
    vault::delete_vault(vault::get_fmp_vault_location());
    }