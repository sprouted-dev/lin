use anyhow::Result;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn list(client: &LinearClient, json: bool) -> Result<()> {
    if json {
        let data = client.execute_raw(TEAMS_QUERY, None).await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: TeamsData = client.execute(TEAMS_QUERY, None).await?;

    let teams = data.teams.nodes;
    output::print_header(&format!("Teams ({})", teams.len()));

    let headers = &["Key", "Name", "Members"];
    let rows: Vec<Vec<String>> = teams
        .iter()
        .map(|t| {
            let member_count = t.members.as_ref().map(|m| m.nodes.len()).unwrap_or(0);
            vec![
                t.key.clone().unwrap_or_default(),
                t.name.clone(),
                member_count.to_string(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}
