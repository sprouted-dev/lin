use anyhow::{Result, bail};
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::resolve;
use crate::api::types::*;
use crate::output;

pub async fn list(client: &LinearClient, team: Option<&str>, json_output: bool) -> Result<()> {
    let filter = match team {
        Some(t) => {
            let tid = resolve::resolve_team_identifier(client, t).await?;
            Some(json!({
                "team": {
                    "id": { "eq": tid }
                }
            }))
        }
        None => None,
    };

    let labels = resolve::fetch_all_labels(client, filter).await?;

    if json_output {
        output::print_json(&serde_json::to_value(&labels)?);
        return Ok(());
    }

    output::print_header(&format!("Labels ({})", labels.len()));

    let headers = &["Name", "Color"];
    let rows: Vec<Vec<String>> = labels
        .iter()
        .map(|l| vec![l.name.clone(), l.color.clone().unwrap_or_default()])
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

pub async fn create(
    client: &LinearClient,
    name: &str,
    team: &str,
    color: Option<&str>,
    description: Option<&str>,
    parent_id: Option<&str>,
) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;
    let mut input = json!({
        "name": name,
        "teamId": team_id,
    });

    if let Some(c) = color {
        input["color"] = json!(c);
    }
    if let Some(d) = description {
        input["description"] = json!(d);
    }
    if let Some(p) = parent_id {
        input["parentId"] = json!(p);
    }

    let data: LabelCreateData = client
        .execute(LABEL_CREATE_MUTATION, Some(json!({ "input": input })))
        .await?;

    if !data.issue_label_create.success {
        bail!("Failed to create label");
    }

    if let Some(label) = data.issue_label_create.issue_label {
        output::print_success(&format!("Created label: {}", label.name));
    }

    Ok(())
}

pub async fn edit(
    client: &LinearClient,
    label: &str,
    team: Option<&str>,
    name: Option<&str>,
    color: Option<&str>,
    description: Option<&str>,
    parent_id: Option<&str>,
) -> Result<()> {
    if name.is_none() && color.is_none() && description.is_none() && parent_id.is_none() {
        bail!(
            "Nothing to update. Provide at least one of --name, --color, --description, --parent-id"
        );
    }

    let label_id = resolve::resolve_label_identifier(client, label, team).await?;
    let mut input = json!({});

    if let Some(n) = name {
        input["name"] = json!(n);
    }
    if let Some(c) = color {
        input["color"] = json!(c);
    }
    if let Some(d) = description {
        input["description"] = json!(d);
    }
    if let Some(p) = parent_id {
        input["parentId"] = json!(p);
    }

    let data: LabelUpdateData = client
        .execute(
            LABEL_UPDATE_MUTATION,
            Some(json!({ "id": label_id, "input": input })),
        )
        .await?;

    if !data.issue_label_update.success {
        bail!("Failed to update label");
    }

    if let Some(label) = data.issue_label_update.issue_label {
        output::print_success(&format!("Updated label: {}", label.name));
    }

    Ok(())
}

pub async fn delete(client: &LinearClient, label: &str, team: Option<&str>) -> Result<()> {
    let label_id = resolve::resolve_label_identifier(client, label, team).await?;

    let data: LabelDeleteData = client
        .execute(LABEL_DELETE_MUTATION, Some(json!({ "id": label_id })))
        .await?;

    if !data.issue_label_delete.success {
        bail!("Failed to delete label");
    }

    output::print_success(&format!("Deleted label: {}", label));
    Ok(())
}
