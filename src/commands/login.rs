use anyhow::Result;

use crate::api::client::LinearClient;
use crate::api::queries::VIEWER_QUERY;
use crate::api::types::ViewerData;
use crate::auth;
use crate::config::Config;
use crate::output;

pub async fn run(
    token: &str,
    workspace_name: &str,
    use_keyring: bool,
    verbose: bool,
) -> Result<()> {
    // Validate the token by querying the viewer
    let client = LinearClient::new(token).with_verbose(verbose);
    let viewer: ViewerData = client.execute(VIEWER_QUERY, None).await?;

    // Store the token
    auth::store_token(workspace_name, token, use_keyring)?;

    // Update default workspace in config if not set
    let mut config = Config::load()?;
    if config.default_workspace.is_none() {
        config.default_workspace = Some(workspace_name.to_string());
        config.save()?;
    }

    output::print_success(&format!(
        "Authenticated as {} ({})",
        viewer.viewer.name,
        viewer.viewer.email.as_deref().unwrap_or("no email")
    ));
    let store_label = if use_keyring { "keychain" } else { "file" };
    output::print_success(&format!(
        "Token stored for workspace '{workspace_name}' ({store_label})"
    ));

    Ok(())
}
