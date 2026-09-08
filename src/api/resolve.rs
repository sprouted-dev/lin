use anyhow::{Result, bail};
use serde_json::{Value, json};

use super::client::LinearClient;
use super::queries::*;
use super::types::*;
use crate::output;

/// Check if a string looks like a UUID (contains dashes and is long enough)
fn is_uuid(s: &str) -> bool {
    s.len() > 20 && s.contains('-')
}

/// Resolve an issue identifier (e.g., APP-123) to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_issue_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    let variables = json!({
        "term": identifier,
        "first": 1,
    });

    let data: IssueSearchData = client.execute(ISSUE_SEARCH_QUERY, Some(variables)).await?;

    let issue = data
        .search_issues
        .nodes
        .into_iter()
        .find(|i| i.identifier.eq_ignore_ascii_case(identifier));

    match issue {
        Some(i) => Ok(i.id),
        None => bail!("Could not resolve issue identifier: {}", identifier),
    }
}

/// Resolve a user identifier (name, email, or "me") to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_user_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    if identifier == "me" {
        let data: ViewerData = client.execute(VIEWER_QUERY, None).await?;
        return Ok(data.viewer.id);
    }

    let data: UsersData = client.execute(USERS_QUERY, None).await?;
    let lower = identifier.to_lowercase();

    let found = data.users.nodes.iter().find(|u| {
        u.name.to_lowercase() == lower
            || u.email.as_deref().map(|e| e.to_lowercase()) == Some(lower.clone())
            || u.display_name.as_deref().map(|d| d.to_lowercase()) == Some(lower.clone())
    });

    match found {
        Some(user) => Ok(user.id.clone()),
        None => bail!(
            "User '{}' not found. Use `lin user list` to see available users.",
            identifier
        ),
    }
}

/// Resolve a team identifier (name, key, or UUID) to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_team_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    let data: TeamsData = client.execute(TEAMS_QUERY, None).await?;
    let lower = identifier.to_lowercase();

    let found = data.teams.nodes.iter().find(|t| {
        t.name.to_lowercase() == lower
            || t.key.as_deref().map(|k| k.to_lowercase()) == Some(lower.clone())
    });

    match found {
        Some(team) => Ok(team.id.clone()),
        None => bail!(
            "Team '{}' not found. Use `lin team list` to see available teams.",
            identifier
        ),
    }
}

/// Resolve a project identifier (name, slug, or UUID) to a UUID.
/// If already a UUID, returns as-is.
pub async fn resolve_project_identifier(client: &LinearClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    let variables = json!({ "first": 250, "includeArchived": false });
    let data: ProjectsData = client.execute(PROJECTS_QUERY, Some(variables)).await?;
    let lower = identifier.to_lowercase();

    let found = data
        .projects
        .nodes
        .iter()
        .find(|p| p.name.to_lowercase() == lower);

    match found {
        Some(project) => Ok(project.id.clone()),
        None => bail!(
            "Project '{}' not found. Use `lin project list` to see available projects.",
            identifier
        ),
    }
}

/// Resolve a workflow state name to a UUID for a given issue.
/// Fetches the issue's team, then matches the state name case-insensitively.
pub async fn resolve_state_name(
    client: &LinearClient,
    issue_id: &str,
    state_name: &str,
) -> Result<String> {
    if is_uuid(state_name) {
        return Ok(state_name.to_string());
    }

    let issue_data: IssueData = client
        .execute(ISSUE_QUERY, Some(json!({ "id": issue_id })))
        .await?;
    let team = issue_data
        .issue
        .team
        .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;

    let team_data: TeamData = client
        .execute(TEAM_STATES_QUERY, Some(json!({ "id": team.id })))
        .await?;

    let target_lower = state_name.to_lowercase();
    let matching = team_data
        .team
        .states
        .nodes
        .iter()
        .find(|s| s.name.to_lowercase() == target_lower);

    match matching {
        Some(state) => Ok(state.id.clone()),
        None => {
            let available: Vec<&str> = team_data
                .team
                .states
                .nodes
                .iter()
                .map(|s| s.name.as_str())
                .collect();
            bail!(
                "State '{}' not found. Available states: {}",
                state_name,
                available.join(", ")
            )
        }
    }
}

