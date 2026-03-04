mod api;
mod auth;
mod cli;
mod commands;
mod config;
mod error;
mod output;
mod workspace;

use anyhow::Result;
use clap::Parser;

use api::client::LinearClient;
use cli::*;

struct CommandContext {
    client: LinearClient,
    #[allow(dead_code)]
    workspace: String,
}

fn ensure_auth(cli_workspace: Option<&str>) -> Result<CommandContext> {
    let ws = workspace::resolve_workspace(cli_workspace);
    let token = auth::get_token(&ws)?;
    let client = LinearClient::new(&token);
    Ok(CommandContext {
        client,
        workspace: ws,
    })
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        output::print_error(&format!("{e}"));
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let ws_flag = cli.workspace.as_deref();

    match cli.command {
        Commands::Login { token, name } => {
            commands::login::run(&token, &name).await?;
        }

        Commands::Workspace(cmd) => match cmd {
            WorkspaceCommand::Current => {
                commands::workspace_cmd::current(ws_flag);
            }
            WorkspaceCommand::List => {
                commands::workspace_cmd::list()?;
            }
            WorkspaceCommand::Set { name, global } => {
                commands::workspace_cmd::set(&name, global)?;
            }
        },

        Commands::Issue(cmd) => {
            let ctx = ensure_auth(ws_flag)?;
            match cmd {
                IssueCommand::View { id } => {
                    commands::issue::view(&ctx.client, &id).await?;
                }
                IssueCommand::Create {
                    title,
                    team_id,
                    description,
                    priority,
                    assignee_id,
                    project_id,
                    label_ids,
                    labels,
                    parent_id,
                    attachment,
                } => {
                    commands::issue::create(
                        &ctx.client,
                        &title,
                        &team_id,
                        description.as_deref(),
                        priority,
                        assignee_id.as_deref(),
                        project_id.as_deref(),
                        label_ids.as_deref(),
                        labels.as_deref(),
                        parent_id.as_deref(),
                        attachment.as_deref(),
                    )
                    .await?;
                }
                IssueCommand::Edit {
                    id,
                    title,
                    description,
                    priority,
                    assignee_id,
                    state,
                    project_id,
                    label_ids,
                    labels,
                    remove_labels,
                    parent_id,
                    attachment,
                } => {
                    commands::issue::edit(
                        &ctx.client,
                        &id,
                        title,
                        description,
                        priority,
                        assignee_id,
                        state,
                        project_id,
                        label_ids,
                        labels,
                        remove_labels,
                        parent_id,
                        attachment,
                    )
                    .await?;
                }
                IssueCommand::Search {
                    query,
                    project_id,
                    team_id,
                    assignee_id,
                    status,
                    limit,
                } => {
                    let q = query.as_deref().unwrap_or("");
                    commands::issue::search(
                        &ctx.client,
                        q,
                        project_id.as_deref(),
                        team_id.as_deref(),
                        assignee_id.as_deref(),
                        status.as_deref(),
                        limit,
                    )
                    .await?;
                }
                IssueCommand::State { id, name, list } => {
                    commands::issue::state(&ctx.client, &id, name.as_deref(), list).await?;
                }
                IssueCommand::Attachments { id } => {
                    commands::issue::attachments(&ctx.client, &id).await?;
                }
            }
        }

        Commands::Comment(cmd) => {
            let ctx = ensure_auth(ws_flag)?;
            match cmd {
                CommentCommand::View { id, show_ids } => {
                    commands::comment::view(&ctx.client, &id, show_ids).await?;
                }
                CommentCommand::Add {
                    id,
                    body,
                    attachment,
                } => {
                    commands::comment::add(&ctx.client, &id, &body, attachment.as_deref()).await?;
                }
                CommentCommand::Edit {
                    id,
                    body,
                    attachment,
                } => {
                    commands::comment::edit(&ctx.client, &id, &body, attachment.as_deref()).await?;
                }
            }
        }

        Commands::Project(cmd) => {
            let ctx = ensure_auth(ws_flag)?;
            match cmd {
                ProjectCommand::List {
                    include_archived,
                    limit,
                } => {
                    commands::project::list(&ctx.client, include_archived, limit).await?;
                }
                ProjectCommand::View { id } => {
                    commands::project::view(&ctx.client, &id).await?;
                }
                ProjectCommand::Create {
                    name,
                    team_ids,
                    description,
                } => {
                    commands::project::create(
                        &ctx.client,
                        &name,
                        &team_ids,
                        description.as_deref(),
                    )
                    .await?;
                }
                ProjectCommand::Edit {
                    id,
                    name,
                    description,
                    state,
                } => {
                    commands::project::edit(
                        &ctx.client,
                        &id,
                        name.as_deref(),
                        description.as_deref(),
                        state.as_deref(),
                    )
                    .await?;
                }
                ProjectCommand::Update(sub) => match sub {
                    ProjectUpdateCommand::List { project_id } => {
                        commands::project::update_list(&ctx.client, &project_id).await?;
                    }
                    ProjectUpdateCommand::Add {
                        project_id,
                        body,
                        health,
                    } => {
                        commands::project::update_add(
                            &ctx.client,
                            &project_id,
                            &body,
                            health.as_deref(),
                        )
                        .await?;
                    }
                    ProjectUpdateCommand::Edit { id, body, health } => {
                        commands::project::update_edit(&ctx.client, &id, &body, health.as_deref())
                            .await?;
                    }
                    ProjectUpdateCommand::Delete { id } => {
                        commands::project::update_delete(&ctx.client, &id).await?;
                    }
                },
            }
        }

        Commands::Team(cmd) => {
            let ctx = ensure_auth(ws_flag)?;
            match cmd {
                TeamCommand::List => {
                    commands::team::list(&ctx.client).await?;
                }
            }
        }

        Commands::User(cmd) => {
            let ctx = ensure_auth(ws_flag)?;
            match cmd {
                UserCommand::List => {
                    commands::user::list(&ctx.client).await?;
                }
            }
        }

        Commands::Label(cmd) => {
            let ctx = ensure_auth(ws_flag)?;
            match cmd {
                LabelCommand::List { team_id } => {
                    commands::label::list(&ctx.client, team_id.as_deref()).await?;
                }
                LabelCommand::Create {
                    name,
                    team_id,
                    color,
                    description,
                    parent_id,
                } => {
                    commands::label::create(
                        &ctx.client,
                        &name,
                        &team_id,
                        color.as_deref(),
                        description.as_deref(),
                        parent_id.as_deref(),
                    )
                    .await?;
                }
            }
        }

        Commands::Changelog => {
            let ctx = ensure_auth(ws_flag)?;
            commands::changelog::run(&ctx.client).await?;
        }
    }

    Ok(())
}
