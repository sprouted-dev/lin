use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::resolve;
use crate::api::types::*;
use crate::output;

pub async fn list(
    client: &LinearClient,
    team: Option<&str>,
    template_type: Option<&str>,
    global_only: bool,
    json_output: bool,
) -> Result<()> {
    let templates = resolve::fetch_templates(client, team, template_type, global_only).await?;

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
                resolve::template_owner(t),
                t.description.clone().unwrap_or_default(),
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
    let template_id = resolve::resolve_template_identifier(client, template, team, None).await?;

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
    output::print_field("Team", &resolve::template_owner(&template));
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
