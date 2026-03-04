use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum LinError {
    #[error("Not authenticated. Run `lin login <token>` first.")]
    NotAuthenticated,

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("GraphQL errors: {}", .0.join("; "))]
    GraphQLErrors(Vec<String>),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
