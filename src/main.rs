use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, Local, Utc};
use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use tabled::{Table, Tabled};
use yestergit::{
    config::{Args, Commands},
    db::{Database, ManuelEntry},
    git_ops, scanner,
};

#[derive(Tabled)]
struct ReportRow {
    #[tabled(rename = "Time")]
    time: String,

    #[tabled(rename = "Type")]
    event_type: String,

    #[tabled(rename = "Source / Repo")]
    source: String,

    #[tabled(rename = "Message")]
    message: String,

    #[tabled(rename = "Hash")]
    hash: String,
}

enum TimelineEvent {
    Commit(git_ops::CommitLog, String),
    Note(yestergit::db::ManuelEntry),
}

impl TimelineEvent {
    fn date(&self) -> DateTime<Local> {
        match self {
            TimelineEvent::Commit(c, _) => c.date,
            TimelineEvent::Note(n) => DateTime::from(n.date),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Some(Commands::Scan { path }) => {
            let repos = scanner::scan_repositories(path.clone())?;

            if repos.is_empty() {
                println!("No repos found.");
                return Ok(());
            }

            let mut db = Database::load()?;
            db.add_repos(repos);
            db.save()?;
            println!("Repo added to local db.");
        }
        Some(Commands::Check { path }) => {
            print_report(vec![path.clone()], Vec::new(), &args)?;
        }

        Some(Commands::List) => {
            let db = Database::load()?;
            println!("Tracked repos:");
            for repo in db.repositories {
                println!(" - {:?}", repo);
            }
        }

        Some(Commands::Note { message }) => {
            let mut db = Database::load()?;
            db.add_entry(message.clone());
            db.save()?;
            println!("Note saved. it will appear in your next daily report.");
        }

        None => {
            let db = Database::load()?;
            print_report(db.repositories, db.entries, &args)?;
        }
    }

    Ok(())
}

fn print_report(repos: Vec<PathBuf>, entries: Vec<ManuelEntry>, args: &Args) -> Result<()> {
    if repos.is_empty() && entries.is_empty() {
        println!("You literally did not commit anything yesterday! DO NOT SAY THAT!");
        return Ok(());
    }

    let now = Utc::now();
    let days_to_go_back = if let Some(d) = args.days {
        d
    } else if now.weekday() == chrono::Weekday::Mon {
        3
    } else {
        1
    };

    let since = now - Duration::days(days_to_go_back as i64);

    println!("Things you did for last {} days", days_to_go_back);

    let mut all_events: Vec<TimelineEvent> = repos
        .par_iter()
        .map(|repo_path| {
            let commits = git_ops::fetch_commits(repo_path, since, args.author.clone());
            match commits {
                Ok(logs) => {
                    let repo_name = repo_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    logs.into_iter()
                        .map(|c| TimelineEvent::Commit(c, repo_name.clone()))
                        .collect()
                }
                Err(_) => Vec::new(),
            }
        })
        .flatten()
        .collect();

    for entry in entries {
        if entry.date > since {
            all_events.push(TimelineEvent::Note(entry));
        }
    }

    all_events.sort_by_key(|a| a.date());

    if all_events.is_empty() {
        return Ok(());
    }

    let mut table_rows = Vec::new();

    for event in all_events {
        let row = match event {
            TimelineEvent::Commit(c, repo_name) => ReportRow {
                time: c.date.format("%d/%m %H:%M").to_string(),
                event_type: "Git".to_string(),
                source: repo_name,
                message: c.message,
                hash: c.hash,
            },
            TimelineEvent::Note(n) => ReportRow {
                time: DateTime::<Local>::from(n.date).format("%H:%M").to_string(),
                event_type: "Note".to_string(),
                source: "-".to_string(),
                message: n.message,
                hash: "-".to_string(),
            },
        };
        table_rows.push(row);
    }

    let table = Table::new(table_rows).to_string();
    println!("{}", table);

    Ok(())
}
