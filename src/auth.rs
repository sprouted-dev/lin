use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::error::LinError;

const SERVICE_NAME: &str = "linear-cli";

fn account_name(workspace: &str) -> String {
    format!("workspace-{workspace}")
}

fn tokens_path() -> Result<PathBuf, LinError> {
    let home = dirs::home_dir()
        .ok_or_else(|| LinError::ConfigError("Could not determine home directory".to_string()))?;
    Ok(home.join(".linear-cli").join("tokens.json"))
}

fn load_tokens() -> Result<HashMap<String, String>, LinError> {
    let path = tokens_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let contents = std::fs::read_to_string(&path)?;
    serde_json::from_str(&contents)
        .map_err(|e| LinError::ConfigError(format!("Failed to parse tokens file: {e}")))
}

fn save_tokens(tokens: &HashMap<String, String>) -> Result<(), LinError> {
    let path = tokens_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(tokens)?;
    std::fs::write(&path, &contents)?;
    // Set file permissions to owner-only (600)
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

fn store_token_file(workspace: &str, token: &str) -> Result<(), LinError> {
    let mut tokens = load_tokens()?;
    tokens.insert(workspace.to_string(), token.to_string());
    save_tokens(&tokens)
}

fn get_token_file(workspace: &str) -> Option<String> {
    load_tokens().ok()?.remove(workspace)
}

fn store_token_keyring(workspace: &str, token: &str) -> Result<(), LinError> {
    let entry = keyring::Entry::new(SERVICE_NAME, &account_name(workspace))
        .map_err(|e| LinError::KeyringError(e.to_string()))?;
    entry
        .set_password(token)
        .map_err(|e| LinError::KeyringError(e.to_string()))?;
    Ok(())
}

fn get_token_keyring(workspace: &str) -> Option<String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &account_name(workspace)).ok()?;
    entry.get_password().ok()
}

pub fn store_token(workspace: &str, token: &str, use_keyring: bool) -> Result<(), LinError> {
    if use_keyring {
        store_token_keyring(workspace, token)
    } else {
        store_token_file(workspace, token)
    }
}

pub fn get_token(workspace: &str) -> Result<String, LinError> {
    // Check file-based tokens first, then fall back to keychain
    if let Some(token) = get_token_file(workspace) {
        return Ok(token);
    }
    if let Some(token) = get_token_keyring(workspace) {
        return Ok(token);
    }
    Err(LinError::NotAuthenticated)
}
