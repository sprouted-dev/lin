use anyhow::{Result, bail};
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::resolve;
use crate::api::types::*;
use crate::date;
use crate::output;

pub async fn list(client: &LinearClient, team: &str, limit: i32, json: bool) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;

    let variables = json!({
        "first": limit,
        "filter": { "team": { "id": { "eq": team_id } } },
    });

    if json {
        let data = client.execute_raw(CYCLES_QUERY, Some(variables)).await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: CyclesData = client.execute(CYCLES_QUERY, Some(variables)).await?;
    let cycles = data.cycles.nodes;

    output::print_header(&format!("Cycles ({})", cycles.len()));

    let headers = &["Number", "Name", "Start", "End"];
    let rows: Vec<Vec<String>> = cycles
        .iter()
        .map(|c| {
            vec![
                c.number.map(|n| n.to_string()).unwrap_or_default(),
                c.name.clone().unwrap_or_default(),
                c.starts_at
                    .as_deref()
                    .map(output::format_date)
                    .unwrap_or_default(),
                c.ends_at
                    .as_deref()
                    .map(output::format_date)
                    .unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

pub async fn active(client: &LinearClient, team: &str, json: bool) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;

    let variables = json!({
        "first": 50,
        "filter": {
            "team": { "id": { "eq": team_id } },
            "isActive": { "eq": true }
        },
    });

    if json {
        let data = client.execute_raw(CYCLES_QUERY, Some(variables)).await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: CyclesData = client.execute(CYCLES_QUERY, Some(variables)).await?;
    let cycles = data.cycles.nodes;

    if cycles.is_empty() {
        println!("No active cycle found.");
        return Ok(());
    }

    let cycle = &cycles[0];
    output::print_header("Active Cycle");
    if let Some(ref name) = cycle.name {
        output::print_field("Name", name);
    }
    if let Some(number) = cycle.number {
        output::print_field("Number", &number.to_string());
    }
    if let Some(ref start) = cycle.starts_at {
        output::print_field("Start", &output::format_date(start));
    }
    if let Some(ref end) = cycle.ends_at {
        output::print_field("End", &output::format_date(end));
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    client: &LinearClient,
    team: &str,
    starts: &str,
    ends: Option<&str>,
    duration: Option<&str>,
    name: Option<&str>,
    description: Option<&str>,
    json_flag: bool,
) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;

    let starts_at = date::parse_date(starts)?;

    let ends_at = if let Some(end) = ends {
        date::parse_date(end)?
    } else if let Some(dur) = duration {
        date::add_duration_to_date(starts, dur)?
    } else {
        bail!("Either --ends or --duration is required")
    };

    let input = CycleCreateInput {
        team_id,
        starts_at,
        ends_at,
        name: name.map(|s| s.to_string()),
        description: description.map(|s| s.to_string()),
    };

    if json_flag {
        let data = client
            .execute_raw(CYCLE_CREATE_MUTATION, Some(json!({ "input": input })))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: CycleCreateData = client
        .execute(CYCLE_CREATE_MUTATION, Some(json!({ "input": input })))
        .await?;

    if !data.cycle_create.success {
        bail!("Failed to create cycle");
    }

    if let Some(cycle) = data.cycle_create.cycle {
        let label = cycle.name.as_deref().unwrap_or("(unnamed)");
        let num = cycle.number.map(|n| format!(" #{}", n)).unwrap_or_default();
        output::print_success(&format!("Created cycle {}{}", label, num));
        if let Some(ref start) = cycle.starts_at {
            output::print_field("Start", &output::format_date(start));
        }
        if let Some(ref end) = cycle.ends_at {
            output::print_field("End", &output::format_date(end));
        }
    }

    Ok(())
}

pub async fn edit(
    client: &LinearClient,
    id: &str,
    team: &str,
    name: Option<String>,
    description: Option<String>,
    starts: Option<&str>,
    ends: Option<&str>,
    json_flag: bool,
) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;
    let cycle_id = resolve::resolve_cycle_identifier(client, &team_id, id).await?;

    let mut input = CycleUpdateInput::default();

    if let Some(n) = name {
        input.name = Some(n);
    }
    if let Some(d) = description {
        input.description = Some(d);
    }
    if let Some(s) = starts {
        input.starts_at = Some(date::parse_date(s)?);
    }
    if let Some(e) = ends {
        input.ends_at = Some(date::parse_date(e)?);
    }

    if input.name.is_none()
        && input.description.is_none()
        && input.starts_at.is_none()
        && input.ends_at.is_none()
    {
        bail!("No updates provided. Use --name, --description, --starts, or --ends.");
    }

    if json_flag {
        let data = client
            .execute_raw(
                CYCLE_UPDATE_MUTATION,
                Some(json!({ "id": cycle_id, "input": input })),
            )
            .await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: CycleUpdateData = client
        .execute(
            CYCLE_UPDATE_MUTATION,
            Some(json!({ "id": cycle_id, "input": input })),
        )
        .await?;

    if !data.cycle_update.success {
        bail!("Failed to update cycle");
    }

    if let Some(cycle) = data.cycle_update.cycle {
        let label = cycle.name.as_deref().unwrap_or("(unnamed)");
        let num = cycle.number.map(|n| format!(" #{}", n)).unwrap_or_default();
        output::print_success(&format!("Updated cycle {}{}", label, num));
        if let Some(ref desc) = cycle.description {
            output::print_field("Description", desc);
        }
        if let Some(ref start) = cycle.starts_at {
            output::print_field("Start", &output::format_date(start));
        }
        if let Some(ref end) = cycle.ends_at {
            output::print_field("End", &output::format_date(end));
        }
    }

    Ok(())
}

pub async fn show(client: &LinearClient, id: &str, team: &str, json_flag: bool) -> Result<()> {
    let team_id = resolve::resolve_team_identifier(client, team).await?;
    let cycle_id = resolve::resolve_cycle_identifier(client, &team_id, id).await?;

    let variables = json!({ "id": cycle_id });

    if json_flag {
        let data = client
            .execute_raw(CYCLE_DETAIL_QUERY, Some(variables))
            .await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: CycleDetailData = client.execute(CYCLE_DETAIL_QUERY, Some(variables)).await?;
    let cycle = data.cycle;

    // Header
    let title = cycle.name.as_deref().unwrap_or("Unnamed Cycle");
    let num = cycle.number.map(|n| format!(" #{}", n)).unwrap_or_default();
    output::print_header(&format!("{}{}", title, num));

    // Status
    let status = if cycle.is_active == Some(true) {
        "Active"
    } else if cycle.is_future == Some(true) {
        "Upcoming"
    } else if cycle.is_past == Some(true) {
        "Past"
    } else {
        "Unknown"
    };
    output::print_field("Status", status);

    // Dates
    if let Some(ref start) = cycle.starts_at {
        output::print_field("Start", &output::format_date(start));
    }
    if let Some(ref end) = cycle.ends_at {
        output::print_field("End", &output::format_date(end));
    }

    // Progress
    if let Some(progress) = cycle.progress {
        let pct = (progress * 100.0) as u32;
        let bar = format_progress_bar(progress, 20);
        output::print_field("Progress", &format!("{} {}%", bar, pct));
    }

    // Description
    if let Some(ref desc) = cycle.description
        && !desc.is_empty()
    {
        println!();
        output::print_header("Description");
        println!("  {}", desc);
    }

    // Issues table
    if let Some(ref issues) = cycle.issues {
        println!();
        output::print_header(&format!("Issues ({})", issues.nodes.len()));

        let headers = &["ID", "Title", "Status", "Assignee"];
        let rows: Vec<Vec<String>> = issues
            .nodes
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
                ]
            })
            .collect();

        output::print_table(headers, &rows);
    }

    Ok(())
}

fn format_progress_bar(progress: f64, width: usize) -> String {
    let filled = (progress * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
