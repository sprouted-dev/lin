use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn run(client: &LinearClient, json: bool) -> Result<()> {
    let filter = json!({
        "state": { "type": { "eq": "completed" } }
    });

    let variables = json!({
        "first": 20,
        "filter": filter,
    });

    if json {
        let data = client.execute_raw(ISSUES_QUERY, Some(variables)).await?;
        output::print_json(&data);
        return Ok(());
    }

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
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max - 1).collect();
        format!("{prefix}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_splits_on_char_boundary() {
        let s = "Submit Block Trading request is unauthenticated — 401s before reaching Schwab";
        let out = truncate(s, 50);
        assert_eq!(out.chars().count(), 50);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 50), "hello");
    }
}
