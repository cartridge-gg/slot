use std::env;

pub fn get_cartridge_keychain_url() -> String {
    get_env("CARTRIDGE_KEYCHAIN_URL", "https://x.cartridge.gg")
}

pub fn get_cartridge_api_url() -> String {
    get_env("CARTRIDGE_API_URL", "https://api.cartridge.gg")
}

pub fn get_env(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_e) => default.to_string(),
    }
}
