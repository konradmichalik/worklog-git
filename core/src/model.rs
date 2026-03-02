use std::collections::HashSet;
use std::fmt;

use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiffStat {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RepoOrigin {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "gitlab")]
    GitLab,
    #[serde(rename = "bitbucket")]
    Bitbucket,
    #[serde(rename = "gitlab-self-hosted")]
    GitLabSelfHosted,
    #[serde(untagged)]
    Custom(String),
}

impl fmt::Display for RepoOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepoOrigin::GitHub => write!(f, "GitHub"),
            RepoOrigin::GitLab => write!(f, "GitLab"),
            RepoOrigin::Bitbucket => write!(f, "Bitbucket"),
            RepoOrigin::GitLabSelfHosted => write!(f, "GitLab Self-Hosted"),
            RepoOrigin::Custom(host) => write!(f, "{host}"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_type: Option<String>,
    #[serde(rename = "timestamp")]
    pub time: DateTime<Local>,
    pub relative_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_stat: Option<DiffStat>,
}

#[derive(Debug, Serialize)]
pub struct BranchLog {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub commits: Vec<Commit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_stat: Option<DiffStat>,
}

#[derive(Debug, Serialize)]
pub struct ProjectLog {
    pub project: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<RepoOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    pub branches: Vec<BranchLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_stat: Option<DiffStat>,
}

impl BranchLog {
    pub fn latest_activity(&self) -> Option<&str> {
        self.commits.first().map(|c| c.relative_time.as_str())
    }
}

impl ProjectLog {
    pub fn total_commits(&self) -> usize {
        let mut seen = HashSet::new();
        self.branches
            .iter()
            .flat_map(|b| &b.commits)
            .filter(|c| seen.insert(&c.hash))
            .count()
    }

    pub fn latest_activity(&self) -> Option<&str> {
        self.branches
            .iter()
            .flat_map(|b| b.commits.first())
            .max_by_key(|c| c.time)
            .map(|c| c.relative_time.as_str())
    }
}
