use anyhow::Result;
use chrono::{Duration, Local, Utc};
use clap::Parser;
use std::path::PathBuf;
use yestergit::{
    config::{Args, Commands},
    db::Database,
    git_ops, scanner,
};

fn main() -> Result<()> {
    let args = Args::parse();
    match &args.command {
        Some(Commands::Scan { path }) => {
            let repos = scanner::scan_repos(path.clone())?;
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
            process_repos(vec![path.clone()], &args)?;
        }

        Some(Commands::List) => {
            let db = Database::load()?;
            println!("Tracked: {:?}", db.repositories);
        }

        Some(Commands::Note { message }) => {
            let mut db = Database::load()?;
            db.add_entry(message.clone());
            db.save()?;
            println!("NOte saved. it will appear in your next daily report.");
        }

        None => {
            let db = Database::load()?;
            println!("Generating Daily report.. for lovely... agile meetings ");

            if !db.repositories.is_empty() {
                process_repos(db.repositories, &args)?;
            }
            if !db.entries.is_empty() {
                println!("Manuel entries:");

                let now = Utc::now();
                let days_ago = now - Duration::days(args.days as i64);
                let mut found_notes = false;

                for entry in db.entries {
                    if entry.date > days_ago {
                        let local_date: chrono::DateTime<Local> =
                            chrono::DateTime::from(entry.date);
                        println!(
                            " - [{}] {} (Manuel) ",
                            local_date.format("%H:%M"),
                            entry.message
                        );
                        found_notes = true;
                    }
                }
                if !found_notes {
                    println!(" No manual notes for this period.");
                }
                println!();
            }
        }
    }

    Ok(())
}

fn process_repos(repos: Vec<PathBuf>, args: &Args) -> Result<()> {
    for repo_path in repos {
        let commits = git_ops::fetch_commits(&repo_path, args.days, args.author.clone());

        if let Ok(logs) = commits {
            if logs.is_empty() {
                continue;
            }

            let repo_name = repo_path.file_name().unwrap().to_string_lossy();
            println!("📦 REPO: {}", repo_name);

            for log in logs {
                println!(
                    "  - [{}] {} ({})",
                    log.date.format("%H:%M"),
                    log.message,
                    log.hash
                );
            }
            println!();
        } else if let Err(e) = commits
            && args.verbose
        {
            eprintln!("⚠️  Error reading {:?}: {}", repo_path, e);
        }
    }
    Ok(())
}
