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

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Signature, Time};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        (temp_dir, repo)
    }

    fn create_commit(repo: &Repository, message: &str, time_offset_secs: i64, author: &str) {
        let mut index = repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let time = Time::new(
            Utc::now().timestamp() + time_offset_secs,
            0,
        );
        let signature = Signature::new(author, "email@example.com", &time).unwrap();

        let parent_commits = match repo.head() {
            Ok(head) => vec![repo.find_commit(head.target().unwrap()).unwrap()],
            Err(_) => vec![],
        };
        let parents: Vec<&git2::Commit> = parent_commits.iter().collect();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )
        .unwrap();
    }

    #[test]
    fn test_fetch_commits_basic() {
        let (temp_dir, repo) = setup_repo();

        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        create_commit(&repo, "Initial commit", -100, "Alice");
        create_commit(&repo, "Second commit", -50, "Bob");

        let since = Utc::now() - chrono::Duration::hours(1);
        let logs = fetch_commits(&temp_dir.path().to_path_buf(), since, None).unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "Second commit");
        assert_eq!(logs[1].message, "Initial commit");
    }

    #[test]
    fn test_fetch_commits_time_filter() {
        let (temp_dir, repo) = setup_repo();

        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        create_commit(&repo, "Old commit", -3600 * 5, "Alice");
        create_commit(&repo, "New commit", -60, "Alice");

        let since = Utc::now() - chrono::Duration::hours(1);
        let logs = fetch_commits(&temp_dir.path().to_path_buf(), since, None).unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "New commit");
    }

    #[test]
    fn test_fetch_commits_author_filter() {
        let (temp_dir, repo) = setup_repo();

        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        create_commit(&repo, "Alice commit", -100, "Alice");
        create_commit(&repo, "Bob commit", -50, "Bob");

        let since = Utc::now() - chrono::Duration::hours(1);

        let alice_logs = fetch_commits(&temp_dir.path().to_path_buf(), since, Some("Alice".to_string())).unwrap();
        assert_eq!(alice_logs.len(), 1);
        assert_eq!(alice_logs[0].author, "Alice");

        let bob_logs = fetch_commits(&temp_dir.path().to_path_buf(), since, Some("Bob".to_string())).unwrap();
        assert_eq!(bob_logs.len(), 1);
        assert_eq!(bob_logs[0].author, "Bob");
    }
}
