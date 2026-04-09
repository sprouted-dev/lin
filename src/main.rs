mod api;
mod auth;
mod cli;
mod commands;
mod config;
mod date;
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
    json: bool,
}

fn ensure_auth(cli_workspace: Option<&str>, json: bool, verbose: bool) -> Result<CommandContext> {
    let ws = workspace::resolve_workspace(cli_workspace);
    let token = auth::get_token(&ws)?;
    let client = LinearClient::new(&token).with_verbose(verbose);
    Ok(CommandContext {
        client,
        workspace: ws,
        json,
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
    let json = cli.json;
    let verbose = cli.verbose;

    match cli.command {
        Commands::Login {
            token,
            name,
            keyring,
        } => {
            let workspace = ws_flag.unwrap_or(&name);
            commands::login::run(&token, workspace, keyring, verbose).await?;
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
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                IssueCommand::View { id } => {
                    commands::issue::view(&ctx.client, &id, ctx.json).await?;
                }
                IssueCommand::Create {
                    title,
                    team,
                    description,
                    priority,
                    assignee,
                    project,
                    label_ids,
                    labels,
                    parent,
                    cycle,
                    attachment,
                } => {
                    commands::issue::create(
                        &ctx.client,
                        &title,
                        &team,
                        description.as_deref(),
                        priority,
                        assignee.as_deref(),
                        project.as_deref(),
                        label_ids.as_deref(),
                        labels.as_deref(),
                        parent.as_deref(),
                        cycle.as_deref(),
                        attachment.as_deref(),
                    )
                    .await?;
                }
                IssueCommand::Edit {
                    id,
                    title,
                    description,
                    priority,
                    assignee,
                    state,
                    project,
                    label_ids,
                    labels,
                    remove_labels,
                    parent,
                    cycle,
                    attachment,
                    comment,
                } => {
                    commands::issue::edit(
                        &ctx.client,
                        &id,
                        title,
                        description,
                        priority,
                        assignee,
                        state,
                        project,
                        label_ids,
                        labels,
                        remove_labels,
                        parent,
                        cycle,
                        attachment,
                    )
                    .await?;
                    if let Some(body) = comment {
                        commands::comment::add(&ctx.client, &id, &body, None).await?;
                    }
                }
                IssueCommand::Search {
                    query,
                    project,
                    team,
                    assignee,
                    status,
                    limit,
                } => {
                    commands::issue::search(
                        &ctx.client,
                        &query,
                        project.as_deref(),
                        team.as_deref(),
                        assignee.as_deref(),
                        status.as_deref(),
                        limit,
                        ctx.json,
                    )
                    .await?;
                }
                IssueCommand::List {
                    team,
                    assignee,
                    creator,
                    status,
                    project,
                    priority,
                    labels,
                    cycle,
                    updated_since,
                    updated_before,
                    created_since,
                    created_before,
                    completed_since,
                    completed_before,
                    due_after,
                    due_before,
                    cancelled_since,
                    estimate,
                    estimate_gte,
                    estimate_lte,
                    parent,
                    no_parent,
                    has_children,
                    subscriber,
                    title,
                    limit,
                } => {
                    let date_filters = commands::issue::DateFilters {
                        updated_since,
                        updated_before,
                        created_since,
                        created_before,
                        completed_since,
                        completed_before,
                        due_after,
                        due_before,
                        cancelled_since,
                    };
                    let convenience_filters = commands::issue::ConvenienceFilters {
                        estimate,
                        estimate_gte,
                        estimate_lte,
                        parent,
                        no_parent,
                        has_children,
                        subscriber,
                        title,
                    };
                    commands::issue::list(
                        &ctx.client,
                        team.as_deref(),
                        assignee.as_deref(),
                        creator.as_deref(),
                        status.as_deref(),
                        project.as_deref(),
                        priority,
                        labels.as_deref(),
                        cycle.as_deref(),
                        date_filters,
                        convenience_filters,
                        limit,
                        ctx.json,
                    )
                    .await?;
                }
                IssueCommand::Me { status, limit } => {
                    commands::issue::me(&ctx.client, status.as_deref(), limit, ctx.json).await?;
                }
                IssueCommand::State { id, name, list } => {
                    commands::issue::state(&ctx.client, &id, name.as_deref(), list, ctx.json)
                        .await?;
                }
                IssueCommand::Attachments(sub) => match sub {
                    AttachmentCommand::List { id } => {
                        commands::issue::attachments(&ctx.client, &id, ctx.json).await?;
                    }
                    AttachmentCommand::Add { id, file, title } => {
                        commands::issue::attachment_add(&ctx.client, &id, &file, title.as_deref())
                            .await?;
                    }
                    AttachmentCommand::Download {
                        id,
                        output,
                        attachment_id,
                    } => {
                        commands::issue::attachment_download(
                            &ctx.client,
                            &id,
                            &output,
                            attachment_id.as_deref(),
                        )
                        .await?;
                    }
                },
                IssueCommand::Comment {
                    id,
                    body,
                    attachment,
                } => {
                    commands::comment::add(&ctx.client, &id, &body, attachment.as_deref()).await?;
                }
            }
        }

        Commands::Comment(cmd) => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                CommentCommand::View { id, show_ids } => {
                    commands::comment::view(&ctx.client, &id, show_ids, ctx.json).await?;
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
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                ProjectCommand::List {
                    include_archived,
                    limit,
                } => {
                    commands::project::list(&ctx.client, include_archived, limit, ctx.json).await?;
                }
                ProjectCommand::View { id, content } => {
                    commands::project::view(&ctx.client, &id, content, ctx.json).await?;
                }
                ProjectCommand::Create {
                    name,
                    teams,
                    description,
                } => {
                    commands::project::create(&ctx.client, &name, &teams, description.as_deref())
                        .await?;
                }
                ProjectCommand::Edit {
                    id,
                    name,
                    description,
                    content,
                    state,
                } => {
                    commands::project::edit(
                        &ctx.client,
                        &id,
                        name.as_deref(),
                        description.as_deref(),
                        content.as_deref(),
                        state.as_deref(),
                    )
                    .await?;
                }
                ProjectCommand::Update(sub) => match sub {
                    ProjectUpdateCommand::List { project } => {
                        commands::project::update_list(&ctx.client, &project, ctx.json).await?;
                    }
                    ProjectUpdateCommand::Add {
                        project,
                        body,
                        health,
                    } => {
                        commands::project::update_add(
                            &ctx.client,
                            &project,
                            &body,
                            health.as_deref(),
                        )
                        .await?;
                    }
                    ProjectUpdateCommand::Edit { id, body, health } => {
                        commands::project::update_edit(&ctx.client, &id, &body, health.as_deref())
                            .await?;
                    }
                    ProjectUpdateCommand::Show { id } => {
                        commands::project::update_show(&ctx.client, &id, ctx.json).await?;
                    }
                    ProjectUpdateCommand::Delete { id } => {
                        commands::project::update_delete(&ctx.client, &id).await?;
                    }
                },
            }
        }

        Commands::Team(cmd) => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                TeamCommand::List => {
                    commands::team::list(&ctx.client, ctx.json).await?;
                }
            }
        }

        Commands::User(cmd) => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                UserCommand::List => {
                    commands::user::list(&ctx.client, ctx.json).await?;
                }
            }
        }

        Commands::Label(cmd) => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                LabelCommand::List { team } => {
                    commands::label::list(&ctx.client, team.as_deref(), ctx.json).await?;
                }
                LabelCommand::Create {
                    name,
                    team,
                    color,
                    description,
                    parent_id,
                } => {
                    commands::label::create(
                        &ctx.client,
                        &name,
                        &team,
                        color.as_deref(),
                        description.as_deref(),
                        parent_id.as_deref(),
                    )
                    .await?;
                }
            }
        }

        Commands::Cycle(cmd) => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                CycleCommand::List { team, limit } => {
                    commands::cycle::list(&ctx.client, &team, limit, ctx.json).await?;
                }
                CycleCommand::Active { team } => {
                    commands::cycle::active(&ctx.client, &team, ctx.json).await?;
                }
                CycleCommand::Create {
                    team,
                    starts,
                    ends,
                    duration,
                    name,
                    description,
                } => {
                    commands::cycle::create(
                        &ctx.client,
                        &team,
                        &starts,
                        ends.as_deref(),
                        duration.as_deref(),
                        name.as_deref(),
                        description.as_deref(),
                        ctx.json,
                    )
                    .await?;
                }
                CycleCommand::Edit {
                    id,
                    team,
                    name,
                    description,
                    starts,
                    ends,
                } => {
                    commands::cycle::edit(
                        &ctx.client,
                        &id,
                        &team,
                        name,
                        description,
                        starts.as_deref(),
                        ends.as_deref(),
                        ctx.json,
                    )
                    .await?;
                }
                CycleCommand::Show { id, team } => {
                    commands::cycle::show(&ctx.client, &id, &team, ctx.json).await?;
                }
            }
        }

        Commands::Initiative(cmd) => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            match cmd {
                InitiativeCommand::List { limit } => {
                    commands::initiative::list(&ctx.client, limit, ctx.json).await?;
                }
                InitiativeCommand::View { id } => {
                    commands::initiative::view(&ctx.client, &id, ctx.json).await?;
                }
            }
        }

        Commands::Download { url, output } => {
            let ws = workspace::resolve_workspace(ws_flag);
            let token = auth::get_token(&ws)?;
            commands::download::run(&token, &url, &output).await?;
        }

        Commands::Changelog => {
            let ctx = ensure_auth(ws_flag, json, verbose)?;
            commands::changelog::run(&ctx.client, ctx.json).await?;
        }
    }

    Ok(())
}
