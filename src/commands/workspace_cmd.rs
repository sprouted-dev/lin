use anyhow::Result;

use crate::config::Config;
use crate::output;
use crate::workspace;

pub fn current(cli_workspace: Option<&str>) {
    let ws = workspace::resolve_workspace(cli_workspace);
    output::print_header("Current workspace");
    output::print_field("Name", &ws);
}

pub fn list() -> Result<()> {
    let config = Config::load()?;

    output::print_header("Workspaces");

    if let Some(ref default) = config.default_workspace {
        output::print_field("Default", default);
    }

    if config.directory_workspaces.is_empty() {
        println!("  No directory workspaces configured.");
    } else {
        println!();
        println!("  Directory workspaces:");
        for (dir, ws) in &config.directory_workspaces {
            output::print_field(dir, ws);
        }
    }

    Ok(())
}

pub fn set(name: &str, global: bool) -> Result<()> {
    if global {
        let mut config = Config::load()?;
        config.default_workspace = Some(name.to_string());
        config.save()?;
        output::print_success(&format!("Global default workspace set to '{name}'"));
    } else {
        workspace::write_workspace_file(name)?;

        // Also update the config's directory workspace mapping
        let cwd = std::env::current_dir()?;
        let mut config = Config::load()?;
        config
            .directory_workspaces
            .insert(cwd.to_string_lossy().to_string(), name.to_string());
        config.save()?;

        output::print_success(&format!("Workspace set to '{name}'"));
        output::print_field("File", ".linear-workspace");
    }
    Ok(())
}
