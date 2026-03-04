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
        "term": identifier,
        "first": 1,
    });

    let data: IssueSearchData = client.execute(ISSUE_SEARCH_QUERY, Some(variables)).await?;

    let issue = data
        .search_issues
        .nodes
        .into_iter()
        .find(|i| i.identifier.eq_ignore_ascii_case(identifier));

    match issue {
        Some(i) => Ok(i.id),
        None => bail!("Could not resolve issue identifier: {}", identifier),
    }
}

/// Resolve a user identifier (name, email, or "me") to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_user_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    if identifier == "me" {
        let data: ViewerData = client.execute(VIEWER_QUERY, None).await?;
        return Ok(data.viewer.id);
    }

    let data: UsersData = client.execute(USERS_QUERY, None).await?;
    let lower = identifier.to_lowercase();

    let found = data.users.nodes.iter().find(|u| {
        u.name.to_lowercase() == lower
            || u.email.as_deref().map(|e| e.to_lowercase()) == Some(lower.clone())
            || u.display_name.as_deref().map(|d| d.to_lowercase()) == Some(lower.clone())
    });

    match found {
        Some(user) => Ok(user.id.clone()),
        None => bail!(
            "User '{}' not found. Use `lin user list` to see available users.",
            identifier
        ),
    }
}

/// Resolve a team identifier (name, key, or UUID) to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_team_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    let data: TeamsData = client.execute(TEAMS_QUERY, None).await?;
    let lower = identifier.to_lowercase();

    let found = data.teams.nodes.iter().find(|t| {
        t.name.to_lowercase() == lower
            || t.key.as_deref().map(|k| k.to_lowercase()) == Some(lower.clone())
    });

    match found {
        Some(team) => Ok(team.id.clone()),
        None => bail!(
            "Team '{}' not found. Use `lin team list` to see available teams.",
            identifier
        ),
    }
}

/// Resolve a project identifier (name, slug, or UUID) to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_project_identifier(
    client: &LinearClient,
    identifier: &str,
) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    let variables = json!({ "first": 250, "includeArchived": false });
    let data: ProjectsData = client.execute(PROJECTS_QUERY, Some(variables)).await?;
    let lower = identifier.to_lowercase();

    let found = data
        .projects
        .nodes
        .iter()
        .find(|p| p.name.to_lowercase() == lower);

    match found {
        Some(project) => Ok(project.id.clone()),
        None => bail!(
            "Project '{}' not found. Use `lin project list` to see available projects.",
            identifier
        ),
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
