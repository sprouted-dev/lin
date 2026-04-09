use anyhow::{Context, Result, bail};

use super::issue::{download_file, format_byte_size};
use crate::output;

pub async fn run(token: &str, url: &str, output: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url).context("invalid URL")?;
    if parsed.host_str() != Some("uploads.linear.app") {
        bail!("URL must be an uploads.linear.app link");
    }

    let output_dir = std::path::Path::new(output);
    std::fs::create_dir_all(output_dir)?;

    let client = reqwest::Client::new();
    // download_file expects a dedup list for batch downloads; empty vec for single-file case
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_non_linear_url() {
        let result = run("fake-token", "https://example.com/file.csv", ".").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("uploads.linear.app")
        );
    }

    #[tokio::test]
    async fn rejects_spoofed_linear_url() {
        // Subdomain spoofing
        let result = run("fake-token", "https://uploads.linear.app.evil.com/file", ".").await;
        assert!(result.is_err());

        // Query param spoofing
        let result =
            run("fake-token", "https://evil.com?ref=uploads.linear.app", ".").await;
        assert!(result.is_err());

        // Path spoofing
        let result =
            run("fake-token", "https://evil.com/uploads.linear.app", ".").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_invalid_url() {
        let result = run("fake-token", "not-a-url", ".").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid URL"));
    }
}
