use anyhow::{Result, bail};
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn list(client: &LinearClient, include_archived: bool, limit: i32) -> Result<()> {
    let mut variables = json!({ "first": limit });
    if include_archived {
        variables["includeArchived"] = json!(true);
    }

    let data: ProjectsData = client.execute(PROJECTS_QUERY, Some(variables)).await?;

    let projects = data.projects.nodes;
    output::print_header(&format!("Projects ({})", projects.len()));

    let headers = &["Name", "State", "Lead", "Start", "Target"];
    let rows: Vec<Vec<String>> = projects
        .iter()
        .map(|p| {
            vec![
                p.name.clone(),
                p.state.clone().unwrap_or_default(),
                p.lead.as_ref().map(|u| u.name.clone()).unwrap_or_default(),
                p.start_date.clone().unwrap_or_default(),
                p.target_date.clone().unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

pub async fn view(client: &LinearClient, id: &str) -> Result<()> {
    let data: ProjectDetailData = client
        .execute(PROJECT_QUERY, Some(json!({ "id": id })))
        .await?;

    let p = data.project;

    output::print_header(&p.name);

    if let Some(ref state) = p.state {
        output::print_field("State", state);
    }
    if let Some(ref lead) = p.lead {
        output::print_field("Lead", &lead.name);
    }
    if let Some(ref members) = p.members
        && !members.nodes.is_empty()
    {
        let names: Vec<&str> = members.nodes.iter().map(|m| m.name.as_str()).collect();
        output::print_field("Members", &names.join(", "));
    }
    if let Some(ref start) = p.start_date {
        output::print_field("Start", start);
    }
    if let Some(ref target) = p.target_date {
        output::print_field("Target", target);
    }
    if let Some(ref desc) = p.description
        && !desc.is_empty()
    {
        println!();
        output::print_header("Description");
        println!("  {desc}");
    }
    if let Some(ref url) = p.url {
        output::print_field("URL", url);
    }

    Ok(())
}

pub async fn create(
    client: &LinearClient,
    name: &str,
    team_ids: &[String],
    description: Option<&str>,
) -> Result<()> {
    let mut input = json!({
        "name": name,
        "teamIds": team_ids,
    });

    if let Some(desc) = description {
        input["description"] = json!(desc);
    }

    let data: ProjectCreateData = client
        .execute(PROJECT_CREATE_MUTATION, Some(json!({ "input": input })))
        .await?;

    if !data.project_create.success {
        bail!("Failed to create project");
    }

    if let Some(project) = data.project_create.project {
        output::print_success(&format!("Created project: {}", project.name));
    }

    Ok(())
}

pub async fn edit(
    client: &LinearClient,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    state: Option<&str>,
) -> Result<()> {
    let mut input = json!({});
    if let Some(n) = name {
        input["name"] = json!(n);
    }
    if let Some(d) = description {
        input["description"] = json!(d);
    }
    if let Some(s) = state {
        input["state"] = json!(s);
    }

    let data: ProjectUpdateMutationData = client
        .execute(
            PROJECT_UPDATE_MUTATION,
            Some(json!({ "id": id, "input": input })),
        )
        .await?;

    if !data.project_update.success {
        bail!("Failed to update project");
    }

    if let Some(project) = data.project_update.project {
        output::print_success(&format!("Updated project: {}", project.name));
    }

    Ok(())
}

// --- Project Updates ---

pub async fn update_list(client: &LinearClient, project_id: &str) -> Result<()> {
    let data: ProjectUpdatesData = client
        .execute(PROJECT_UPDATES_QUERY, Some(json!({ "id": project_id })))
        .await?;

    let updates = data.project.project_updates.nodes;
    output::print_header(&format!("Project Updates ({})", updates.len()));

    let headers = &["ID", "Health", "Author", "Date"];
    let rows: Vec<Vec<String>> = updates
        .iter()
        .map(|u| {
            vec![
                truncate_id(&u.id),
                u.health.clone().unwrap_or_default(),
                u.user.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                u.created_at
                    .as_deref()
                    .map(output::format_date)
                    .unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}

fn validate_health(health: &str) -> Result<()> {
    match health {
        "onTrack" | "atRisk" | "offTrack" => Ok(()),
        _ => bail!(
            "Invalid health value '{}'. Must be one of: onTrack, atRisk, offTrack",
            health
        ),
    }
}

pub async fn update_add(
    client: &LinearClient,
    project_id: &str,
    body: &str,
    health: Option<&str>,
) -> Result<()> {
    if let Some(h) = health {
        validate_health(h)?;
    }

    let mut input = json!({
        "projectId": project_id,
        "body": body,
    });

    if let Some(h) = health {
        input["health"] = json!(h);
    }

    let data: ProjectUpdateCreateData = client
        .execute(
            PROJECT_UPDATE_CREATE_MUTATION,
            Some(json!({ "input": input })),
        )
        .await?;

    if !data.project_update_create.success {
        bail!("Failed to create project update");
    }

    output::print_success("Project update added");
    Ok(())
}

pub async fn update_edit(
    client: &LinearClient,
    update_id: &str,
    body: &str,
    health: Option<&str>,
) -> Result<()> {
    if let Some(h) = health {
        validate_health(h)?;
    }

    let mut input = json!({ "body": body });

    if let Some(h) = health {
        input["health"] = json!(h);
    }

    let data: ProjectUpdateEditData = client
        .execute(
            PROJECT_UPDATE_EDIT_MUTATION,
            Some(json!({ "id": update_id, "input": input })),
        )
        .await?;

    if !data.project_update_update.success {
        bail!("Failed to edit project update");
    }

    output::print_success("Project update edited");
    Ok(())
}

pub async fn update_delete(client: &LinearClient, update_id: &str) -> Result<()> {
    let data: ProjectUpdateDeleteData = client
        .execute(
            PROJECT_UPDATE_DELETE_MUTATION,
            Some(json!({ "id": update_id })),
        )
        .await?;

    if !data.project_update_delete.success {
        bail!("Failed to delete project update");
    }

    output::print_success("Project update deleted");
    Ok(())
}

fn truncate_id(id: &str) -> String {
    if id.len() > 8 {
        format!("{}…", &id[..8])
    } else {
        id.to_string()
    }
}
