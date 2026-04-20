use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lin", about = "A fast CLI for Linear", version)]
pub struct Cli {
    /// Workspace to use (overrides config and .linear-workspace file)
    #[arg(long, short, global = true)]
    pub workspace: Option<String>,

    /// Output raw JSON from the Linear API (read commands only)
    #[arg(long, global = true)]
    pub json: bool,

    /// Show response body in error messages for debugging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Authenticate with Linear using an API token
    Login {
        /// Your Linear API token
        token: String,

        /// Workspace name to associate with this token
        #[arg(long, default_value = "default")]
        name: String,

        /// Store token in OS keychain instead of file
        #[arg(long)]
        keyring: bool,
    },

    /// Manage issues
    #[command(subcommand)]
    Issue(IssueCommand),

    /// Manage workspaces
    #[command(subcommand)]
    Workspace(WorkspaceCommand),

    /// Manage comments
    #[command(subcommand)]
    Comment(CommentCommand),

    /// Manage projects
    #[command(subcommand)]
    Project(ProjectCommand),

    /// List teams
    #[command(subcommand)]
    Team(TeamCommand),

    /// List users
    #[command(subcommand)]
    User(UserCommand),

    /// Manage labels
    #[command(subcommand)]
    Label(LabelCommand),

    /// Manage cycles
    #[command(subcommand)]
    Cycle(CycleCommand),

    /// Manage initiatives
    #[command(subcommand)]
    Initiative(InitiativeCommand),

    /// Download a file from a Linear upload URL
    Download {
        /// Linear upload URL (https://uploads.linear.app/...)
        url: String,

        /// Output directory (defaults to current directory)
        #[arg(long, short, default_value = ".")]
        output: String,
    },