/// Resolve a cycle identifier (name, number, "current", or UUID) to a UUID.
/// Requires a pre-resolved team_id since cycles are team-scoped.
/// If identifier is "current", returns the active cycle for the team.
pub async fn resolve_cycle_identifier(
    client: &LinearClient,
    team_id: &str,
    identifier: &str,
) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    // Handle "current" to get the active cycle
    if identifier.eq_ignore_ascii_case("current") {
        let variables = json!({
            "first": 1,
            "filter": {
                "team": { "id": { "eq": team_id } },
                "isActive": { "eq": true }
            },
        });

        let data: CyclesData = client.execute(CYCLES_QUERY, Some(variables)).await?;
        match data.cycles.nodes.first() {
            Some(cycle) => return Ok(cycle.id.clone()),
            None => bail!("No active cycle found for this team."),
        }
    }

    // Fetch all cycles for the team
    let variables = json!({
        "first": 100,
        "filter": { "team": { "id": { "eq": team_id } } },
    });

    let data: CyclesData = client.execute(CYCLES_QUERY, Some(variables)).await?;
    let lower = identifier.to_lowercase();

    // Try matching by number first (if identifier is numeric)
    if let Ok(num) = identifier.parse::<i32>()
        && let Some(cycle) = data.cycles.nodes.iter().find(|c| c.number == Some(num))
    {
        return Ok(cycle.id.clone());
    }

    // Try matching by name
    let found = data
        .cycles
        .nodes
        .iter()
        .find(|c| c.name.as_ref().map(|n| n.to_lowercase()) == Some(lower.clone()));

    match found {
        Some(cycle) => Ok(cycle.id.clone()),
        None => {
            let available: Vec<String> = data
                .cycles
                .nodes
                .iter()
                .map(|c| {
                    let num = c.number.map(|n| n.to_string()).unwrap_or_default();
                    let name = c.name.clone().unwrap_or_default();
                    if name.is_empty() {
                        format!("#{}", num)
                    } else {
                        format!("{} (#{num})", name)
                    }
                })
                .collect();
            bail!(
                "Cycle '{}' not found. Available cycles: {}",
                identifier,
                available.join(", ")
            )
        }
    }
}

/// Resolve a single label identifier (name or UUID) to a UUID.
///
/// Tries a case-insensitive name match first, since label names are free-text
/// and can themselves look like a UUID to `is_uuid` (e.g. a long hyphenated
/// name). Only when no name matches do we fall back to treating the input as a
/// raw UUID, so a genuine ID still works without a wasted error.
///
/// Linear allows the same label name in multiple teams, so a bare name can be
/// ambiguous. When more than one label matches we bail rather than pick an
/// arbitrary one — important because callers like `label delete` are
/// destructive. Pass `team` to scope the search to a single team and
/// disambiguate.
pub async fn resolve_label_identifier(
    client: &LinearClient,
    identifier: &str,
    team: Option<&str>,
) -> Result<String> {
    let filter = match team {
        Some(t) => {
            let team_id = resolve_team_identifier(client, t).await?;
            Some(json!({ "team": { "id": { "eq": team_id } } }))
        }
        None => None,
    };

    let all_labels = fetch_all_labels(client, filter).await?;

    resolve_unique_by_name(
        identifier,
        &all_labels,
        NameLookup {
            noun: "Label",
            plural: "labels",
            ambiguity_hint: "Re-run with --team to choose one, or pass the label's UUID.",
        },
        |l| l.name.as_str(),
        |l| l.id.as_str(),
        |l| owner_label(l.team.as_ref()),
    )
}

