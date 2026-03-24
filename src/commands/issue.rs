use anyhow::{Result, bail};
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::resolve;
use crate::api::types::*;
use crate::api::upload;
use crate::date;
use crate::output;

/// Date filter options for issue listing
#[derive(Default)]
pub struct DateFilters {
    pub updated_since: Option<String>,
    pub updated_before: Option<String>,
    pub created_since: Option<String>,
    pub created_before: Option<String>,
    pub completed_since: Option<String>,
    pub completed_before: Option<String>,
    pub due_after: Option<String>,
    pub due_before: Option<String>,
    pub cancelled_since: Option<String>,
}

/// Convenience filter options for issue listing
#[derive(Default)]
pub struct ConvenienceFilters {
    pub estimate: Option<f64>,
    pub estimate_gte: Option<f64>,
    pub estimate_lte: Option<f64>,
    pub parent: Option<String>,
    pub no_parent: bool,
    pub has_children: bool,
    pub subscriber: Option<String>,
    pub title: Option<String>,
}

pub async fn view(client: &LinearClient, id: &str) -> Result<()> {
    let data: IssueData = client
        .execute(ISSUE_QUERY, Some(json!({ "id": id })))
        .await?;

    let issue = data.issue;

    output::print_header(&format!("{} — {}", issue.identifier, issue.title));

    if let Some(ref state) = issue.state {
        output::print_field("Status", &state.name);
    }
    if let Some(ref assignee) = issue.assignee {
        output::print_field("Assignee", &assignee.name);
    }
    if let Some(ref team) = issue.team {
        output::print_field("Team", &team.name);
    }
    if let Some(ref project) = issue.project {
        output::print_field("Project", &project.name);
    }
    if let Some(priority) = issue.priority {
        let label = match priority as i32 {
            0 => "None",
            1 => "Urgent",
            2 => "High",
            3 => "Medium",
            4 => "Low",
            _ => "Unknown",
        };
        output::print_field("Priority", label);
    }

    if let Some(ref labels) = issue.labels
        && !labels.nodes.is_empty()
    {
        let names: Vec<&str> = labels.nodes.iter().map(|l| l.name.as_str()).collect();
        output::print_field("Labels", &names.join(", "));
    }

    if let Some(ref parent) = issue.parent {
        output::print_field(
            "Parent",
            &format!("{} — {}", parent.identifier, parent.title),
        );
    }

    if let Some(ref children) = issue.children
        && !children.nodes.is_empty()
    {
        println!();
        output::print_header("Sub-issues");
        for child in &children.nodes {
            println!("  {} — {}", child.identifier, child.title);
        }
    }

    if let Some(ref desc) = issue.description
        && !desc.is_empty()
    {
        println!();
        output::print_header("Description");
        println!("  {desc}");
    }

    println!();
    if let Some(ref created) = issue.created_at {
        output::print_field("Created", &output::format_date(created));
    }
    if let Some(ref updated) = issue.updated_at {
        output::print_field("Updated", &output::format_date(updated));
    }
    if let Some(ref due) = issue.due_date {
        output::print_field("Due", due);
    }
    if let Some(ref url) = issue.url {
        output::print_field("URL", url);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    client: &LinearClient,
    title: &str,
    team: &str,
    description: Option<&str>,
    priority: Option<i32>,
    assignee: Option<&str>,
    project: Option<&str>,
    label_ids: Option<&[String]>,
    labels: Option<&[String]>,
    parent: Option<&str>,
    attachment_path: Option<&str>,
) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;

    let mut input = IssueCreateInput {
        title: title.to_string(),
        team_id,
        ..Default::default()
    };
    input.description = description.map(|s| s.to_string());
    input.priority = priority;
    input.assignee_id = match assignee {
        Some(aid) => Some(resolve::resolve_user_identifier(client, aid).await?),
        None => None,
    };
    input.project_id = match project {
        Some(p) => Some(resolve::resolve_project_identifier(client, p).await?),
        None => None,
    };

    // Resolve label names to IDs and merge with explicit label_ids
    let mut all_label_ids: Vec<String> = label_ids.map(|ids| ids.to_vec()).unwrap_or_default();
    if let Some(names) = labels {
        let resolved = resolve::resolve_label_names(client, names).await?;
        all_label_ids.extend(resolved);
    }
    if !all_label_ids.is_empty() {
        input.label_ids = Some(all_label_ids);
    }

    // Resolve parent if it's an identifier
    if let Some(pid) = parent {
        let resolved = resolve::resolve_issue_identifier(client, pid).await?;
        input.parent_id = Some(resolved);
    }

    let data: IssueCreateData = client
        .execute(ISSUE_CREATE_MUTATION, Some(json!({ "input": input })))
        .await?;

    if !data.issue_create.success {
        bail!("Failed to create issue");
    }

    if let Some(issue) = data.issue_create.issue {
        output::print_success(&format!("Created {} — {}", issue.identifier, issue.title));
        if let Some(ref url) = issue.url {
            output::print_field("URL", url);
        }

        // Handle attachment upload
        if let Some(file_path) = attachment_path {
            let asset_url = upload::upload_file(client, file_path).await?;
            let filename = std::path::Path::new(file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "attachment".to_string());
            upload::create_attachment(client, &issue.id, &asset_url, &filename).await?;
            output::print_success(&format!("Attached: {}", filename));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn edit(
    client: &LinearClient,
    id: &str,
    title: Option<String>,
    description: Option<String>,
    priority: Option<i32>,
    assignee: Option<String>,
    state: Option<String>,
    project: Option<String>,
    label_ids: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    remove_labels: Option<Vec<String>>,
    parent: Option<String>,
    attachment_path: Option<String>,
) -> Result<()> {
    // Resolve label names
    let mut final_label_ids = label_ids;

    if labels.is_some() || remove_labels.is_some() {
        // Need to fetch current labels to merge
        let issue_data: IssueData = client
            .execute(ISSUE_QUERY, Some(json!({ "id": id })))
            .await?;
        let mut current_ids: Vec<String> = issue_data
            .issue
            .labels
            .map(|l| l.nodes.into_iter().map(|n| n.id).collect())
            .unwrap_or_default();

        // Add new labels by name
        if let Some(ref names) = labels {
            let resolved = resolve::resolve_label_names(client, names).await?;
            for lid in resolved {
                if !current_ids.contains(&lid) {
                    current_ids.push(lid);
                }
            }
        }

        // Remove labels by name
        if let Some(ref names) = remove_labels {
            let resolved = resolve::resolve_label_names(client, names).await?;
            current_ids.retain(|id| !resolved.contains(id));
        }

        // Merge with explicit label_ids if provided
        if let Some(ref explicit) = final_label_ids {
            for lid in explicit {
                if !current_ids.contains(lid) {
                    current_ids.push(lid.clone());
                }
            }
        }

        final_label_ids = Some(current_ids);
    }

    // Resolve parent if it's an identifier
    let resolved_parent = if let Some(ref pid) = parent {
        Some(resolve::resolve_issue_identifier(client, pid).await?)
    } else {
        None
    };

    let resolved_assignee = match assignee {
        Some(aid) => Some(resolve::resolve_user_identifier(client, &aid).await?),
        None => None,
    };

    let resolved_state = match state {
        Some(ref s) => Some(resolve::resolve_state_name(client, id, s).await?),
        None => None,
    };

    let resolved_project = match project {
        Some(ref p) => Some(resolve::resolve_project_identifier(client, p).await?),
        None => None,
    };

    let input = IssueUpdateInput {
        title,
        description,
        priority,
        assignee_id: resolved_assignee,
        state_id: resolved_state,
        project_id: resolved_project,
        label_ids: final_label_ids,
        parent_id: resolved_parent,
    };

    let data: IssueUpdateData = client
        .execute(
            ISSUE_UPDATE_MUTATION,
            Some(json!({ "id": id, "input": input })),
        )
        .await?;

    if !data.issue_update.success {
        bail!("Failed to update issue");
    }

    if let Some(issue) = data.issue_update.issue {
        output::print_success(&format!("Updated {} — {}", issue.identifier, issue.title));

        // Handle attachment upload
        if let Some(ref file_path) = attachment_path {
            let asset_url = upload::upload_file(client, file_path).await?;
            let filename = std::path::Path::new(file_path.as_str())
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "attachment".to_string());
            upload::create_attachment(client, &issue.id, &asset_url, &filename).await?;
            output::print_success(&format!("Attached: {}", filename));
        }
    }

    Ok(())
}

pub async fn search(
    client: &LinearClient,
    query: &str,
    project: Option<&str>,
    team: Option<&str>,
    assignee: Option<&str>,
    status: Option<&str>,
    limit: i32,
) -> Result<()> {
    let mut filter = json!({});
    if let Some(pid) = project {
        let resolved = resolve::resolve_project_identifier(client, pid).await?;
        filter["project"] = json!({ "id": { "eq": resolved } });
    }
    if let Some(tid) = team {
        let resolved = resolve::resolve_team_identifier(client, tid).await?;
        filter["team"] = json!({ "id": { "eq": resolved } });
    }
    if let Some(aid) = assignee {
        let resolved = resolve::resolve_user_identifier(client, aid).await?;
        filter["assignee"] = json!({ "id": { "eq": resolved } });
    }
    if let Some(s) = status {
        filter["state"] = json!({ "name": { "eq": s } });
    }

    let variables = json!({
        "term": query,
        "first": limit,
        "filter": filter,
    });

    let data: IssueSearchData = client.execute(ISSUE_SEARCH_QUERY, Some(variables)).await?;
    let issues = data.search_issues.nodes;

    print_issues_table(&issues);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn list(
    client: &LinearClient,
    team: Option<&str>,
    assignee: Option<&str>,
    creator: Option<&str>,
    status: Option<&str>,
    project: Option<&str>,
    priority: Option<i32>,
    labels: Option<&[String]>,
    cycle: Option<&str>,
    date_filters: DateFilters,
    convenience_filters: ConvenienceFilters,
    limit: i32,
) -> Result<()> {
    let mut filter = json!({});

    // Resolve team first since cycle depends on it
    let resolved_team_id = if let Some(tid) = team {
        let resolved = resolve::resolve_team_identifier(client, tid).await?;
        filter["team"] = json!({ "id": { "eq": &resolved } });
        Some(resolved)
    } else {
        None
    };

    if let Some(aid) = assignee {
        let resolved = resolve::resolve_user_identifier(client, aid).await?;
        filter["assignee"] = json!({ "id": { "eq": resolved } });
    }
    if let Some(cid) = creator {
        let resolved = resolve::resolve_user_identifier(client, cid).await?;
        filter["creator"] = json!({ "id": { "eq": resolved } });
    }
    if let Some(s) = status {
        filter["state"] = json!({ "name": { "eq": s } });
    }
    if let Some(pid) = project {
        let resolved = resolve::resolve_project_identifier(client, pid).await?;
        filter["project"] = json!({ "id": { "eq": resolved } });
    }
    if let Some(p) = priority {
        filter["priority"] = json!({ "eq": p });
    }

    // Handle cycle filter (requires team)
    if let Some(cyc) = cycle {
        let team_id = resolved_team_id.as_ref().ok_or_else(|| {
            anyhow::anyhow!("--cycle requires --team to be specified (cycles are team-scoped)")
        })?;
        let resolved = resolve::resolve_cycle_identifier(client, team_id, cyc).await?;
        filter["cycle"] = json!({ "id": { "eq": resolved } });
    }

    // Apply date filters
    apply_date_filter(
        &mut filter,
        "updatedAt",
        date_filters.updated_since.as_deref(),
        date_filters.updated_before.as_deref(),
    )?;
    apply_date_filter(
        &mut filter,
        "createdAt",
        date_filters.created_since.as_deref(),
        date_filters.created_before.as_deref(),
    )?;
    apply_date_filter(
        &mut filter,
        "completedAt",
        date_filters.completed_since.as_deref(),
        date_filters.completed_before.as_deref(),
    )?;
    apply_date_filter(
        &mut filter,
        "dueDate",
        date_filters.due_after.as_deref(),
        date_filters.due_before.as_deref(),
    )?;

    // Apply cancelled_since filter
    if let Some(ref cancelled_since) = date_filters.cancelled_since {
        let parsed = date::parse_date(cancelled_since)?;
        filter["cancelledAt"] = json!({ "gte": parsed });
    }

    // Apply estimate filters
    apply_estimate_filter(
        &mut filter,
        convenience_filters.estimate,
        convenience_filters.estimate_gte,
        convenience_filters.estimate_lte,
    );

    // Apply parent filter
    if let Some(ref parent_id) = convenience_filters.parent {
        let resolved = resolve::resolve_issue_identifier(client, parent_id).await?;
        filter["parent"] = json!({ "id": { "eq": resolved } });
    }

    // Apply no-parent filter (top-level issues only)
    if convenience_filters.no_parent {
        filter["parent"] = json!({ "null": true });
    }

    // Apply has-children filter
    if convenience_filters.has_children {
        filter["children"] = json!({ "some": {} });
    }

    // Apply subscriber filter
    if let Some(ref subscriber) = convenience_filters.subscriber {
        let resolved = resolve::resolve_user_identifier(client, subscriber).await?;
        filter["subscribers"] = json!({ "some": { "id": { "eq": resolved } } });
    }

    // Apply title filter
    if let Some(ref title) = convenience_filters.title {
        filter["title"] = json!({ "contains": title });
    }

    // Handle label filters with AND logic (multiple --label flags require all labels)
    let final_filter = if let Some(label_names) = labels {
        if label_names.is_empty() {
            filter
        } else {
            // Build array of label conditions that will be ANDed together
            let mut and_conditions: Vec<serde_json::Value> = label_names
                .iter()
                .map(|name| json!({ "labels": { "some": { "name": { "eqIgnoreCase": name } } } }))
                .collect();

            // Add the base filter to the AND conditions if it has any fields
            if filter.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
                and_conditions.push(filter);
            }

            json!({ "and": and_conditions })
        }
    } else {
        filter
    };

    let variables = json!({
        "first": limit,
        "filter": final_filter,
    });

    let data: IssuesData = client.execute(ISSUES_QUERY, Some(variables)).await?;
    let issues = data.issues.nodes;

    print_issues_table(&issues);
    Ok(())
}

/// Applies a date filter to the filter object.
/// For updatedAt/createdAt/completedAt: uses "gte" for since and "lt" for before
/// For dueDate: uses "gt" for after and "lt" for before
fn apply_date_filter(
    filter: &mut serde_json::Value,
    field: &str,
    since_or_after: Option<&str>,
    before: Option<&str>,
) -> Result<()> {
    let since_key = if field == "dueDate" { "gt" } else { "gte" };

    let since_parsed = since_or_after.map(date::parse_date).transpose()?;
    let before_parsed = before.map(date::parse_date).transpose()?;

    match (since_parsed, before_parsed) {
        (Some(s), Some(b)) => {
            filter[field] = json!({ since_key: s, "lt": b });
        }
        (Some(s), None) => {
            filter[field] = json!({ since_key: s });
        }
        (None, Some(b)) => {
            filter[field] = json!({ "lt": b });
        }
        (None, None) => {}
    }

    Ok(())
}

/// Applies estimate filters to the filter object.
/// Supports exact match, greater-than-or-equal, and less-than-or-equal comparisons.
fn apply_estimate_filter(
    filter: &mut serde_json::Value,
    exact: Option<f64>,
    gte: Option<f64>,
    lte: Option<f64>,
) {
    if let Some(val) = exact {
        filter["estimate"] = json!({ "eq": val });
    } else {
        match (gte, lte) {
            (Some(g), Some(l)) => {
                filter["estimate"] = json!({ "gte": g, "lte": l });
            }
            (Some(g), None) => {
                filter["estimate"] = json!({ "gte": g });
            }
            (None, Some(l)) => {
                filter["estimate"] = json!({ "lte": l });
            }
            (None, None) => {}
        }
    }
}

pub async fn me(client: &LinearClient, status: Option<&str>, limit: i32) -> Result<()> {
    let viewer: ViewerData = client.execute(VIEWER_QUERY, None).await?;
    let user_id = viewer.viewer.id;

    let mut filter = json!({
        "assignee": { "id": { "eq": user_id } }
    });
    if let Some(s) = status {
        filter["state"] = json!({ "name": { "eq": s } });
    }

    let variables = json!({
        "first": limit,
        "filter": filter,
    });

    let data: IssuesData = client.execute(ISSUES_QUERY, Some(variables)).await?;
    let issues = data.issues.nodes;

    output::print_header(&format!(
        "My Issues ({})",
        viewer
            .viewer
            .display_name
            .as_deref()
            .unwrap_or(&viewer.viewer.name)
    ));
    print_issues_table(&issues);
    Ok(())
}

pub async fn state(
    client: &LinearClient,
    id: &str,
    new_state_name: Option<&str>,
    list_flag: bool,
) -> Result<()> {
    // First fetch the issue to get current state and team
    let issue_data: IssueData = client
        .execute(ISSUE_QUERY, Some(json!({ "id": id })))
        .await?;
    let issue = issue_data.issue;

    if list_flag {
        // List all available states grouped by type
        let team = issue
            .team
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;

        let team_data: TeamData = client
            .execute(TEAM_STATES_QUERY, Some(json!({ "id": team.id })))
            .await?;

        let current_state_id = issue.state.as_ref().map(|s| s.id.as_str());

        output::print_header(&format!(
            "States for {} (team: {})",
            issue.identifier, team.name
        ));

        // Group by type
        let type_order = ["backlog", "unstarted", "started", "completed", "cancelled"];
        for state_type in &type_order {
            let states_of_type: Vec<&WorkflowState> = team_data
                .team
                .states
                .nodes
                .iter()
                .filter(|s| {
                    s.state_type
                        .as_deref()
                        .map(|t| t.to_lowercase() == *state_type)
                        .unwrap_or(false)
                })
                .collect();

            if !states_of_type.is_empty() {
                println!();
                output::print_field("Type", state_type);
                for s in &states_of_type {
                    let marker = if Some(s.id.as_str()) == current_state_id {
                        " ← current"
                    } else {
                        ""
                    };
                    println!("    {}{}", s.name, marker);
                }
            }
        }

        // Any states with unknown types
        let known_types: Vec<&str> = type_order.to_vec();
        let other: Vec<&WorkflowState> = team_data
            .team
            .states
            .nodes
            .iter()
            .filter(|s| {
                s.state_type
                    .as_deref()
                    .map(|t| !known_types.contains(&t.to_lowercase().as_str()))
                    .unwrap_or(true)
            })
            .collect();

        if !other.is_empty() {
            println!();
            output::print_field("Type", "other");
            for s in &other {
                let marker = if Some(s.id.as_str()) == current_state_id {
                    " ← current"
                } else {
                    ""
                };
                println!("    {}{}", s.name, marker);
            }
        }

        return Ok(());
    }

    match new_state_name {
        None => {
            // Just display current state
            let state_name = issue
                .state
                .as_ref()
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown");
            output::print_header(&format!("{} — {}", issue.identifier, issue.title));
            output::print_field("State", state_name);
        }
        Some(target_name) => {
            // Get team workflow states
            let team = issue
                .team
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;

            let team_data: TeamData = client
                .execute(TEAM_STATES_QUERY, Some(json!({ "id": team.id })))
                .await?;

            let target_lower = target_name.to_lowercase();
            let matching_state = team_data
                .team
                .states
                .nodes
                .iter()
                .find(|s| s.name.to_lowercase() == target_lower);

            let ws = matching_state.ok_or_else(|| {
                let available: Vec<&str> = team_data
                    .team
                    .states
                    .nodes
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect();
                anyhow::anyhow!(
                    "State '{}' not found. Available states: {}",
                    target_name,
                    available.join(", ")
                )
            })?;

            let input = IssueUpdateInput {
                state_id: Some(ws.id.clone()),
                ..Default::default()
            };

            let data: IssueUpdateData = client
                .execute(
                    ISSUE_UPDATE_MUTATION,
                    Some(json!({ "id": issue.id, "input": input })),
                )
                .await?;

            if !data.issue_update.success {
                bail!("Failed to update issue state");
            }

            output::print_success(&format!(
                "{} state changed to '{}'",
                issue.identifier, ws.name
            ));
        }
    }

    Ok(())
}

pub async fn attachment_add(
    client: &LinearClient,
    id: &str,
    file_path: &str,
    title: Option<&str>,
) -> Result<()> {
    let issue_id = resolve::resolve_issue_identifier(client, id).await?;
    let asset_url = upload::upload_file(client, file_path).await?;
    let filename = std::path::Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "attachment".to_string());
    let attachment_title = title.unwrap_or(&filename);
    upload::create_attachment(client, &issue_id, &asset_url, attachment_title).await?;
    output::print_success(&format!("Attached '{}' to {}", attachment_title, id));
    Ok(())
}

pub async fn attachments(client: &LinearClient, id: &str) -> Result<()> {
    let data: IssueAttachmentsData = client
        .execute(ISSUE_ATTACHMENTS_QUERY, Some(json!({ "id": id })))
        .await?;

    let attachments = data.issue.attachments.nodes;
    output::print_header(&format!("Attachments ({})", attachments.len()));

    if attachments.is_empty() {
        println!("  No attachments.");
        return Ok(());
    }

    let headers = &["ID", "Title", "URL", "Created"];
    let rows: Vec<Vec<String>> = attachments
        .iter()
        .map(|a| {
            vec![
                truncate(&a.id, 8),
                a.title.clone().unwrap_or_default(),
                a.url.clone().unwrap_or_default(),
                a.created_at
                    .as_deref()
                    .map(output::format_date)
                    .unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

fn print_issues_table(issues: &[Issue]) {
    output::print_header(&format!("Issues ({})", issues.len()));

    let headers = &["ID", "Title", "Status", "Assignee", "Team"];
    let rows: Vec<Vec<String>> = issues
        .iter()
        .map(|i| {
            vec![
                i.identifier.clone(),
                truncate(&i.title, 50),
                i.state.as_ref().map(|s| s.name.clone()).unwrap_or_default(),
                i.assignee
                    .as_ref()
                    .map(|a| a.name.clone())
                    .unwrap_or_default(),
                i.team.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
