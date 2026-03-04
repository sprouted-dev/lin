use anyhow::{Result, bail};
use serde_json::json;
use std::path::Path;

use super::client::LinearClient;
use super::queries::*;
use super::types::*;

fn mime_from_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "csv" => "text/csv",
        "json" => "application/json",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

pub async fn upload_file(client: &LinearClient, file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    if !path.exists() {
        bail!("File not found: {}", file_path);
    }

    let file_bytes = std::fs::read(path)?;
    let file_size = file_bytes.len() as i64;
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let content_type = mime_from_extension(&ext);

    // Step 1: Get presigned upload URL
    let variables = json!({
        "contentType": content_type,
        "filename": filename,
        "size": file_size,
    });

    let data: FileUploadData = client
        .execute(FILE_UPLOAD_MUTATION, Some(variables))
        .await?;

    let upload_url = &data.file_upload.upload_file.upload_url;
    let asset_url = data.file_upload.upload_file.asset_url.clone();

    // Step 2: PUT file bytes to the presigned URL
    let http_client = reqwest::Client::new();
    let response = http_client
        .put(upload_url)
        .header("Content-Type", content_type)
        .header("Cache-Control", "public, max-age=31536000")
        .body(file_bytes)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        bail!("File upload failed: HTTP {} - {}", status, text);
    }

    Ok(asset_url)
}

pub async fn create_attachment(
    client: &LinearClient,
    issue_id: &str,
    url: &str,
    title: &str,
) -> Result<()> {
    let input = json!({
        "issueId": issue_id,
        "url": url,
        "title": title,
    });

    let data: AttachmentCreateData = client
        .execute(ATTACHMENT_CREATE_MUTATION, Some(json!({ "input": input })))
        .await?;

    if !data.attachment_create.success {
        bail!("Failed to create attachment");
    }

    Ok(())
}
