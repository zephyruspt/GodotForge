use keyring_core::{Entry, Error};

const SERVICE: &str = "pt.zephyrus.godot-forge";
const GITHUB_TOKEN_ACCOUNT: &str = "github-token";

pub(crate) fn initialize_secret_store() -> Result<(), String> {
    keyring::use_native_store(true).map_err(|error| error.to_string())
}

fn github_token_entry() -> Result<Entry, String> {
    Entry::new(SERVICE, GITHUB_TOKEN_ACCOUNT).map_err(|error| error.to_string())
}

pub(crate) fn github_token() -> Option<String> {
    github_token_entry()
        .ok()
        .and_then(|entry| entry.get_password().ok())
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

pub(crate) fn github_token_configured() -> bool {
    github_token().is_some()
}

pub(crate) fn save_github_token(token: &str) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        return clear_github_token();
    }

    github_token_entry()?
        .set_password(token)
        .map_err(|error| error.to_string())
}

pub(crate) fn clear_github_token() -> Result<(), String> {
    match github_token_entry()?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
