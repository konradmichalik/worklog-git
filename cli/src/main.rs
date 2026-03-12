mod cli;
mod clipboard;
mod config;
mod interactive;
mod output;

use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use devcap_core::{
    discovery, git, model,
    period::{Period, TimeRange},
};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let cfg = config::load();

    let range = resolve_time_range(cli.since, cli.until, cli.period, &cfg)?;

    let path = cli.path.or(cfg.path).unwrap_or_else(|| PathBuf::from("."));
    let author = cli.author.or(cfg.author).or_else(git::default_author);
    let show_origin = cli.show_origin || cfg.show_origin.unwrap_or(false);
    let with_stat = cli.stat || cfg.stat.unwrap_or(false);

    let use_color = if cli.no_color || cli.json {
        false
    } else if let Some(cfg_color) = cfg.color {
        cfg_color
    } else {
        std::io::stdout().is_terminal()
    };
    output::set_color_enabled(use_color);
    let author_ref = author.as_deref();

    let spinner = if !cli.json {
        let sp = ProgressBar::new_spinner();
        if let Ok(style) = ProgressStyle::default_spinner()
            .tick_strings(&[
                "\u{2802}", "\u{2816}", "\u{2834}", "\u{2830}", "\u{2860}", "\u{28e0}", "\u{28c0}",
                "\u{2880}",
            ])
            .template("{spinner} {msg}")
        {
            sp.set_style(style);
        }
        sp.set_message("Scanning repositories...");
        sp.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(sp)
    } else {
        None
    };

    let repos = discovery::find_repos(&path);

    if repos.is_empty() {
        if let Some(sp) = &spinner {
            sp.finish_and_clear();
        }
        if cli.json {
            println!("[]");
        } else {
            eprintln!("No git repositories found in: {}", path.display());
        }
        return Ok(());
    }

    let mut projects: Vec<_> = repos
        .par_iter()
        .filter_map(|repo| git::collect_project_log(repo, &range, author_ref, with_stat))
        .collect();

    let sort_spec = cli
        .sort
        .or_else(|| {
            cfg.sort
                .as_deref()
                .and_then(|s| s.parse::<cli::SortSpec>().ok())
        })
        .unwrap_or_default();

    projects.sort_by(|a, b| {
        let ord = match sort_spec.field {
            cli::SortField::Time => {
                let latest = |p: &model::ProjectLog| {
                    p.branches
                        .iter()
                        .flat_map(|br| br.commits.first())
                        .map(|c| c.time)
                        .max()
                };
                latest(a).cmp(&latest(b))
            }
            cli::SortField::Commits => {
                let count = |p: &model::ProjectLog| {
                    p.branches.iter().map(|br| br.commits.len()).sum::<usize>()
                };
                count(a).cmp(&count(b))
            }
            cli::SortField::Name => a.project.to_lowercase().cmp(&b.project.to_lowercase()),
            cli::SortField::Lines => {
                let lines = |p: &model::ProjectLog| {
                    p.branches
                        .iter()
                        .flat_map(|br| &br.commits)
                        .filter_map(|c| c.diff_stat.as_ref())
                        .map(|s| (s.insertions + s.deletions) as u64)
                        .sum::<u64>()
                };
                lines(a).cmp(&lines(b))
            }
        };
        match sort_spec.direction {
            cli::SortDirection::Asc => ord,
            cli::SortDirection::Desc => ord.reverse(),
        }
    });

    if let Some(sp) = &spinner {
        sp.finish_with_message(format!("\u{2713} {}", output::summary_line(&projects)));
    }

    if cli.interactive {
        interactive::run(&projects, show_origin)?;
    } else if cli.json {
        println!("{}", output::render_json(&projects));
    } else {
        if !projects.is_empty() {
            println!();
        }
        output::render_terminal(&projects, cli.depth, show_origin);
    }

    if cli.copy {
        let text = clipboard::render_plain(&projects, cli.depth, show_origin);
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Err(e) = cb.set_text(&text) {
                    eprintln!("Warning: could not copy to clipboard: {e}");
                } else {
                    eprintln!("Copied to clipboard.");
                }
            }
            Err(e) => eprintln!("Warning: clipboard unavailable: {e}"),
        }
    }

    Ok(())
}

fn resolve_time_range(
    cli_since: Option<NaiveDate>,
    cli_until: Option<NaiveDate>,
    cli_period: Option<Period>,
    cfg: &config::DevcapConfig,
) -> Result<TimeRange> {
    let since = cli_since.or_else(|| {
        cfg.since
            .as_deref()
            .and_then(|s| s.parse::<NaiveDate>().ok())
    });
    let until = cli_until.or_else(|| {
        cfg.until
            .as_deref()
            .and_then(|s| s.parse::<NaiveDate>().ok())
    });

    let resolve_period = || {
        cli_period
            .or_else(|| cfg.period.as_deref().and_then(|s| s.parse::<Period>().ok()))
            .unwrap_or(Period::Today)
    };

    match (since, until) {
        (Some(s), Some(u)) => TimeRange::from_dates(s, u).map_err(|e| anyhow::anyhow!(e)),
        (Some(s), None) => TimeRange::from_since_date(s).map_err(|e| anyhow::anyhow!(e)),
        (None, Some(u)) => {
            let range = resolve_period().to_time_range();
            range.with_until_date(u).map_err(|e| anyhow::anyhow!(e))
        }
        (None, None) => Ok(resolve_period().to_time_range()),
    }
}
