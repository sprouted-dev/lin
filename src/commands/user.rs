use anyhow::Result;

use crate::api::client::LinearClient;
use crate::api::queries::*;
use crate::api::types::*;
use crate::output;

pub async fn list(client: &LinearClient, json: bool) -> Result<()> {
    if json {
        let data = client.execute_raw(USERS_QUERY, None).await?;
        output::print_json(&data);
        return Ok(());
    }

    let data: UsersData = client.execute(USERS_QUERY, None).await?;

    let users = data.users.nodes;
    output::print_header(&format!("Users ({})", users.len()));

    let headers = &["Name", "Display Name", "Email"];
    let rows: Vec<Vec<String>> = users
        .iter()
        .map(|u| {
            vec![
                u.name.clone(),
                u.display_name.clone().unwrap_or_default(),
                u.email.clone().unwrap_or_default(),
            ]
        })
        .collect();

    output::print_table(headers, &rows);
    Ok(())
}
