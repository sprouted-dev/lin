use std::io::Read;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::api::client::LinearClient;
use crate::output;

/// Read an argument that may be a literal string, `@path` to read from a file,
/// or `-` to read from stdin.
fn read_source(arg: &str, what: &str) -> Result<String> {
    if arg == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .with_context(|| format!("Failed to read {what} from stdin"))?;
        Ok(buf)
    } else if let Some(path) = arg.strip_prefix('@') {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {what} from file: {path}"))
    } else {
        Ok(arg.to_string())
    }
}

pub async fn run(client: &LinearClient, query: &str, variables: Option<&str>) -> Result<()> {
    let document = read_source(query, "query")?;

    let vars: Option<Value> = match variables {
        Some(v) => {
            let raw = read_source(v, "variables")?;
            let parsed: Value =
                serde_json::from_str(&raw).context("Variables must be a valid JSON object")?;
            Some(parsed)
        }
        None => None,
    };

    let data = client.execute_raw(&document, vars).await?;
    output::print_json(&data);
    Ok(())
}
