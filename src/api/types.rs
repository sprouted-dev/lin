use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Connection<T> {
    pub nodes: Vec<T>,
}

// --- Viewer ---

#[derive(Debug, Deserialize)]
pub struct ViewerData {
    pub viewer: User,
}

// --- User ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

// --- Team ---

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub key: Option<String>,
}

// --- Label ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

// --- WorkflowState ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: Option<String>,
}

// --- Project ---

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

// --- Issue ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<f64>,
    pub url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub due_date: Option<String>,
    pub state: Option<WorkflowState>,
    pub assignee: Option<User>,
    pub team: Option<Team>,
    pub project: Option<Project>,
    pub labels: Option<Connection<Label>>,
    pub children: Option<Connection<ChildIssue>>,
    pub parent: Option<ParentIssue>,
    pub cycle: Option<CycleRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleRef {
    pub id: String,
    pub number: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChildIssue {
    pub identifier: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParentIssue {
    pub identifier: String,
    pub title: String,
}

// --- Issue search ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueSearchData {
    pub search_issues: Connection<Issue>,
}

// --- Issues list ---

#[derive(Debug, Deserialize)]
pub struct IssuesData {
    pub issues: Connection<Issue>,
}

// --- Issue by ID ---

#[derive(Debug, Deserialize)]
pub struct IssueData {
    pub issue: Issue,
}

// --- Issue create ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateData {
    pub issue_create: IssuePayload,
}

#[derive(Debug, Deserialize)]
pub struct IssuePayload {
    pub success: bool,
    pub issue: Option<Issue>,
}

// --- Issue update ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateData {
    pub issue_update: IssuePayload,
}

// --- Workflow states ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamData {
    pub team: TeamWithStates,
}

#[derive(Debug, Deserialize)]
pub struct TeamWithStates {
    pub states: Connection<WorkflowState>,
}

// --- Input types ---

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateInput {
    pub title: String,
    pub team_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_id: Option<String>,
}

// --- Comment ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub user: Option<User>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentsData {
    pub issue: IssueWithComments,
}

#[derive(Debug, Deserialize)]
pub struct IssueWithComments {
    pub comments: Connection<Comment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreateData {
    pub comment_create: CommentPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentUpdateData {
    pub comment_update: CommentPayload,
}

#[derive(Debug, Deserialize)]
pub struct CommentPayload {
    pub success: bool,
    pub comment: Option<Comment>,
}

#[derive(Debug, Deserialize)]
pub struct CommentByIdData {
    pub comment: Option<CommentWithIssue>,
}

#[derive(Debug, Deserialize)]
pub struct CommentWithIssue {
    pub id: String,
    pub issue: CommentIssueRef,
}

#[derive(Debug, Deserialize)]
pub struct CommentIssueRef {
    pub id: String,
}

// --- Project (extended) ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub state: Option<String>,
    pub lead: Option<User>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectsData {
    pub projects: Connection<ProjectSummary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub state: Option<String>,
    pub lead: Option<User>,
    pub members: Option<Connection<ProjectMember>>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectMember {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectDetailData {
    pub project: ProjectDetail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateData {
    pub project_create: ProjectPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateMutationData {
    pub project_update: ProjectPayload,
}

#[derive(Debug, Deserialize)]
pub struct ProjectPayload {
    pub success: bool,
    pub project: Option<ProjectSummary>,
}

// --- Project Updates ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdate {
    pub id: String,
    pub body: Option<String>,
    pub health: Option<String>,
    pub url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub user: Option<User>,
    pub project: Option<ProjectRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateData {
    pub project_update: ProjectUpdate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWithUpdates {
    pub project_updates: Connection<ProjectUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectUpdatesData {
    pub project: ProjectWithUpdates,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateCreateData {
    pub project_update_create: ProjectUpdatePayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateEditData {
    pub project_update_update: ProjectUpdatePayload,
}

#[derive(Debug, Deserialize)]
pub struct ProjectUpdatePayload {
    pub success: bool,
    #[serde(rename = "projectUpdate")]
    pub project_update: Option<ProjectUpdate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateDeleteData {
    pub project_update_delete: DeletePayload,
}

#[derive(Debug, Deserialize)]
pub struct DeletePayload {
    pub success: bool,
}

// --- Teams (extended) ---

#[derive(Debug, Clone, Deserialize)]
pub struct TeamWithMembers {
    pub id: String,
    pub name: String,
    pub key: Option<String>,
    pub members: Option<Connection<MemberId>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberId {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamsData {
    pub teams: Connection<TeamWithMembers>,
}

// --- Users ---

#[derive(Debug, Deserialize)]
pub struct UsersData {
    pub users: Connection<User>,
}

// --- Pagination ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedConnection<T> {
    pub nodes: Vec<T>,
    pub page_info: PageInfo,
}

// --- Labels ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelsData {
    pub issue_labels: PaginatedConnection<Label>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelCreateData {
    pub issue_label_create: LabelPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelPayload {
    pub success: bool,
    pub issue_label: Option<Label>,
}

// --- Attachments ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadData {
    pub file_upload: FileUploadPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadPayload {
    pub upload_file: UploadFile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFile {
    pub upload_url: String,
    pub asset_url: String,
    #[serde(default)]
    pub headers: Vec<UploadHeader>,
}

#[derive(Debug, Deserialize)]
pub struct UploadHeader {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentCreateData {
    pub attachment_create: AttachmentPayload,
}

#[derive(Debug, Deserialize)]
pub struct AttachmentPayload {
    pub success: bool,
    pub attachment: Option<Attachment>,
}

#[derive(Debug, Deserialize)]
pub struct IssueAttachmentsData {
    pub issue: IssueWithAttachments,
}

#[derive(Debug, Deserialize)]
pub struct IssueWithAttachments {
    pub attachments: Connection<Attachment>,
}

#[derive(Debug, Deserialize)]
pub struct IssueDownloadData {
    pub issue: IssueForDownload,
}

#[derive(Debug, Deserialize)]
pub struct IssueForDownload {
    pub description: Option<String>,
    pub attachments: Connection<Attachment>,
}

// --- Input types ---

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_id: Option<String>,
}

// --- Cycles ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    pub id: String,
    pub number: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub completed_at: Option<String>,
    pub is_active: Option<bool>,
    pub is_future: Option<bool>,
    pub is_past: Option<bool>,
    pub progress: Option<f64>,
    pub issues: Option<Connection<Issue>>,
}

#[derive(Debug, Deserialize)]
pub struct CyclesData {
    pub cycles: Connection<Cycle>,
}

#[derive(Debug, Deserialize)]
pub struct CycleDetailData {
    pub cycle: Cycle,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CycleCreateInput {
    pub team_id: String,
    pub starts_at: String,
    pub ends_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CyclePayload {
    pub success: bool,
    pub cycle: Option<Cycle>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleCreateData {
    pub cycle_create: CyclePayload,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CycleUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleUpdateData {
    pub cycle_update: CyclePayload,
}

// --- Initiatives ---

#[derive(Debug, Clone, Deserialize)]
pub struct Initiative {
    pub id: String,
    pub name: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InitiativesData {
    pub initiatives: Connection<Initiative>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InitiativeDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub projects: Option<Connection<ProjectSummary>>,
}

#[derive(Debug, Deserialize)]
pub struct InitiativeDetailData {
    pub initiative: InitiativeDetail,
}