/// Resolve label names to label IDs via case-insensitive matching.
/// Paginates through all workspace labels to avoid missing any.
pub async fn resolve_label_names(client: &LinearClient, names: &[String]) -> Result<Vec<String>> {
    let all_labels = fetch_all_labels(client, None).await?;
    let mut ids = Vec::new();

    for name in names {
        let lower = name.to_lowercase();
        let found = all_labels.iter().find(|l| l.name.to_lowercase() == lower);
        match found {
            Some(label) => ids.push(label.id.clone()),
            None => bail!(
                "Label '{}' not found. Available labels: {}",
                name,
                all_labels
                    .iter()
                    .map(|l| l.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    Ok(ids)
}

/// Fetch all labels, paginating through all pages.
pub async fn fetch_all_labels(
    client: &LinearClient,
    filter: Option<serde_json::Value>,
) -> Result<Vec<Label>> {
    let mut all_labels: Vec<Label> = Vec::new();
    let mut after: Option<String> = None;

    loop {
        let mut vars = json!({ "first": 250 });
        if let Some(ref f) = filter {
            vars["filter"] = f.clone();
        }
        if let Some(ref cursor) = after {
            vars["after"] = json!(cursor);
        }

        let data: LabelsData = client.execute(LABELS_QUERY, Some(vars)).await?;

        all_labels.extend(data.issue_labels.nodes);

        if data.issue_labels.page_info.has_next_page {
            after = data.issue_labels.page_info.end_cursor;
            if after.is_none() {
                break;
            }
        } else {
            break;
        }
    }

    Ok(all_labels)
}

/// Human-readable owner of a team-scoped entity: the team key, falling back to
/// the team name, or `(workspace)` when it belongs to no team.
pub fn owner_label(team: Option<&TeamRef>) -> String {
    match team {
        Some(t) => t.key.clone().unwrap_or_else(|| t.name.clone()),
        None => "(workspace)".to_string(),
    }
}

/// Pick the single candidate whose name matches `identifier`, case-insensitively.
///
/// Names are tried before `is_uuid`, because free-text names can themselves look
/// like a UUID to `is_uuid` (e.g. a long hyphenated name such as
/// "Engineering - Bug Report"). Only when nothing matches by name do we fall
/// back to treating the input as a raw UUID, so a genuine ID still works
/// without a wasted error.
///
/// More than one match is an error rather than an arbitrary pick, since callers
/// include destructive commands like `label delete`.
/// Wording [`resolve_unique_by_name`] puts in its errors, so the one algorithm
/// can speak for whichever entity the caller is resolving.
struct NameLookup<'a> {
    /// Capitalised singular, e.g. `Template`.
    noun: &'a str,
    /// Lowercase plural, e.g. `templates`.
    plural: &'a str,
    /// Sentence telling the user how to narrow an ambiguous match.
    ambiguity_hint: &'a str,
}

fn resolve_unique_by_name<T>(
    identifier: &str,
    candidates: &[T],
    words: NameLookup<'_>,
    name_of: impl Fn(&T) -> &str,
    id_of: impl Fn(&T) -> &str,
    owner_of: impl Fn(&T) -> String,
) -> Result<String> {
    let NameLookup {
        noun,
        plural,
        ambiguity_hint,
    } = words;
    let lower = identifier.to_lowercase();
    let matches: Vec<&T> = candidates
        .iter()
        .filter(|c| name_of(c).to_lowercase() == lower)
        .collect();

    match matches.as_slice() {
        [only] => return Ok(id_of(only).to_string()),
        [_, ..] => bail!(
            "{} '{}' is ambiguous — {} {} share that name ({}). {}",
            noun,
            identifier,
            matches.len(),
            plural,
            matches
                .iter()
                .map(|c| owner_of(c))
                .collect::<Vec<_>>()
                .join(", "),
            ambiguity_hint
        ),
        [] => {}
    }

    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    bail!(
        "{} '{}' not found. Available {}: {}",
        noun,
        identifier,
        plural,
        if candidates.is_empty() {
            "(none)".to_string()
        } else {
            candidates
                .iter()
                .map(&name_of)
                .collect::<Vec<_>>()
                .join(", ")
        }
    )
}

/// Templates `templateSearch` can return in one call before it truncates.
const TEMPLATE_SEARCH_LIMIT: usize = 250;

/// Fetch templates in the order the Linear app shows them (`templateSearch`
/// documents that ordering), optionally narrowed to one entity type.
///
/// `team_id` must already be resolved — callers that hold a team ID should not
/// pay for a second `TEAMS_QUERY`. Scoping to a team returns the templates
/// *available to* that team: its own plus the workspace-level ones, which is
/// what the Linear issue-creation picker shows. `global_only` narrows to just
/// the workspace-level templates.
pub async fn fetch_templates(
    client: &LinearClient,
    team_id: Option<&str>,
    template_type: Option<&str>,
    global_only: bool,
) -> Result<Vec<Template>> {
    let mut filter = serde_json::Map::new();

    if let Some(t) = template_type {
        filter.insert("type".to_string(), json!({ "eq": t }));
    }

    if global_only {
        filter.insert("team".to_string(), json!({ "null": true }));
    } else if let Some(team_id) = team_id {
        // A workspace-level template has a null team, so a `team.id.eq` filter
        // alone would hide every global template from a team-scoped lookup.
        filter.insert(
            "or".to_string(),
            json!([
                { "team": { "id": { "eq": team_id } } },
                { "team": { "null": true } }
            ]),
        );
    }

    let mut vars = json!({ "first": TEMPLATE_SEARCH_LIMIT });
    if !filter.is_empty() {
        vars["filter"] = Value::Object(filter);
    }

    let data: TemplateSearchData = client.execute(TEMPLATE_SEARCH_QUERY, Some(vars)).await?;
    let templates = data.template_search;

    // templateSearch returns a plain list with no cursor, so there is no way to
    // page past the cap. Say so rather than presenting a truncated set as complete.
    if templates.len() >= TEMPLATE_SEARCH_LIMIT {
        output::print_warning(&format!(
            "Showing the first {TEMPLATE_SEARCH_LIMIT} templates; the API returns no more in one \
             request. Narrow the search with --team or --type."
        ));
    }

    Ok(templates)
}

/// Resolve a template name (case-insensitive) or UUID to a template ID.
/// `team_id` must already be resolved; see [`fetch_templates`].
pub async fn resolve_template_identifier(
    client: &LinearClient,
    identifier: &str,
    team_id: Option<&str>,
    template_type: Option<&str>,
) -> Result<String> {
    let templates = fetch_templates(client, team_id, template_type, false).await?;

    resolve_unique_by_name(
        identifier,
        &templates,
        NameLookup {
            noun: "Template",
            plural: "templates",
            ambiguity_hint: "Pass the template's UUID to choose one, or --team to narrow the search.",
        },
        |t| t.name.as_str(),
        |t| t.id.as_str(),
        |t| owner_label(t.team.as_ref()),
    )
}

#[cfg(test)]
mod tests {
    use super::{NameLookup, is_uuid, owner_label, resolve_unique_by_name};
    use crate::api::types::TeamRef;

    struct Candidate {
        id: &'static str,
        name: &'static str,
        team: Option<TeamRef>,
    }

    fn candidate(id: &'static str, name: &'static str, team_key: Option<&str>) -> Candidate {
        Candidate {
            id,
            name,
            team: team_key.map(|k| TeamRef {
                key: Some(k.to_string()),
                name: format!("{k} team"),
            }),
        }
    }

    fn resolve(identifier: &str, candidates: &[Candidate]) -> anyhow::Result<String> {
        resolve_unique_by_name(
            identifier,
            candidates,
            NameLookup {
                noun: "Template",
                plural: "templates",
                ambiguity_hint: "Pass the UUID.",
            },
            |c| c.name,
            |c| c.id,
            |c| owner_label(c.team.as_ref()),
        )
    }

    #[test]
    fn matches_name_case_insensitively() {
        let items = [candidate("id-1", "Bug Report", Some("ENG"))];
        assert_eq!(resolve("bug report", &items).unwrap(), "id-1");
    }

    #[test]
    fn hyphenated_name_beats_uuid_heuristic() {
        // "Engineering - Bug Report" is >20 chars and contains '-', so is_uuid
        // says true. The name match has to win, or the literal name is sent to
        // the API as an ID.
        let name = "Engineering - Bug Report";
        assert!(is_uuid(name));
        let items = [candidate("id-1", name, Some("ENG"))];
        assert_eq!(resolve(name, &items).unwrap(), "id-1");
    }

    #[test]
    fn falls_back_to_uuid_when_no_name_matches() {
        let items = [candidate("id-1", "Bug Report", Some("ENG"))];
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        assert_eq!(resolve(uuid, &items).unwrap(), uuid);
    }

    #[test]
    fn duplicate_names_are_ambiguous_not_arbitrary() {
        let items = [
            candidate("id-1", "Bug Report", Some("ENG")),
            candidate("id-2", "Bug Report", Some("APP")),
        ];
        let err = resolve("Bug Report", &items).unwrap_err().to_string();
        assert!(err.contains("ambiguous"), "{err}");
        assert!(err.contains("ENG") && err.contains("APP"), "{err}");
    }

    #[test]
    fn unknown_name_lists_what_is_available() {
        let items = [candidate("id-1", "Bug Report", Some("ENG"))];
        let err = resolve("Nope", &items).unwrap_err().to_string();
        assert!(err.contains("not found"), "{err}");
        assert!(err.contains("Bug Report"), "{err}");
    }

    #[test]
    fn empty_candidate_list_says_none() {
        let items: [Candidate; 0] = [];
        let err = resolve("Nope", &items).unwrap_err().to_string();
        assert!(err.contains("(none)"), "{err}");
    }

    #[test]
    fn workspace_level_entity_has_no_team_key() {
        let items = [candidate("id-1", "Global", None)];
        assert_eq!(owner_label(items[0].team.as_ref()), "(workspace)");
    }

    #[test]
    fn detects_real_uuids() {
        assert!(is_uuid("a1b2c3d4-e5f6-7890-abcd-ef1234567890"));
    }

    #[test]
    fn rejects_short_or_plain_strings() {
        assert!(!is_uuid("bug"));
        assert!(!is_uuid("needs review")); // spaces, no dash
        assert!(!is_uuid("good-first-issue")); // has dash but under the length cutoff
    }

    #[test]
    fn long_hyphenated_label_name_looks_like_a_uuid() {
        // The heuristic can't tell this free-text label name from a UUID, which
        // is exactly why `resolve_label_identifier` matches by name first and
        // only falls back to treating the input as a raw id.
        assert!(is_uuid("needs-more-information-before-triage"));
    }
}
