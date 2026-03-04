use std::env;
use std::path::Path;

use crate::config::Config;

/// Resolve workspace name using 4-tier precedence:
/// 1. CLI --workspace flag
/// 2. .linear-workspace file (walk up directories)
/// 3. Directory-specific config
/// 4. Global default from config
///
/// Falls back to "default"
pub fn resolve_workspace(cli_workspace: Option<&str>) -> String {
    // 1. CLI flag
    if let Some(ws) = cli_workspace {
        return ws.to_string();
    }

    // 2. .linear-workspace file (walk up)
    if let Some(ws) = find_workspace_file() {
        return ws;
    }

    // 3 & 4. Config-based resolution
    if let Ok(config) = Config::load() {
        // 3. Directory workspace
        if let Ok(cwd) = env::current_dir() {
            let cwd_str = cwd.to_string_lossy().to_string();
            if let Some(ws) = config.directory_workspaces.get(&cwd_str) {
                return ws.clone();
            }
        }

        // 4. Global default
        if let Some(ws) = config.default_workspace {
            return ws;
        }
    }

    "default".to_string()
}

fn find_workspace_file() -> Option<String> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let file = dir.join(".linear-workspace");
        if file.exists() {
            let contents = std::fs::read_to_string(&file).ok()?;
            let trimmed = contents.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

pub fn write_workspace_file(name: &str) -> Result<(), std::io::Error> {
    let path = Path::new(".linear-workspace");
    std::fs::write(path, name)
}
