use crate::error::LinError;

const SERVICE_NAME: &str = "linear-cli";

fn account_name(workspace: &str) -> String {
    format!("workspace-{workspace}")
}

pub fn store_token(workspace: &str, token: &str) -> Result<(), LinError> {
    let entry = keyring::Entry::new(SERVICE_NAME, &account_name(workspace))
        .map_err(|e| LinError::KeyringError(e.to_string()))?;
    entry
        .set_password(token)
        .map_err(|e| LinError::KeyringError(e.to_string()))?;
    Ok(())
}

pub fn get_token(workspace: &str) -> Result<String, LinError> {
    let entry = keyring::Entry::new(SERVICE_NAME, &account_name(workspace))
        .map_err(|e| LinError::KeyringError(e.to_string()))?;
    entry.get_password().map_err(|_| LinError::NotAuthenticated)
}
