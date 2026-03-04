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

        /// Team ID (required)
        #[arg(long)]
        team_id: String,

        /// Issue description
        #[arg(long)]
        description: Option<String>,

        /// Priority (0=None, 1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,

        /// Assignee user ID
        #[arg(long)]
        assignee_id: Option<String>,

        /// Project ID
        #[arg(long)]
        project_id: Option<String>,

        /// Label IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        label_ids: Option<Vec<String>>,

        /// Label names (repeatable, resolved to IDs)
        #[arg(long = "label", value_delimiter = ',')]
        labels: Option<Vec<String>>,

        /// Parent issue ID or identifier (e.g., APP-123)
        #[arg(long)]
        parent_id: Option<String>,

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

        /// Assignee user ID
        #[arg(long)]
        assignee_id: Option<String>,

        /// Workflow state ID
        #[arg(long)]
        state: Option<String>,

        /// Project ID
        #[arg(long)]
        project_id: Option<String>,

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
        parent_id: Option<String>,

        /// Attach a file to the issue
        #[arg(long)]
        attachment: Option<String>,
    },
    /// Search issues
    Search {
        /// Search query
        query: Option<String>,

        /// Filter by project ID
        #[arg(long)]
        project_id: Option<String>,

        /// Filter by team ID
        #[arg(long)]
        team_id: Option<String>,

        /// Filter by assignee ID
        #[arg(long)]
        assignee_id: Option<String>,

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
        /// Project ID or slug
        id: String,
    },
    /// Create a new project
    Create {
        /// Project name
        name: String,

        /// Team IDs (comma-separated, at least one required)
        #[arg(long, value_delimiter = ',')]
        team_ids: Vec<String>,

        /// Project description
        #[arg(long)]
        description: Option<String>,
    },
    /// Edit a project
    Edit {
        /// Project ID
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
        /// Project ID
        project_id: String,
    },
    /// Add a project update
    Add {
        /// Project ID
        project_id: String,
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
        /// Filter by team ID
        #[arg(long)]
        team_id: Option<String>,
    },
    /// Create a label
    Create {
        /// Label name
        name: String,

        /// Team ID (required)
        #[arg(long)]
        team_id: String,

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
