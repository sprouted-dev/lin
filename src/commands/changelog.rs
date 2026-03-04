use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn run(client: &LinearClient) -> Result<()> {
    let filter = json!({
        "state": { "type": { "eq": "completed" } }
    });

    let variables = json!({
        "first": 20,
        "filter": filter,
    });

    let data: IssuesData = client.execute(ISSUES_QUERY, Some(variables)).await?;

    let issues = data.issues.nodes;
    output::print_header(&format!(
        "Changelog — Recently Completed ({})",
        issues.len()
    ));

    let headers = &["ID", "Title", "Team", "Assignee"];
    let rows: Vec<Vec<String>> = issues
        .iter()
        .map(|i| {
            vec![
                i.identifier.clone(),
                truncate(&i.title, 50),
                i.team.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
                i.assignee
                    .as_ref()
                    .map(|a| a.name.clone())
                    .unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
