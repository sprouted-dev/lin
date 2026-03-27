use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::resolve;
use crate::api::types::*;
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
