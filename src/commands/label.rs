use anyhow::{Result, bail};
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn list(client: &LinearClient, team_id: Option<&str>) -> Result<()> {
    let variables = team_id.map(|tid| {
        json!({
            "filter": {
                "team": {
                    "id": { "eq": tid }
                }
            }
        })
    });

    let data: LabelsData = client.execute(LABELS_QUERY, variables).await?;

    let labels = data.issue_labels.nodes;
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
    team_id: &str,
    color: Option<&str>,
    description: Option<&str>,
    parent_id: Option<&str>,
) -> Result<()> {
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
