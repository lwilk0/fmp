/// Sort vaults by name, case insensitive.
pub fn sort_vaults<'a>(names: &'a [String], filter: &str) -> Vec<&'a str> {
    let mut view: Vec<&str> = if filter.is_empty() {
        names.iter().map(std::string::String::as_str).collect()
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

    view.sort_by_cached_key(|s| s.to_ascii_lowercase());

    view
}
