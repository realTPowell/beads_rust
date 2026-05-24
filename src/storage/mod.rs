use crate::model::{
    Comment, Dependency, DependencyType, Event, Issue, IssueType, Priority, Status,
};
use anyhow::Result;
use chrono::{DateTime, Utc};

pub trait Storage: Send + Sync {
    // ── CRUD ──────────────────────────────────────────────────────────────────
    fn create(&self, issue: &NewIssue) -> Result<Issue>;
    fn get(&self, id: &str) -> Result<Option<Issue>>;
    fn update(&self, id: &str, patch: &IssueUpdate) -> Result<Issue>;
    fn close(&self, id: &str, reason: Option<&str>, actor: Option<&str>) -> Result<Issue>;
    fn reopen(&self, id: &str, actor: Option<&str>) -> Result<Issue>;

    // ── LIST ──────────────────────────────────────────────────────────────────
    fn list(&self, filters: &ListFilters) -> Result<Vec<Issue>>;
    fn count(&self, filters: &ListFilters) -> Result<u64>;

    // ── SEARCH ────────────────────────────────────────────────────────────────
    fn search(&self, query: &str, filters: &ListFilters) -> Result<Vec<Issue>>;

    // ── DEPENDENCY GRAPH ──────────────────────────────────────────────────────
    fn dep_add(&self, from: &str, to: &str, dep_type: DependencyType) -> Result<()>;
    fn dep_remove(&self, from: &str, to: &str) -> Result<()>;
    fn deps_of(&self, id: &str) -> Result<Vec<Dependency>>; // what `id` depends on
    fn dependents_of(&self, id: &str) -> Result<Vec<Dependency>>; // what depends on `id`
    fn ready(&self, filters: &ReadyFilters) -> Result<Vec<Issue>>;
    fn blocked(&self, filters: &ListFilters) -> Result<Vec<Issue>>;
    fn cycles(&self) -> Result<Vec<Vec<String>>>; // each Vec is one cycle

    // ── LABELS ────────────────────────────────────────────────────────────────
    fn labels_add(&self, id: &str, labels: &[String]) -> Result<()>;
    fn labels_remove(&self, id: &str, labels: &[String]) -> Result<()>;
    fn labels_list(&self) -> Result<Vec<String>>;

    // ── COMMENTS ──────────────────────────────────────────────────────────────
    fn comment_add(&self, id: &str, author: &str, body: &str) -> Result<Comment>;
    fn comments_list(&self, id: &str) -> Result<Vec<Comment>>;

    // ── EVENTS ────────────────────────────────────────────────────────────────
    fn events_list(&self, id: &str) -> Result<Vec<Event>>;
    fn events_recent(&self, limit: usize) -> Result<Vec<Event>>;

    // ── STATS ─────────────────────────────────────────────────────────────────
    fn stats(&self) -> Result<Stats>;

    // ── SYNC SUPPORT ──────────────────────────────────────────────────────────
    fn all_for_export(&self) -> Result<Vec<Issue>>; // full dump for JSONL flush
    fn upsert(&self, issue: &Issue) -> Result<UpsertResult>; // import-time merge
}

// ── Supporting types ──────────────────────────────────────────────────────────
pub struct NewIssue {
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub issue_type: IssueType,
    pub assignee: Option<String>,
    pub owner: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub defer_until: Option<DateTime<Utc>>,
    pub external_ref: Option<String>,
    pub labels: Vec<String>,
    pub created_by: Option<String>,
}

// Option<Option<T>>: None = leave alone, Some(None) = clear, Some(Some(v)) = set
pub struct IssueUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub issue_type: Option<IssueType>,
    pub assignee: Option<Option<String>>,
    pub owner: Option<Option<String>>,
    pub due_at: Option<Option<DateTime<Utc>>>,
    pub defer_until: Option<Option<DateTime<Utc>>>,
    pub external_ref: Option<Option<String>>,
    pub close_reason: Option<Option<String>>,
    pub closed_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Default)]
pub struct ListFilters {
    pub statuses: Option<Vec<Status>>,
    pub types: Option<Vec<IssueType>>,
    pub priorities: Option<Vec<Priority>>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,    // AND semantics
    pub labels_or: Option<Vec<String>>, // OR semantics
    pub include_closed: bool,
    pub include_deferred: bool,
    pub title_contains: Option<String>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
    pub sort: SortField,
    pub reverse: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Default)]
pub enum SortField {
    #[default]
    Priority,
    CreatedAt,
    UpdatedAt,
    Title,
}

#[derive(Default)]
pub struct ReadyFilters {
    pub assignee: Option<String>,
    pub labels_and: Vec<String>,
    pub labels_or: Vec<String>,
    pub types: Option<Vec<IssueType>>,
    pub priorities: Option<Vec<Priority>>,
    pub parent: Option<String>, // restrict to children of this epic
    pub limit: Option<usize>,
}

pub struct Stats {
    pub total: u64,
    pub by_status: Vec<(Status, u64)>,
    pub by_type: Vec<(IssueType, u64)>,
    pub by_priority: Vec<(Priority, u64)>,
}

pub enum UpsertResult {
    Inserted,
    Updated,
    Skipped, // content hash matched — no change needed
}
