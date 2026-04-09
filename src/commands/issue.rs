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

pub async fn view(client: &LinearClient, id: &str, json: bool) -> Result<()> {
    if json {
        let data = client
            .execute_raw(ISSUE_QUERY, Some(json!({ "id": id })))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

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
    if let Some(ref cycle) = issue.cycle {
        let display = match (&cycle.name, cycle.number) {
            (Some(name), Some(num)) => format!("{} (#{})", name, num),
            (Some(name), None) => name.clone(),
            (None, Some(num)) => format!("#{}", num),
            (None, None) => "—".to_string(),
        };
        output::print_field("Cycle", &display);
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
    cycle: Option<&str>,
    attachment_path: Option<&str>,
) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;

    let mut input = IssueCreateInput {
        title: title.to_string(),
        team_id: team_id.clone(),
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

    // Resolve cycle
    if let Some(cyc) = cycle {
        input.cycle_id = Some(resolve::resolve_cycle_identifier(client, &team_id, cyc).await?);
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
    cycle: Option<String>,
    attachment_path: Option<String>,
) -> Result<()> {
    // Resolve label names — fetch issue if labels or cycle need it
    let mut final_label_ids = label_ids;
    let needs_issue_fetch = labels.is_some() || remove_labels.is_some() || cycle.is_some();

    let fetched_issue = if needs_issue_fetch {
        let issue_data: IssueData = client
            .execute(ISSUE_QUERY, Some(json!({ "id": id })))
            .await?;
        Some(issue_data.issue)
    } else {
        None
    };

    if labels.is_some() || remove_labels.is_some() {
        let issue = fetched_issue.as_ref().unwrap();
        let mut current_ids: Vec<String> = issue
            .labels
            .as_ref()
            .map(|l| l.nodes.iter().map(|n| n.id.clone()).collect())
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

    // Resolve cycle — get team from fetched issue
    let resolved_cycle = if let Some(ref cyc) = cycle {
        let issue = fetched_issue.as_ref().unwrap();
        let team = issue
            .team
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Issue has no team; cannot resolve cycle"))?;
        Some(resolve::resolve_cycle_identifier(client, &team.id, cyc).await?)
    } else {
        None
    };

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
        cycle_id: resolved_cycle,
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

#[allow(clippy::too_many_arguments)]
pub async fn search(
    client: &LinearClient,
    query: &str,
    project: Option<&str>,
    team: Option<&str>,
    assignee: Option<&str>,
    status: Option<&str>,
    limit: i32,
    json: bool,
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

    if json {
        let data = client
            .execute_raw(ISSUE_SEARCH_QUERY, Some(variables))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

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
    json: bool,
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

    if json {
        let data = client.execute_raw(ISSUES_QUERY, Some(variables)).await?;
        output::print_json(&data);
        return Ok(());
    }

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

pub async fn me(client: &LinearClient, status: Option<&str>, limit: i32, json: bool) -> Result<()> {
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

    if json {
        let data = client.execute_raw(ISSUES_QUERY, Some(variables)).await?;
        output::print_json(&data);
        return Ok(());
    }

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
    json: bool,
) -> Result<()> {
    // When just viewing state as JSON, return raw data without a typed fetch first
    if json && !list_flag && new_state_name.is_none() {
        let data = client
            .execute_raw(ISSUE_QUERY, Some(json!({ "id": id })))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

    // Fetch the issue to get current state and team
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

        if json {
            let data = client
                .execute_raw(TEAM_STATES_QUERY, Some(json!({ "id": team.id })))
                .await?;
            output::print_json(&data);
            return Ok(());
        }

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

pub async fn attachments(client: &LinearClient, id: &str, json: bool) -> Result<()> {
    if json {
        let data = client
            .execute_raw(ISSUE_ATTACHMENTS_QUERY, Some(json!({ "id": id })))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

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

pub async fn attachment_download(
    client: &LinearClient,
    id: &str,
    output_dir: &str,
    filter_id: Option<&str>,
) -> Result<()> {
    let data: IssueDownloadData = client
        .execute(ISSUE_ATTACHMENTS_DOWNLOAD_QUERY, Some(json!({ "id": id })))
        .await?;

    let mut download_urls: Vec<String> = Vec::new();

    // Collect formal attachment URLs (only uploads.linear.app)
    for att in &data.issue.attachments.nodes {
        let Some(url) = &att.url else { continue };

        if let Some(fid) = filter_id
            && !att.id.starts_with(fid)
        {
            continue;
        }

        if !url.contains("uploads.linear.app") {
            output::print_field(
                "Skipping",
                &format!(
                    "{} (not a Linear upload)",
                    att.title.as_deref().unwrap_or(url)
                ),
            );
            continue;
        }

        if !download_urls.contains(url) {
            download_urls.push(url.clone());
        }
    }

    // Collect inline uploads from description (skip when filtering by ID)
    if filter_id.is_none()
        && let Some(desc) = &data.issue.description
    {
        for url in extract_inline_upload_urls(desc) {
            if !download_urls.contains(&url) {
                download_urls.push(url);
            }
        }
    }

    if download_urls.is_empty() {
        println!("  No attachments to download.");
        return Ok(());
    }

    let output_path = std::path::Path::new(output_dir);
    std::fs::create_dir_all(output_path)?;

    let http_client = reqwest::Client::new();
    let token = client.token();
    let mut success_count = 0;
    let mut used_filenames: Vec<String> = Vec::new();

    for url in &download_urls {
        match download_file(&http_client, url, token, output_path, &mut used_filenames).await {
            Ok((filename, bytes)) => {
                success_count += 1;
                output::print_success(&format!("{} ({})", filename, format_byte_size(bytes)));
            }
            Err(e) => {
                output::print_error(&format!("Failed to download '{}': {}", url, e));
            }
        }
    }

    if download_urls.len() > 1 {
        output::print_success(&format!(
            "Downloaded {}/{} files to {}",
            success_count,
            download_urls.len(),
            output_dir
        ));
    }
    Ok(())
}

fn is_linear_upload_url(url: &str) -> bool {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h == "uploads.linear.app"))
        .unwrap_or(false)
}

pub(crate) async fn download_file(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    output_dir: &std::path::Path,
    used_filenames: &mut Vec<String>,
) -> Result<(String, usize)> {
    let mut request = client.get(url);
    if is_linear_upload_url(url) {
        request = request.header("Authorization", token);
    }
    let response = request.send().await?;
    if !response.status().is_success() {
        bail!("HTTP {}", response.status());
    }

    let raw_name =
        content_disposition_filename(&response).unwrap_or_else(|| "download".to_string());

    // Sanitize: strip path components to prevent traversal
    let safe_name = std::path::Path::new(&raw_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());

    // Deduplicate: append suffix if filename already used
    let filename = deduplicate_filename(&safe_name, used_filenames);
    used_filenames.push(filename.clone());

    let bytes = response.bytes().await?;
    let len = bytes.len();
    std::fs::write(output_dir.join(&filename), &bytes)?;
    Ok((filename, len))
}

fn deduplicate_filename(name: &str, used: &[String]) -> String {
    if !used.contains(&name.to_string()) {
        return name.to_string();
    }
    let stem = std::path::Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let ext = std::path::Path::new(name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()));
    let mut i = 1;
    loop {
        let candidate = format!("{}-{}{}", stem, i, ext.as_deref().unwrap_or(""));
        if !used.contains(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

fn content_disposition_filename(response: &reqwest::Response) -> Option<String> {
    let header = response.headers().get("content-disposition")?;
    let value = header.to_str().ok()?;
    // Parse: attachment; filename="name.ext" or filename=name.ext; ...
    let after = value.split("filename=").nth(1)?;
    let name = after
        .split(';')
        .next()?
        .trim()
        .trim_matches('"')
        .trim_matches('\'');
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn extract_inline_upload_urls(text: &str) -> Vec<String> {
    let prefix = "https://uploads.linear.app/";
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(start) = text[search_from..].find(prefix) {
        let abs_start = search_from + start;
        let rest = &text[abs_start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ')' || c == ']' || c == '>' || c == '"')
            .unwrap_or(rest.len());
        let url = text[abs_start..abs_start + end].to_string();
        results.push(url);
        search_from = abs_start + end;
    }
    results
}

pub(crate) fn format_byte_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_inline_urls_from_markdown() {
        let desc = r#"Some text
![image001.png](https://uploads.linear.app/org/abc/def)
[report.docx](https://uploads.linear.app/org/123/456)
No link here."#;
        let urls = extract_inline_upload_urls(desc);
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://uploads.linear.app/org/abc/def");
        assert_eq!(urls[1], "https://uploads.linear.app/org/123/456");
    }

    #[test]
    fn extract_inline_urls_bare() {
        let desc = "Check https://uploads.linear.app/org/a/b for details";
        let urls = extract_inline_upload_urls(desc);
        assert_eq!(urls, vec!["https://uploads.linear.app/org/a/b"]);
    }

    #[test]
    fn extract_inline_urls_none() {
        let urls = extract_inline_upload_urls("No uploads here");
        assert!(urls.is_empty());
    }

    #[test]
    fn extract_inline_urls_deduplicates_not_here() {
        // Dedup happens in the caller, not in extract — same URL should appear twice
        let desc = "[a](https://uploads.linear.app/x) [b](https://uploads.linear.app/x)";
        let urls = extract_inline_upload_urls(desc);
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn deduplicate_no_conflict() {
        let used = vec!["other.txt".to_string()];
        assert_eq!(deduplicate_filename("file.docx", &used), "file.docx");
    }

    #[test]
    fn deduplicate_one_conflict() {
        let used = vec!["file.docx".to_string()];
        assert_eq!(deduplicate_filename("file.docx", &used), "file-1.docx");
    }

    #[test]
    fn deduplicate_multiple_conflicts() {
        let used = vec!["file.docx".to_string(), "file-1.docx".to_string()];
        assert_eq!(deduplicate_filename("file.docx", &used), "file-2.docx");
    }

    #[test]
    fn deduplicate_no_extension() {
        let used = vec!["README".to_string()];
        assert_eq!(deduplicate_filename("README", &used), "README-1");
    }

    #[test]
    fn format_bytes() {
        assert_eq!(format_byte_size(500), "500 B");
        assert_eq!(format_byte_size(1024), "1.0 KB");
        assert_eq!(format_byte_size(1024 * 1024 * 2), "2.0 MB");
    }

    #[test]
    fn is_linear_upload_url_accepts_valid() {
        assert!(is_linear_upload_url(
            "https://uploads.linear.app/org/abc/def"
        ));
    }

    #[test]
    fn is_linear_upload_url_rejects_spoofed() {
        // Subdomain spoofing
        assert!(!is_linear_upload_url(
            "https://uploads.linear.app.evil.com/file"
        ));
        // Query param spoofing
        assert!(!is_linear_upload_url(
            "https://evil.com?ref=uploads.linear.app"
        ));
        // Path spoofing
        assert!(!is_linear_upload_url(
            "https://evil.com/uploads.linear.app"
        ));
    }

    #[test]
    fn is_linear_upload_url_rejects_non_url() {
        assert!(!is_linear_upload_url("not-a-url"));
        assert!(!is_linear_upload_url(""));
    }
}
