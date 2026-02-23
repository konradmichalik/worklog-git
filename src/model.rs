use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_type: Option<String>,
    #[serde(rename = "timestamp")]
    pub time: DateTime<Local>,
    pub relative_time: String,
}

#[derive(Debug, Serialize)]
pub struct BranchLog {
    pub name: String,
    pub commits: Vec<Commit>,
}

#[derive(Debug, Serialize)]
pub struct ProjectLog {
    pub project: String,
    pub path: String,
    pub branches: Vec<BranchLog>,
}

impl ProjectLog {
    pub fn total_commits(&self) -> usize {
        self.branches.iter().map(|b| b.commits.len()).sum()
    }
}
