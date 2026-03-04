use anyhow::{Result, bail};
use serde_json::json;

use super::client::LinearClient;
use super::queries::*;
use super::types::*;

/// Check if a string looks like a UUID (contains dashes and is long enough)
fn is_uuid(s: &str) -> bool {
    s.len() > 20 && s.contains('-')
}

/// Resolve an issue identifier (e.g., APP-123) to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_issue_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    let variables = json!({
        "query": identifier,
        "first": 1,
        "filter": {},
    });

    let data: IssueSearchData = client.execute(ISSUE_SEARCH_QUERY, Some(variables)).await?;

    let issue = data
        .issue_search
        .nodes
        .into_iter()
        .find(|i| i.identifier.eq_ignore_ascii_case(identifier))
        .or(None);

    match issue {
        Some(i) => Ok(i.id),
        None => bail!("Could not resolve issue identifier: {}", identifier),
    }
}

/// Resolve label names to label IDs via case-insensitive matching.
pub async fn resolve_label_names(client: &LinearClient, names: &[String]) -> Result<Vec<String>> {
    let data: LabelsData = client.execute(LABELS_QUERY, None).await?;

    let all_labels = data.issue_labels.nodes;
    let mut ids = Vec::new();

    for name in names {
        let lower = name.to_lowercase();
        let found = all_labels.iter().find(|l| l.name.to_lowercase() == lower);
        match found {
            Some(label) => ids.push(label.id.clone()),
            None => bail!(
                "Label '{}' not found. Available labels: {}",
                name,
                all_labels
                    .iter()
                    .map(|l| l.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    Ok(ids)
}
