use anyhow::{Result, bail};

use super::issue::{download_file, format_byte_size};
use crate::output;

pub async fn run(token: &str, url: &str, output: &str) -> Result<()> {
    if !url.contains("uploads.linear.app") {
        bail!("URL must be an uploads.linear.app link");
    }

    let output_dir = std::path::Path::new(output);
    std::fs::create_dir_all(output_dir)?;

    let client = reqwest::Client::new();
    let mut used: Vec<String> = Vec::new();

    match download_file(&client, url, token, output_dir, &mut used).await {
        Ok((filename, size)) => {
            output::print_success(&format!(
                "Downloaded {} ({})",
                filename,
                format_byte_size(size)
            ));
        }
        Err(e) => {
            bail!("Download failed: {e}");
        }
    }

    Ok(())
}