    /// View changelog
    Changelog,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum IssueCommand {
    /// View issue details
    View {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
    },
    /// Create a new issue
    Create {
        /// Issue title
        title: String,

        /// Team name, key, or UUID
        #[arg(long)]
        team: String,

        /// Issue description
        #[arg(long)]
        description: Option<String>,

        /// Priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,

        /// Assignee (name, email, "me", or UUID)
        #[arg(long)]
        assignee: Option<String>,

        /// Project name or UUID
        #[arg(long)]
        project: Option<String>,

        /// Label IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        label_ids: Option<Vec<String>>,

        /// Label names (repeatable, resolved to IDs)
        #[arg(long = "label", value_delimiter = ',')]
        labels: Option<Vec<String>>,

        /// Parent issue ID or identifier (e.g., APP-123)
        #[arg(long)]
        parent: Option<String>,

        /// Cycle (name, number, or "current"; requires --team)
        #[arg(long)]
        cycle: Option<String>,

        /// Attach a file to the created issue
        #[arg(long)]
        attachment: Option<String>,
    },
    /// Edit an existing issue
    Edit {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New description
        #[arg(long)]
        description: Option<String>,

        /// Priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,

        /// Assignee (name, email, "me", or UUID)
        #[arg(long)]
        assignee: Option<String>,

        /// Workflow state name
        #[arg(long)]
        state: Option<String>,

        /// Project name or UUID
        #[arg(long)]
        project: Option<String>,

        /// Label IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        label_ids: Option<Vec<String>>,

        /// Label names to add (repeatable, resolved to IDs)
        #[arg(long = "label", value_delimiter = ',')]
        labels: Option<Vec<String>>,

        /// Label names to remove (repeatable, resolved to IDs)
        #[arg(long = "remove-label", value_delimiter = ',')]
        remove_labels: Option<Vec<String>>,

        /// Parent issue ID or identifier (e.g., APP-123)
        #[arg(long)]
        parent: Option<String>,

        /// Cycle (name, number, or "current"; requires --team)
        #[arg(long)]
        cycle: Option<String>,

        /// Attach a file to the issue
        #[arg(long)]
        attachment: Option<String>,

        /// Add a comment to the issue after editing
        #[arg(long)]
        comment: Option<String>,
    },
    /// Search issues (requires a search term)
    Search {
        /// Search query
        query: String,

        /// Filter by project (name or UUID)
        #[arg(long)]
        project: Option<String>,

        /// Filter by team (name, key, or UUID)
        #[arg(long)]
        team: Option<String>,

        /// Filter by assignee (name, email, "me", or UUID)
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by status name
        #[arg(long)]
        status: Option<String>,

        /// Max results
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// List issues with filters (no text search)
    List {
        /// Filter by team (name, key, or UUID)
        #[arg(long)]
        team: Option<String>,

        /// Filter by assignee (name, email, "me", or UUID)
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by creator (name, email, "me", or UUID)
        #[arg(long)]
        creator: Option<String>,

        /// Filter by status name
        #[arg(long)]
        status: Option<String>,

        /// Filter by project (name or UUID)
        #[arg(long)]
        project: Option<String>,

        /// Filter by priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,

        /// Filter by label name (repeatable for AND logic)
        #[arg(long = "label", action = clap::ArgAction::Append)]
        labels: Option<Vec<String>>,

        /// Filter by cycle (name, number, or "current" for active cycle; requires --team)
        #[arg(long)]
        cycle: Option<String>,

        /// Filter issues updated on or after date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        updated_since: Option<String>,

        /// Filter issues updated before date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        updated_before: Option<String>,

        /// Filter issues created on or after date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        created_since: Option<String>,

        /// Filter issues created before date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        created_before: Option<String>,

        /// Filter issues completed on or after date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        completed_since: Option<String>,

        /// Filter issues completed before date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        completed_before: Option<String>,

        /// Filter issues due after date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        due_after: Option<String>,

        /// Filter issues due before date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        due_before: Option<String>,

        /// Filter issues cancelled on or after date (ISO 8601: 2026-03-24 or relative: 3d, 1w, 2h)
        #[arg(long)]
        cancelled_since: Option<String>,

        /// Filter by exact estimate (story points)
        #[arg(long)]
        estimate: Option<f64>,

        /// Filter by minimum estimate (story points)
        #[arg(long)]
        estimate_gte: Option<f64>,

        /// Filter by maximum estimate (story points)
        #[arg(long)]
        estimate_lte: Option<f64>,

        /// Filter by parent issue (identifier like APP-123 or UUID)
        #[arg(long)]
        parent: Option<String>,

        /// Show only top-level issues (no parent)
        #[arg(long)]
        no_parent: bool,

        /// Show only issues that have sub-issues
        #[arg(long)]
        has_children: bool,

        /// Filter by subscriber (name, email, "me", or UUID)
        #[arg(long)]
        subscriber: Option<String>,

        /// Filter by title (substring match)
        #[arg(long)]
        title: Option<String>,

        /// Max results
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// List issues assigned to you
    Me {
        /// Filter by status name
        #[arg(long)]
        status: Option<String>,

        /// Max results
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// View or change issue state
    State {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,

        /// New state name (omit to show current state)
        name: Option<String>,

        /// List all available states for the issue's team
        #[arg(long)]
        list: bool,
    },
    /// Manage issue attachments
    #[command(subcommand)]
    Attachments(AttachmentCommand),
    /// Add a comment to an issue (alias for `lin comment add`)
    Comment {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
        /// Comment body
        body: String,

        /// Attach a file (uploads and embeds markdown link in body)
        #[arg(long)]
        attachment: Option<String>,

        /// Reply under an existing comment (parent comment UUID)
        #[arg(long)]
        parent: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AttachmentCommand {
    /// List attachments on an issue
    List {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
    },
    /// Add a file attachment to an issue
    Add {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
        /// Path to the file to attach
        file: String,
        /// Custom title for the attachment (defaults to filename)
        #[arg(long)]
        title: Option<String>,
    },
    /// Download attachments from an issue
    Download {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
        /// Output directory (defaults to current directory)
        #[arg(long, short, default_value = ".")]
        output: String,
        /// Download only the attachment matching this ID prefix
        #[arg(long)]
        attachment_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Show current workspace
    Current,
    /// List all configured workspaces
    List,
    /// Set workspace for the current directory
    Set {
        /// Workspace name
        name: String,

        /// Set as global default workspace instead of directory-specific
        #[arg(long)]
        global: bool,
    },
}

#[derive(Subcommand)]
pub enum CommentCommand {
    /// View comments on an issue
    View {
        /// Issue identifier
        id: String,

        /// Show comment IDs
        #[arg(long)]
        show_ids: bool,
    },
    /// Add a comment to an issue
    Add {
        /// Issue identifier
        id: String,
        /// Comment body
        body: String,

        /// Attach a file (uploads and embeds markdown link in body)
        #[arg(long)]
        attachment: Option<String>,

        /// Reply under an existing comment (parent comment UUID)
        #[arg(long)]
        parent: Option<String>,
    },
    /// Reply to an existing comment (looks up the issue automatically)
    Reply {
        /// Parent comment UUID (get via `lin comment view --show-ids`)
        id: String,
        /// Reply body
        body: String,

        /// Attach a file (uploads and embeds markdown link in body)
        #[arg(long)]
        attachment: Option<String>,
    },
    /// Edit a comment
    Edit {
        /// Comment ID
        id: String,
        /// New comment body
        body: String,

        /// Attach a file (uploads and embeds markdown link in body)
        #[arg(long)]
        attachment: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommand {
    /// List projects
    List {
        /// Include archived projects
        #[arg(long)]
        include_archived: bool,

        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// View project details
    View {
        /// Project name or UUID
        id: String,

        /// Show full project description
        #[arg(long)]
        content: bool,
    },
    /// Create a new project
    Create {
        /// Project name
        name: String,

        /// Team names, keys, or UUIDs (comma-separated, at least one required)
        #[arg(long, value_delimiter = ',')]
        teams: Vec<String>,

        /// Project description
        #[arg(long)]
        description: Option<String>,
    },
    /// Edit a project
    Edit {
        /// Project name or UUID
        id: String,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// New short summary
        #[arg(long)]
        description: Option<String>,

        /// New long-form description/overview
        #[arg(long)]
        content: Option<String>,

        /// New state (e.g., planned, started, paused, completed, canceled)
        #[arg(long)]
        state: Option<String>,
    },
    /// Manage project updates
    #[command(subcommand)]
    Update(ProjectUpdateCommand),
}

#[derive(Subcommand)]
pub enum ProjectUpdateCommand {
    /// List project updates
    List {
        /// Project name or UUID
        project: String,
    },
    /// Add a project update
    Add {
        /// Project name or UUID
        project: String,
        /// Update body
        body: String,

        /// Health status (onTrack, atRisk, offTrack)
        #[arg(long)]
        health: Option<String>,
    },
    /// Edit a project update
    Edit {
        /// Update ID
        id: String,
        /// New body
        #[arg(long)]
        body: String,

        /// Health status (onTrack, atRisk, offTrack)
        #[arg(long)]
        health: Option<String>,
    },
    /// Show a project update
    Show {
        /// Update ID (or prefix)
        id: String,
    },
    /// Delete a project update
    Delete {
        /// Update ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum TeamCommand {
    /// List teams
    List,
}

#[derive(Subcommand)]
pub enum UserCommand {
    /// List users
    List,
}

#[derive(Subcommand)]
pub enum LabelCommand {
    /// List labels
    List {
        /// Filter by team (name, key, or UUID)
        #[arg(long)]
        team: Option<String>,
    },
    /// Create a label
    Create {
        /// Label name
        name: String,

        /// Team name, key, or UUID
        #[arg(long)]
        team: String,

        /// Label color (hex, e.g., #ff0000)
        #[arg(long)]
        color: Option<String>,

        /// Label description
        #[arg(long)]
        description: Option<String>,

        /// Parent label ID
        #[arg(long)]
        parent_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CycleCommand {
    /// List cycles for a team
    List {
        /// Team name, key, or UUID
        #[arg(long)]
        team: String,

        /// Max results
        #[arg(long, default_value = "10")]
        limit: i32,
    },
    /// Show the currently active cycle for a team
    Active {
        /// Team name, key, or UUID
        #[arg(long)]
        team: String,
    },
    /// Create a new cycle
    Create {
        /// Team name, key, or UUID
        #[arg(long)]
        team: String,

        /// Start date (ISO 8601, e.g. 2026-04-07)
        #[arg(long)]
        starts: String,

        /// End date (ISO 8601, e.g. 2026-04-14)
        #[arg(long, conflicts_with = "duration")]
        ends: Option<String>,

        /// Duration from start (e.g. 1w, 2w, 10d)
        #[arg(long, conflicts_with = "ends")]
        duration: Option<String>,

        /// Cycle name
        #[arg(long)]
        name: Option<String>,

        /// Cycle description
        #[arg(long)]
        description: Option<String>,
    },
    /// Edit a cycle's name or description
    Edit {
        /// Cycle name, number, or "current"
        id: String,

        /// Team name, key, or UUID
        #[arg(long)]
        team: String,

        /// New cycle name
        #[arg(long)]
        name: Option<String>,

        /// New cycle description
        #[arg(long)]
        description: Option<String>,

        /// New start date (ISO 8601, e.g. 2026-04-07)
        #[arg(long)]
        starts: Option<String>,

        /// New end date (ISO 8601, e.g. 2026-04-14)
        #[arg(long)]
        ends: Option<String>,
    },
    /// View cycle details including issues
    Show {
        /// Cycle name, number, or "current"
        id: String,

        /// Team name, key, or UUID
        #[arg(long)]
        team: String,
    },
}

#[derive(Subcommand)]
pub enum InitiativeCommand {
    /// List initiatives
    List {
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// View initiative details
    View {
        /// Initiative ID
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("failed to parse")
    }

    #[test]
    fn issue_comment_parses() {
        let cli = parse(&["lin", "issue", "comment", "ENG-123", "Hello world"]);
        match cli.command {
            Commands::Issue(IssueCommand::Comment {
                id,
                body,
                attachment,
                parent,
            }) => {
                assert_eq!(id, "ENG-123");
                assert_eq!(body, "Hello world");
                assert!(attachment.is_none());
                assert!(parent.is_none());
            }
            _ => panic!("expected Issue Comment"),
        }
    }

    #[test]
    fn issue_comment_with_attachment() {
        let cli = parse(&[
            "lin",
            "issue",
            "comment",
            "ENG-1",
            "body text",
            "--attachment",
            "file.png",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Comment { attachment, .. }) => {
                assert_eq!(attachment.as_deref(), Some("file.png"));
            }
            _ => panic!("expected Issue Comment"),
        }
    }

    #[test]
    fn issue_comment_with_parent() {
        let cli = parse(&[
            "lin",
            "issue",
            "comment",
            "ENG-1",
            "nested reply",
            "--parent",
            "abc123-parent",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Comment { parent, .. }) => {
                assert_eq!(parent.as_deref(), Some("abc123-parent"));
            }
            _ => panic!("expected Issue Comment"),
        }
    }

    #[test]
    fn issue_edit_with_comment() {
        let cli = parse(&[
            "lin",
            "issue",
            "edit",
            "ENG-5",
            "--title",
            "New title",
            "--comment",
            "Looks good",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Edit {
                id, title, comment, ..
            }) => {
                assert_eq!(id, "ENG-5");
                assert_eq!(title.as_deref(), Some("New title"));
                assert_eq!(comment.as_deref(), Some("Looks good"));
            }
            _ => panic!("expected Issue Edit"),
        }
    }

    #[test]
    fn issue_edit_without_comment() {
        let cli = parse(&["lin", "issue", "edit", "ENG-5", "--title", "New title"]);
        match cli.command {
            Commands::Issue(IssueCommand::Edit { comment, .. }) => {
                assert!(comment.is_none());
            }
            _ => panic!("expected Issue Edit"),
        }
    }

    #[test]
    fn issue_attachments_list_parses() {
        let cli = parse(&["lin", "issue", "attachments", "list", "ENG-10"]);
        match cli.command {
            Commands::Issue(IssueCommand::Attachments(AttachmentCommand::List { id })) => {
                assert_eq!(id, "ENG-10");
            }
            _ => panic!("expected Issue Attachments List"),
        }
    }

    #[test]
    fn issue_attachments_add_parses() {
        let cli = parse(&[
            "lin",
            "issue",
            "attachments",
            "add",
            "ENG-10",
            "screenshot.png",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Attachments(AttachmentCommand::Add {
                id,
                file,
                title,
            })) => {
                assert_eq!(id, "ENG-10");
                assert_eq!(file, "screenshot.png");
                assert!(title.is_none());
            }
            _ => panic!("expected Issue Attachments Add"),
        }
    }

    #[test]
    fn issue_attachments_add_with_title() {
        let cli = parse(&[
            "lin",
            "issue",
            "attachments",
            "add",
            "ENG-10",
            "screenshot.png",
            "--title",
            "My Screenshot",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Attachments(AttachmentCommand::Add {
                title, ..
            })) => {
                assert_eq!(title.as_deref(), Some("My Screenshot"));
            }
            _ => panic!("expected Issue Attachments Add"),
        }
    }

    #[test]
    fn top_level_comment_add_still_works() {
        let cli = parse(&["lin", "comment", "add", "ENG-1", "top level comment"]);
        match cli.command {
            Commands::Comment(CommentCommand::Add {
                id, body, parent, ..
            }) => {
                assert_eq!(id, "ENG-1");
                assert_eq!(body, "top level comment");
                assert!(parent.is_none());
            }
            _ => panic!("expected Comment Add"),
        }
    }

    #[test]
    fn comment_add_with_parent_parses() {
        let cli = parse(&[
            "lin",
            "comment",
            "add",
            "ENG-1",
            "nested reply",
            "--parent",
            "parent-uuid",
        ]);
        match cli.command {
            Commands::Comment(CommentCommand::Add {
                id, body, parent, ..
            }) => {
                assert_eq!(id, "ENG-1");
                assert_eq!(body, "nested reply");
                assert_eq!(parent.as_deref(), Some("parent-uuid"));
            }
            _ => panic!("expected Comment Add"),
        }
    }

    #[test]
    fn comment_add_parent_and_attachment() {
        let cli = parse(&[
            "lin",
            "comment",
            "add",
            "ENG-1",
            "reply with file",
            "--parent",
            "parent-uuid",
            "--attachment",
            "shot.png",
        ]);
        match cli.command {
            Commands::Comment(CommentCommand::Add {
                parent, attachment, ..
            }) => {
                assert_eq!(parent.as_deref(), Some("parent-uuid"));
                assert_eq!(attachment.as_deref(), Some("shot.png"));
            }
            _ => panic!("expected Comment Add"),
        }
    }

    #[test]
    fn comment_reply_parses() {
        let cli = parse(&["lin", "comment", "reply", "parent-uuid", "reply body"]);
        match cli.command {
            Commands::Comment(CommentCommand::Reply {
                id,
                body,
                attachment,
            }) => {
                assert_eq!(id, "parent-uuid");
                assert_eq!(body, "reply body");
                assert!(attachment.is_none());
            }
            _ => panic!("expected Comment Reply"),
        }
    }

    #[test]
    fn comment_reply_with_attachment() {
        let cli = parse(&[
            "lin",
            "comment",
            "reply",
            "parent-uuid",
            "reply body",
            "--attachment",
            "shot.png",
        ]);
        match cli.command {
            Commands::Comment(CommentCommand::Reply { attachment, .. }) => {
                assert_eq!(attachment.as_deref(), Some("shot.png"));
            }
            _ => panic!("expected Comment Reply"),
        }
    }

    #[test]
    fn issue_attachments_download_parses() {
        let cli = parse(&["lin", "issue", "attachments", "download", "ENG-10"]);
        match cli.command {
            Commands::Issue(IssueCommand::Attachments(AttachmentCommand::Download {
                id,
                output,
                attachment_id,
            })) => {
                assert_eq!(id, "ENG-10");
                assert_eq!(output, ".");
                assert!(attachment_id.is_none());
            }
            _ => panic!("expected Issue Attachments Download"),
        }
    }

    #[test]
    fn issue_attachments_download_with_options() {
        let cli = parse(&[
            "lin",
            "issue",
            "attachments",
            "download",
            "PLO-358",
            "--output",
            "/tmp/downloads",
            "--attachment-id",
            "f17bacc6",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Attachments(AttachmentCommand::Download {
                id,
                output,
                attachment_id,
            })) => {
                assert_eq!(id, "PLO-358");
                assert_eq!(output, "/tmp/downloads");
                assert_eq!(attachment_id.as_deref(), Some("f17bacc6"));
            }
            _ => panic!("expected Issue Attachments Download"),
        }
    }

    #[test]
    fn issue_attachments_download_short_output() {
        let cli = parse(&[
            "lin",
            "issue",
            "attachments",
            "download",
            "ENG-1",
            "-o",
            "/tmp",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Attachments(AttachmentCommand::Download {
                output,
                ..
            })) => {
                assert_eq!(output, "/tmp");
            }
            _ => panic!("expected Issue Attachments Download"),
        }
    }

    #[test]
    fn issue_list_with_date_filters() {
        let cli = parse(&[
            "lin",
            "issue",
            "list",
            "--updated-since",
            "3d",
            "--updated-before",
            "2026-03-24",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                updated_since,
                updated_before,
                ..
            }) => {
                assert_eq!(updated_since.as_deref(), Some("3d"));
                assert_eq!(updated_before.as_deref(), Some("2026-03-24"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_all_date_filters() {
        let cli = parse(&[
            "lin",
            "issue",
            "list",
            "--updated-since",
            "1w",
            "--created-since",
            "2w",
            "--completed-since",
            "2026-01-01",
            "--due-before",
            "2026-12-31",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                updated_since,
                created_since,
                completed_since,
                due_before,
                ..
            }) => {
                assert_eq!(updated_since.as_deref(), Some("1w"));
                assert_eq!(created_since.as_deref(), Some("2w"));
                assert_eq!(completed_since.as_deref(), Some("2026-01-01"));
                assert_eq!(due_before.as_deref(), Some("2026-12-31"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_combines_with_existing_filters() {
        let cli = parse(&[
            "lin",
            "issue",
            "list",
            "--team",
            "engineering",
            "--status",
            "In Progress",
            "--updated-since",
            "3d",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                team,
                status,
                updated_since,
                ..
            }) => {
                assert_eq!(team.as_deref(), Some("engineering"));
                assert_eq!(status.as_deref(), Some("In Progress"));
                assert_eq!(updated_since.as_deref(), Some("3d"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_single_label() {
        let cli = parse(&["lin", "issue", "list", "--label", "bug"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { labels, .. }) => {
                assert_eq!(labels, Some(vec!["bug".to_string()]));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_multiple_labels() {
        let cli = parse(&[
            "lin", "issue", "list", "--label", "bug", "--label", "urgent",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List { labels, .. }) => {
                assert_eq!(labels, Some(vec!["bug".to_string(), "urgent".to_string()]));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_cycle() {
        let cli = parse(&[
            "lin", "issue", "list", "--team", "eng", "--cycle", "current",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List { team, cycle, .. }) => {
                assert_eq!(team.as_deref(), Some("eng"));
                assert_eq!(cycle.as_deref(), Some("current"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_cycle_number() {
        let cli = parse(&["lin", "issue", "list", "--team", "eng", "--cycle", "42"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { cycle, .. }) => {
                assert_eq!(cycle.as_deref(), Some("42"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_creator() {
        let cli = parse(&["lin", "issue", "list", "--creator", "me"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { creator, .. }) => {
                assert_eq!(creator.as_deref(), Some("me"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_creator_email() {
        let cli = parse(&["lin", "issue", "list", "--creator", "user@example.com"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { creator, .. }) => {
                assert_eq!(creator.as_deref(), Some("user@example.com"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_combines_all_new_filters() {
        let cli = parse(&[
            "lin",
            "issue",
            "list",
            "--team",
            "eng",
            "--label",
            "bug",
            "--label",
            "p0",
            "--cycle",
            "current",
            "--creator",
            "me",
            "--assignee",
            "alice",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                team,
                labels,
                cycle,
                creator,
                assignee,
                ..
            }) => {
                assert_eq!(team.as_deref(), Some("eng"));
                assert_eq!(labels, Some(vec!["bug".to_string(), "p0".to_string()]));
                assert_eq!(cycle.as_deref(), Some("current"));
                assert_eq!(creator.as_deref(), Some("me"));
                assert_eq!(assignee.as_deref(), Some("alice"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_cancelled_since() {
        let cli = parse(&["lin", "issue", "list", "--cancelled-since", "1w"]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                cancelled_since, ..
            }) => {
                assert_eq!(cancelled_since.as_deref(), Some("1w"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_estimate_exact() {
        let cli = parse(&["lin", "issue", "list", "--estimate", "3"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { estimate, .. }) => {
                assert_eq!(estimate, Some(3.0));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_estimate_range() {
        let cli = parse(&[
            "lin",
            "issue",
            "list",
            "--estimate-gte",
            "2",
            "--estimate-lte",
            "8",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                estimate_gte,
                estimate_lte,
                ..
            }) => {
                assert_eq!(estimate_gte, Some(2.0));
                assert_eq!(estimate_lte, Some(8.0));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_parent() {
        let cli = parse(&["lin", "issue", "list", "--parent", "APP-123"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { parent, .. }) => {
                assert_eq!(parent.as_deref(), Some("APP-123"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_no_parent() {
        let cli = parse(&["lin", "issue", "list", "--no-parent"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { no_parent, .. }) => {
                assert!(no_parent);
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_has_children() {
        let cli = parse(&["lin", "issue", "list", "--has-children"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { has_children, .. }) => {
                assert!(has_children);
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_subscriber() {
        let cli = parse(&["lin", "issue", "list", "--subscriber", "me"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { subscriber, .. }) => {
                assert_eq!(subscriber.as_deref(), Some("me"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_with_title() {
        let cli = parse(&["lin", "issue", "list", "--title", "authentication"]);
        match cli.command {
            Commands::Issue(IssueCommand::List { title, .. }) => {
                assert_eq!(title.as_deref(), Some("authentication"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn issue_list_combines_convenience_filters() {
        let cli = parse(&[
            "lin",
            "issue",
            "list",
            "--team",
            "eng",
            "--no-parent",
            "--has-children",
            "--estimate-gte",
            "5",
            "--title",
            "epic",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::List {
                team,
                no_parent,
                has_children,
                estimate_gte,
                title,
                ..
            }) => {
                assert_eq!(team.as_deref(), Some("eng"));
                assert!(no_parent);
                assert!(has_children);
                assert_eq!(estimate_gte, Some(5.0));
                assert_eq!(title.as_deref(), Some("epic"));
            }
            _ => panic!("expected Issue List"),
        }
    }

    #[test]
    fn project_view_without_content_flag() {
        let cli = parse(&["lin", "project", "view", "March 2026"]);
        match cli.command {
            Commands::Project(ProjectCommand::View { id, content }) => {
                assert_eq!(id, "March 2026");
                assert!(!content);
            }
            _ => panic!("expected Project View"),
        }
    }

    #[test]
    fn project_view_with_content_flag() {
        let cli = parse(&["lin", "project", "view", "March 2026", "--content"]);
        match cli.command {
            Commands::Project(ProjectCommand::View { id, content }) => {
                assert_eq!(id, "March 2026");
                assert!(content);
            }
            _ => panic!("expected Project View"),
        }
    }

    #[test]
    fn global_json_flag() {
        let cli = parse(&["lin", "--json", "project", "list"]);
        assert!(cli.json);
        match cli.command {
            Commands::Project(ProjectCommand::List { .. }) => {}
            _ => panic!("expected Project List"),
        }
    }

    #[test]
    fn json_flag_after_subcommand() {
        let cli = parse(&["lin", "issue", "list", "--json"]);
        assert!(cli.json);
    }

    #[test]
    fn no_json_flag_by_default() {
        let cli = parse(&["lin", "team", "list"]);
        assert!(!cli.json);
    }

    #[test]
    fn cycle_create_with_ends() {
        let cli = parse(&[
            "lin",
            "cycle",
            "create",
            "--team",
            "eng",
            "--starts",
            "2026-04-07",
            "--ends",
            "2026-04-14",
        ]);
        match cli.command {
            Commands::Cycle(CycleCommand::Create {
                team,
                starts,
                ends,
                duration,
                ..
            }) => {
                assert_eq!(team, "eng");
                assert_eq!(starts, "2026-04-07");
                assert_eq!(ends.as_deref(), Some("2026-04-14"));
                assert!(duration.is_none());
            }
            _ => panic!("expected Cycle Create"),
        }
    }

    #[test]
    fn cycle_create_with_duration() {
        let cli = parse(&[
            "lin",
            "cycle",
            "create",
            "--team",
            "eng",
            "--starts",
            "2026-04-07",
            "--duration",
            "1w",
        ]);
        match cli.command {
            Commands::Cycle(CycleCommand::Create { duration, ends, .. }) => {
                assert_eq!(duration.as_deref(), Some("1w"));
                assert!(ends.is_none());
            }
            _ => panic!("expected Cycle Create"),
        }
    }

    #[test]
    fn cycle_show_parses() {
        let cli = parse(&["lin", "cycle", "show", "current", "--team", "eng"]);
        match cli.command {
            Commands::Cycle(CycleCommand::Show { id, team }) => {
                assert_eq!(id, "current");
                assert_eq!(team, "eng");
            }
            _ => panic!("expected Cycle Show"),
        }
    }

    #[test]
    fn cycle_edit_parses() {
        let cli = parse(&[
            "lin",
            "cycle",
            "edit",
            "current",
            "--team",
            "eng",
            "--name",
            "Sprint 5",
            "--description",
            "Updated desc",
            "--starts",
            "2026-04-07",
            "--ends",
            "2026-04-14",
        ]);
        match cli.command {
            Commands::Cycle(CycleCommand::Edit {
                id,
                team,
                name,
                description,
                starts,
                ends,
            }) => {
                assert_eq!(id, "current");
                assert_eq!(team, "eng");
                assert_eq!(name.as_deref(), Some("Sprint 5"));
                assert_eq!(description.as_deref(), Some("Updated desc"));
                assert_eq!(starts.as_deref(), Some("2026-04-07"));
                assert_eq!(ends.as_deref(), Some("2026-04-14"));
            }
            _ => panic!("expected Cycle Edit"),
        }
    }

    #[test]
    fn issue_create_with_cycle() {
        let cli = parse(&[
            "lin",
            "issue",
            "create",
            "Test issue",
            "--team",
            "eng",
            "--cycle",
            "current",
        ]);
        match cli.command {
            Commands::Issue(IssueCommand::Create { cycle, .. }) => {
                assert_eq!(cycle.as_deref(), Some("current"));
            }
            _ => panic!("expected Issue Create"),
        }
    }

    #[test]
    fn issue_edit_with_cycle() {
        let cli = parse(&["lin", "issue", "edit", "ENG-5", "--cycle", "42"]);
        match cli.command {
            Commands::Issue(IssueCommand::Edit { cycle, .. }) => {
                assert_eq!(cycle.as_deref(), Some("42"));
            }
            _ => panic!("expected Issue Edit"),
        }
    }

    #[test]
    fn download_parses() {
        let cli = parse(&["lin", "download", "https://uploads.linear.app/abc/def/ghi"]);
        match cli.command {
            Commands::Download { url, output } => {
                assert_eq!(url, "https://uploads.linear.app/abc/def/ghi");
                assert_eq!(output, ".");
            }
            _ => panic!("expected Download"),
        }
    }

    #[test]
    fn download_with_output() {
        let cli = parse(&[
            "lin",
            "download",
            "https://uploads.linear.app/abc/def/ghi",
            "-o",
            "/tmp/downloads",
        ]);
        match cli.command {
            Commands::Download { url, output } => {
                assert_eq!(url, "https://uploads.linear.app/abc/def/ghi");
                assert_eq!(output, "/tmp/downloads");
            }
            _ => panic!("expected Download"),
        }
    }
}
