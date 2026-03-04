use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LinError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub default_workspace: Option<String>,

    #[serde(default)]
    pub directory_workspaces: HashMap<String, String>,
}

impl Config {
    pub fn path() -> Result<PathBuf, LinError> {
        let home = dirs::home_dir().ok_or_else(|| {
            LinError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(".linear-cli").join("config.json"))
    }

    pub fn load() -> Result<Self, LinError> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)?;
        serde_json::from_str(&contents)
            .map_err(|e| LinError::ConfigError(format!("Failed to parse config: {e}")))
    }

    pub fn save(&self) -> Result<(), LinError> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}
