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
pub async fn resolve_project_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
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

/// Resolve a workflow state name to a UUID for a given issue.
/// Fetches the issue's team, then matches the state name case-insensitively.
pub async fn resolve_state_name(
    client: &LinearClient,
    issue_id: &str,
    state_name: &str,
) -> Result<String> {
    if is_uuid(state_name) {
        return Ok(state_name.to_string());
    }

    let issue_data: IssueData = client
        .execute(ISSUE_QUERY, Some(json!({ "id": issue_id })))
        .await?;
    let team = issue_data
        .issue
        .team
        .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;

    let team_data: TeamData = client
        .execute(TEAM_STATES_QUERY, Some(json!({ "id": team.id })))
        .await?;

    let target_lower = state_name.to_lowercase();
    let matching = team_data
        .team
        .states
        .nodes
        .iter()
        .find(|s| s.name.to_lowercase() == target_lower);

    match matching {
        Some(state) => Ok(state.id.clone()),
        None => {
            let available: Vec<&str> = team_data
                .team
                .states
                .nodes
                .iter()
                .map(|s| s.name.as_str())
                .collect();
            bail!(
                "State '{}' not found. Available states: {}",
                state_name,
                available.join(", ")
            )
        }
    }
}

/// Resolve a cycle identifier (name, number, "current", or UUID) to a UUID.
/// Requires a pre-resolved team_id since cycles are team-scoped.
/// If identifier is "current", returns the active cycle for the team.
pub async fn resolve_cycle_identifier(
    client: &LinearClient,
    team_id: &str,
    identifier: &str,
) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    // Handle "current" to get the active cycle
    if identifier.eq_ignore_ascii_case("current") {
        let variables = json!({
            "first": 1,
            "filter": {
                "team": { "id": { "eq": team_id } },
                "isActive": { "eq": true }
            },
        });

        let data: CyclesData = client.execute(CYCLES_QUERY, Some(variables)).await?;
        match data.cycles.nodes.first() {
            Some(cycle) => return Ok(cycle.id.clone()),
            None => bail!("No active cycle found for this team."),
        }
    }

    // Fetch all cycles for the team
    let variables = json!({
        "first": 100,
        "filter": { "team": { "id": { "eq": team_id } } },
    });

    let data: CyclesData = client.execute(CYCLES_QUERY, Some(variables)).await?;
    let lower = identifier.to_lowercase();

    // Try matching by number first (if identifier is numeric)
    if let Ok(num) = identifier.parse::<i32>()
        && let Some(cycle) = data.cycles.nodes.iter().find(|c| c.number == Some(num))
    {
        return Ok(cycle.id.clone());
    }

    // Try matching by name
    let found = data
        .cycles
        .nodes
        .iter()
        .find(|c| c.name.as_ref().map(|n| n.to_lowercase()) == Some(lower.clone()));

    match found {
        Some(cycle) => Ok(cycle.id.clone()),
        None => {
            let available: Vec<String> = data
                .cycles
                .nodes
                .iter()
                .map(|c| {
                    let num = c.number.map(|n| n.to_string()).unwrap_or_default();
                    let name = c.name.clone().unwrap_or_default();
                    if name.is_empty() {
                        format!("#{}", num)
                    } else {
                        format!("{} (#{num})", name)
                    }
                })
                .collect();
            bail!(
                "Cycle '{}' not found. Available cycles: {}",
                identifier,
                available.join(", ")
            )
        }
    }
}

/// Resolve label names to label IDs via case-insensitive matching.
/// Paginates through all workspace labels to avoid missing any.
pub async fn resolve_label_names(client: &LinearClient, names: &[String]) -> Result<Vec<String>> {
    let all_labels = fetch_all_labels(client, None).await?;
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

/// Fetch all labels, paginating through all pages.
pub async fn fetch_all_labels(
    client: &LinearClient,
    filter: Option<serde_json::Value>,
) -> Result<Vec<Label>> {
    let mut all_labels: Vec<Label> = Vec::new();
    let mut after: Option<String> = None;

    loop {
        let mut vars = json!({ "first": 250 });
        if let Some(ref f) = filter {
            vars["filter"] = f.clone();
        }
        if let Some(ref cursor) = after {
            vars["after"] = json!(cursor);
        }

        let data: LabelsData = client.execute(LABELS_QUERY, Some(vars)).await?;

        all_labels.extend(data.issue_labels.nodes);

        if data.issue_labels.page_info.has_next_page {
            after = data.issue_labels.page_info.end_cursor;
        } else {
            break;
        }
    }

    Ok(all_labels)
}
