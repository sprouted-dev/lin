use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::resolve;
use crate::api::types::*;
use crate::output;

/// Resolve a team name/key to an ID once, so the resolver underneath does not
/// repeat the lookup.
async fn team_id_for(client: &LinearClient, team: Option<&str>) -> Result<Option<String>> {
    match team {
        Some(t) => Ok(Some(resolve::resolve_team_identifier(client, t).await?)),
        None => Ok(None),
    }
}

pub async fn list(
    client: &LinearClient,
    team: Option<&str>,
    template_type: Option<&str>,
    global_only: bool,
    json_output: bool,
) -> Result<()> {
    let team_id = team_id_for(client, team).await?;
    let templates =
        resolve::fetch_templates(client, team_id.as_deref(), template_type, global_only).await?;

    if json_output {
        output::print_json(&serde_json::to_value(&templates)?);
        return Ok(());
    }

    output::print_header(&format!("Templates ({})", templates.len()));

    let headers = &["Name", "Type", "Team", "Description"];
    let rows: Vec<Vec<String>> = templates
        .iter()
        .map(|t| {
            vec![
                t.name.clone(),
                t.template_type.clone(),
                resolve::owner_label(t.team.as_ref()),
                output::truncate(t.description.as_deref().unwrap_or_default(), 50),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

pub async fn view(
    client: &LinearClient,
    template: &str,
    team: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let team_id = team_id_for(client, team).await?;
    let template_id =
        resolve::resolve_template_identifier(client, template, team_id.as_deref(), None).await?;

    // Second round trip on purpose: TEMPLATE_SEARCH_QUERY omits content and
    // templateData so that listing and name resolution stay cheap, and this is
    // the only path that needs them.
    let data: TemplateData = client
        .execute(TEMPLATE_QUERY, Some(json!({ "id": template_id })))
        .await?;
    let template = data.template;

    if json_output {
        output::print_json(&serde_json::to_value(&template)?);
        return Ok(());
    }

    output::print_header(&template.name);
    output::print_field("Type", &template.template_type);
    output::print_field("Team", &resolve::owner_label(template.team.as_ref()));
    if let Some(description) = &template.description {
        output::print_field("Description", description);
    }

    match &template.content {
        Some(content) if !content.is_empty() => {
            println!();
            println!("{content}");
        }
        // Form templates carry their fields in templateData rather than a
        // markdown body, so an empty content field is expected, not an error.
        _ => {
            println!();
            println!("(no content; use --json to see templateData)");
        }
    }

    Ok(())
}
