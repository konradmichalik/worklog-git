use clap::{Parser, ValueEnum};
use devcap_core::period::Period;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Depth {
    /// Show only project names with summary
    Projects,
    /// Show projects and branches
    Branches,
    /// Show projects, branches, and commits (default)
    Commits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Time,
    Commits,
    Name,
    Lines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortSpec {
    pub field: SortField,
    pub direction: SortDirection,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self {
            field: SortField::Time,
            direction: SortDirection::Desc,
        }
    }
}

impl FromStr for SortSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (field_str, dir_str) = match s.split_once(':') {
            Some((f, d)) => (f, Some(d)),
            None => (s, None),
        };

        let field = match field_str {
            "time" => SortField::Time,
            "commits" => SortField::Commits,
            "name" => SortField::Name,
            "lines" => SortField::Lines,
            _ => return Err(format!("unknown sort field: {field_str} (expected: time, commits, name, lines)")),
        };

        let default_dir = match field {
            SortField::Name => SortDirection::Asc,
            _ => SortDirection::Desc,
        };

        let direction = match dir_str {
            Some("asc") => SortDirection::Asc,
            Some("desc") => SortDirection::Desc,
            None => default_dir,
            Some(d) => return Err(format!("unknown sort direction: {d} (expected: asc, desc)")),
        };

        Ok(Self { field, direction })
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "devcap",
    about = "Aggregate git commits across repos for standups and time tracking",
    version
)]
pub struct Cli {
    /// Time period: today, yesterday, 24h, 3d, 7d, week
    #[arg(short, long)]
    pub period: Option<Period>,

    /// Root directory to scan for git repos
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Disable colored output (overrides TTY auto-detection)
    #[arg(long)]
    pub no_color: bool,

    /// Interactive drill-down mode (projects > branches > commits)
    #[arg(short, long, conflicts_with = "json")]
    pub interactive: bool,

    /// Output depth: projects, branches, commits
    #[arg(short, long, default_value = "commits", conflicts_with = "json")]
    pub depth: Depth,

    /// Filter by author name (defaults to git config user.name)
    #[arg(short, long)]
    pub author: Option<String>,

    /// Show repository origin (GitHub, GitLab, etc.)
    #[arg(short = 'o', long)]
    pub show_origin: bool,

    /// Show diff stats (+insertions -deletions ~files) per commit
    #[arg(short = 's', long)]
    pub stat: bool,

    /// Sort projects: time, commits, name, lines (append :asc or :desc)
    #[arg(long)]
    pub sort: Option<SortSpec>,

    /// Copy output to clipboard as plain text (for stand-ups)
    #[arg(long)]
    pub copy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sort_field_only_defaults_direction() {
        let spec: SortSpec = "time".parse().unwrap();
        assert_eq!(spec.field, SortField::Time);
        assert_eq!(spec.direction, SortDirection::Desc);

        let spec: SortSpec = "commits".parse().unwrap();
        assert_eq!(spec.field, SortField::Commits);
        assert_eq!(spec.direction, SortDirection::Desc);

        let spec: SortSpec = "name".parse().unwrap();
        assert_eq!(spec.field, SortField::Name);
        assert_eq!(spec.direction, SortDirection::Asc);

        let spec: SortSpec = "lines".parse().unwrap();
        assert_eq!(spec.field, SortField::Lines);
        assert_eq!(spec.direction, SortDirection::Desc);
    }

    #[test]
    fn parse_sort_with_explicit_direction() {
        let spec: SortSpec = "time:asc".parse().unwrap();
        assert_eq!(spec.field, SortField::Time);
        assert_eq!(spec.direction, SortDirection::Asc);

        let spec: SortSpec = "name:desc".parse().unwrap();
        assert_eq!(spec.field, SortField::Name);
        assert_eq!(spec.direction, SortDirection::Desc);

        let spec: SortSpec = "commits:asc".parse().unwrap();
        assert_eq!(spec.field, SortField::Commits);
        assert_eq!(spec.direction, SortDirection::Asc);

        let spec: SortSpec = "lines:asc".parse().unwrap();
        assert_eq!(spec.field, SortField::Lines);
        assert_eq!(spec.direction, SortDirection::Asc);
    }

    #[test]
    fn parse_sort_unknown_field_errors() {
        assert!("foo".parse::<SortSpec>().is_err());
        assert!("foo:asc".parse::<SortSpec>().is_err());
    }

    #[test]
    fn parse_sort_unknown_direction_errors() {
        assert!("time:up".parse::<SortSpec>().is_err());
    }

    #[test]
    fn default_sort_spec_is_time_desc() {
        let spec = SortSpec::default();
        assert_eq!(spec.field, SortField::Time);
        assert_eq!(spec.direction, SortDirection::Desc);
    }
}
