mod vault;
mod json;

fn main() {
    let test = json::read_json(vault::get_fmp_vault_location(), "Proton");
    println!("{} {}", test.username, test.password)
}