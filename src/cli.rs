use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lin", about = "A fast CLI for Linear", version)]
pub struct Cli {
    /// Workspace to use (overrides config and .linear-workspace file)
    #[arg(long, short, global = true)]
    pub workspace: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Linear using an API token
    Login {
        /// Your Linear API token
        token: String,

        /// Workspace name to associate with this token
        #[arg(long, default_value = "default")]
        name: String,
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

    /// View changelog
    Changelog,
}

#[derive(Subcommand)]
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

        /// Filter by status name
        #[arg(long)]
        status: Option<String>,

        /// Filter by project (name or UUID)
        #[arg(long)]
        project: Option<String>,

        /// Filter by priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,

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
    /// List attachments on an issue
    Attachments {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
    },
    /// Add a comment to an issue (alias for `lin comment add`)
    Comment {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
        /// Comment body
        body: String,

        /// Attach a file (uploads and embeds markdown link in body)
        #[arg(long)]
        attachment: Option<String>,
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

        /// New description
        #[arg(long)]
        description: Option<String>,

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
            }) => {
                assert_eq!(id, "ENG-123");
                assert_eq!(body, "Hello world");
                assert!(attachment.is_none());
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
    fn top_level_comment_add_still_works() {
        let cli = parse(&["lin", "comment", "add", "ENG-1", "top level comment"]);
        match cli.command {
            Commands::Comment(CommentCommand::Add { id, body, .. }) => {
                assert_eq!(id, "ENG-1");
                assert_eq!(body, "top level comment");
            }
            _ => panic!("expected Comment Add"),
        }
    }
}
