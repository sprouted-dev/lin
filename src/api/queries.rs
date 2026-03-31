pub const VIEWER_QUERY: &str = r#"
    query {
        viewer {
            id
            name
            email
            displayName
        }
    }
"#;

pub const ISSUE_QUERY: &str = r#"
    query Issue($id: String!) {
        issue(id: $id) {
            id
            identifier
            title
            description
            priority
            url
            createdAt
            updatedAt
            dueDate
            state { id name type }
            assignee { id name email displayName }
            team { id name key }
            project { id name }
            labels { nodes { id name color } }
            children { nodes { identifier title } }
            parent { identifier title }
            cycle { id number name }
        }
    }
"#;

pub const ISSUE_SEARCH_QUERY: &str = r#"
    query SearchIssues($term: String!, $first: Int, $filter: IssueFilter, $includeComments: Boolean) {
        searchIssues(term: $term, first: $first, filter: $filter, includeComments: $includeComments) {
            nodes {
                id
                identifier
                title
                priority
                url
                state { id name type }
                assignee { id name displayName }
                team { id name key }
            }
        }
    }
"#;

pub const ISSUES_QUERY: &str = r#"
    query Issues($first: Int, $filter: IssueFilter, $includeArchived: Boolean) {
        issues(first: $first, filter: $filter, includeArchived: $includeArchived) {
            nodes {
                id
                identifier
                title
                priority
                url
                state { id name type }
                assignee { id name displayName }
                team { id name key }
            }
        }
    }
"#;

pub const ISSUE_CREATE_MUTATION: &str = r#"
    mutation IssueCreate($input: IssueCreateInput!) {
        issueCreate(input: $input) {
            success
            issue {
                id
                identifier
                title
                url
                state { id name }
                team { id name key }
            }
        }
    }
"#;

pub const ISSUE_UPDATE_MUTATION: &str = r#"
    mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
        issueUpdate(id: $id, input: $input) {
            success
            issue {
                id
                identifier
                title
                url
                state { id name }
            }
        }
    }
"#;

pub const TEAM_STATES_QUERY: &str = r#"
    query Team($id: String!) {
        team(id: $id) {
            states {
                nodes {
                    id
                    name
                    type
                }
            }
        }
    }
"#;

// --- Comments ---

pub const COMMENTS_QUERY: &str = r#"
    query IssueComments($id: String!) {
        issue(id: $id) {
            comments {
                nodes {
                    id
                    body
                    user { id name }
                    createdAt
                    updatedAt
                }
            }
        }
    }
"#;

pub const COMMENT_CREATE_MUTATION: &str = r#"
    mutation CommentCreate($input: CommentCreateInput!) {
        commentCreate(input: $input) {
            success
            comment {
                id
                body
            }
        }
    }
"#;

pub const COMMENT_UPDATE_MUTATION: &str = r#"
    mutation CommentUpdate($id: String!, $input: CommentUpdateInput!) {
        commentUpdate(id: $id, input: $input) {
            success
            comment {
                id
                body
            }
        }
    }
"#;

// --- Projects ---

pub const PROJECTS_QUERY: &str = r#"
    query Projects($first: Int, $includeArchived: Boolean) {
        projects(first: $first, includeArchived: $includeArchived) {
            nodes {
                id
                name
                state
                lead { id name }
                startDate
                targetDate
            }
        }
    }
"#;

pub const PROJECT_QUERY: &str = r#"
    query Project($id: String!) {
        project(id: $id) {
            id
            name
            description
            content
            state
            lead { id name }
            members { nodes { name } }
            startDate
            targetDate
            url
        }
    }
"#;

pub const PROJECT_CREATE_MUTATION: &str = r#"
    mutation ProjectCreate($input: ProjectCreateInput!) {
        projectCreate(input: $input) {
            success
            project {
                id
                name
                url
            }
        }
    }
"#;

pub const PROJECT_UPDATE_MUTATION: &str = r#"
    mutation ProjectUpdate($id: String!, $input: ProjectUpdateInput!) {
        projectUpdate(id: $id, input: $input) {
            success
            project {
                id
                name
            }
        }
    }
