use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use git2::{Repository, Sort};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CommitLog {
    pub message: String,
    pub author: String,
    pub date: DateTime<Local>,
    pub hash: String,
}

pub fn fetch_commits(
    repo_path: &PathBuf,
    since: DateTime<Utc>,
    author_filter: Option<String>,
) -> Result<Vec<CommitLog>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Could not find git repo: {:?}", repo_path))?;

    let mut revwalk = repo.revwalk()?;

    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }

    revwalk.set_sorting(Sort::TIME)?;

    let mut logs = Vec::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let commit_time_raw = commit.time();
        let commit_datetime_utc = Utc.timestamp_opt(commit_time_raw.seconds(), 0).unwrap();

        if commit_datetime_utc < since {
            break;
        }

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();

        if let Some(filter) = &author_filter
            && !author_name.to_lowercase().contains(&filter.to_lowercase())
        {
            continue;
        }

        let full_message = commit.message().unwrap_or("");
        let short_message = full_message.lines().next().unwrap_or("").to_string();


        logs.push(CommitLog {
            message: short_message,
            author: author_name,
            date: DateTime::from(commit_datetime_utc),
            hash: oid.to_string()[0..7].to_string(),
        });
    }
    Ok(logs)
}
