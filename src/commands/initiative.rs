use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn list(client: &LinearClient, limit: i32) -> Result<()> {
    let variables = json!({ "first": limit });

    let data: InitiativesData = client.execute(INITIATIVES_QUERY, Some(variables)).await?;
    let initiatives = data.initiatives.nodes;

    output::print_header(&format!("Initiatives ({})", initiatives.len()));

    let headers = &["ID", "Name", "Status"];
    let rows: Vec<Vec<String>> = initiatives
        .iter()
        .map(|i| {
            vec![
                truncate(&i.id, 8),
                i.name.clone(),
                i.status.clone().unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

pub async fn view(client: &LinearClient, id: &str) -> Result<()> {
    let data: InitiativeDetailData = client
        .execute(INITIATIVE_QUERY, Some(json!({ "id": id })))
        .await?;

    let init = data.initiative;

    output::print_header(&init.name);

    if let Some(ref status) = init.status {
        output::print_field("Status", status);
    }

    if let Some(ref desc) = init.description
        && !desc.is_empty()
    {
        println!();
        output::print_header("Description");
        println!("  {desc}");
    }

    if let Some(ref projects) = init.projects
        && !projects.nodes.is_empty()
    {
        println!();
        output::print_header(&format!("Projects ({})", projects.nodes.len()));
        let headers = &["Name", "State"];
        let rows: Vec<Vec<String>> = projects
            .nodes
            .iter()
            .map(|p| vec![p.name.clone(), p.state.clone().unwrap_or_default()])
            .collect();
        output::print_table(headers, &rows);
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