"#;

// --- Project Updates ---

pub const PROJECT_UPDATES_QUERY: &str = r#"
    query ProjectUpdates($id: String!) {
        project(id: $id) {
            projectUpdates {
                nodes {
                    id
                    body
                    health
                    createdAt
                    user { id name }
                }
            }
        }
    }
"#;

pub const PROJECT_UPDATE_QUERY: &str = r#"
    query ProjectUpdate($id: String!) {
        projectUpdate(id: $id) {
            id
            body
            health
            url
            createdAt
            updatedAt
            user { id name }
            project { id name }
        }
    }
"#;

pub const PROJECT_UPDATE_CREATE_MUTATION: &str = r#"
    mutation ProjectUpdateCreate($input: ProjectUpdateCreateInput!) {
        projectUpdateCreate(input: $input) {
            success
            projectUpdate {
                id
                body
            }
        }
    }
"#;

pub const PROJECT_UPDATE_EDIT_MUTATION: &str = r#"
    mutation ProjectUpdateUpdate($id: String!, $input: ProjectUpdateUpdateInput!) {
        projectUpdateUpdate(id: $id, input: $input) {
            success
            projectUpdate {
                id
                body
            }
        }
    }
"#;

pub const PROJECT_UPDATE_DELETE_MUTATION: &str = r#"
    mutation ProjectUpdateDelete($id: String!) {
        projectUpdateDelete(id: $id) {
            success
        }
    }
"#;

// --- Teams ---

pub const TEAMS_QUERY: &str = r#"
    query Teams {
        teams {
            nodes {
                id
                name
                key
                members { nodes { id } }
            }
        }
    }
"#;

// --- Users ---

pub const USERS_QUERY: &str = r#"
    query Users {
        users {
            nodes {
                id
                name
                email
                displayName
            }
        }
    }
"#;

// --- Labels ---

pub const LABELS_QUERY: &str = r#"
    query IssueLabels($filter: IssueLabelFilter) {
        issueLabels(filter: $filter) {
            nodes {
                id
                name
                color
            }
        }
    }
"#;

pub const LABEL_CREATE_MUTATION: &str = r#"
    mutation IssueLabelCreate($input: IssueLabelCreateInput!) {
        issueLabelCreate(input: $input) {
            success
            issueLabel {
                id
                name
                color
            }
        }
    }
"#;

// --- File Attachments ---

pub const FILE_UPLOAD_MUTATION: &str = r#"
    mutation FileUpload($contentType: String!, $filename: String!, $size: Int!) {
        fileUpload(contentType: $contentType, filename: $filename, size: $size) {
            uploadFile {
                uploadUrl
                assetUrl
                headers {
                    key
                    value
                }
            }
        }
    }
"#;

pub const ATTACHMENT_CREATE_MUTATION: &str = r#"
    mutation AttachmentCreate($input: AttachmentCreateInput!) {
        attachmentCreate(input: $input) {
            success
            attachment {
                id
                title
                url
            }
        }
    }
"#;

pub const ISSUE_ATTACHMENTS_QUERY: &str = r#"
    query IssueAttachments($id: String!) {
        issue(id: $id) {
            attachments {
                nodes {
                    id
                    title
                    url
                    createdAt
                }
            }
        }
    }
"#;

// --- Cycles ---

pub const CYCLES_QUERY: &str = r#"
    query Cycles($first: Int, $filter: CycleFilter) {
        cycles(first: $first, filter: $filter) {
            nodes {
                id
                number
                name
                startsAt
                endsAt
            }
        }
    }
"#;

// --- Initiatives ---

pub const INITIATIVES_QUERY: &str = r#"
    query Initiatives($first: Int) {
        initiatives(first: $first) {
            nodes {
                id
                name
                status
            }
        }
    }
"#;

pub const INITIATIVE_QUERY: &str = r#"
    query Initiative($id: String!) {
        initiative(id: $id) {
            id
            name
            description
            status
            projects {
                nodes {
                    id
                    name
                    state
                }
            }
        }
    }
"#;
