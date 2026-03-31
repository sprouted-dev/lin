use anyhow::{Result, bail};
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::api::upload;
use crate::output;

pub async fn view(client: &LinearClient, issue_id: &str, show_ids: bool, json: bool) -> Result<()> {
    if json {
        let data = client
            .execute_raw(COMMENTS_QUERY, Some(json!({ "id": issue_id })))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: IssueCommentsData = client
        .execute(COMMENTS_QUERY, Some(json!({ "id": issue_id })))
        .await?;

    let comments = data.issue.comments.nodes;
    output::print_header(&format!("Comments ({})", comments.len()));

    if comments.is_empty() {
        println!("  No comments.");
        return Ok(());
    }

    for comment in &comments {
        let author = comment
            .user
            .as_ref()
            .map(|u| u.name.as_str())
            .unwrap_or("Unknown");
        let date = comment
            .created_at
            .as_deref()
            .map(output::format_date)
            .unwrap_or_default();

        println!();
        if show_ids {
            println!("  [{}]", comment.id);
        }
        println!("  {} — {}", author, date);
        println!("  {}", comment.body);
    }

    Ok(())
}

pub async fn add(
    client: &LinearClient,
    issue_id: &str,
    body: &str,
    attachment_path: Option<&str>,
) -> Result<()> {
    let mut final_body = body.to_string();

    // Upload attachment and embed markdown link
    if let Some(file_path) = attachment_path {
        let asset_url = upload::upload_file(client, file_path).await?;
        let filename = std::path::Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "attachment".to_string());
        final_body = format!("{}\n\n[{}]({})", final_body, filename, asset_url);
    }

    let input = json!({
        "issueId": issue_id,
        "body": final_body,
    });

    let data: CommentCreateData = client
        .execute(COMMENT_CREATE_MUTATION, Some(json!({ "input": input })))
        .await?;

    if !data.comment_create.success {
        bail!("Failed to create comment");
    }

    output::print_success("Comment added");
    Ok(())
}

pub async fn edit(
    client: &LinearClient,
    comment_id: &str,
    body: &str,
    attachment_path: Option<&str>,
) -> Result<()> {
    let mut final_body = body.to_string();

    // Upload attachment and embed markdown link
    if let Some(file_path) = attachment_path {
        let asset_url = upload::upload_file(client, file_path).await?;
        let filename = std::path::Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "attachment".to_string());
        final_body = format!("{}\n\n[{}]({})", final_body, filename, asset_url);
    }

    let input = json!({ "body": final_body });

    let data: CommentUpdateData = client
        .execute(
            COMMENT_UPDATE_MUTATION,
            Some(json!({ "id": comment_id, "input": input })),
        )
        .await?;

    if !data.comment_update.success {
        bail!("Failed to update comment");
    }

    output::print_success("Comment updated");
    Ok(())
}
