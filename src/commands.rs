use anyhow::Result;
use colored::Colorize;
use rayon::prelude::*;
use std::path::PathBuf;
use tabled::{Table, Tabled};
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, Utc};
use crate::{
    ai,
    config::Args,
    db::{Database, ManuelEntry},
    git_ops, scanner, settings,
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
    Note(ManuelEntry),
}

impl TimelineEvent {
    fn date(&self) -> DateTime<Local> {
        match self {
            TimelineEvent::Commit(c, _) => c.date,
            TimelineEvent::Note(n) => DateTime::from(n.date),
        }
    }
}

pub fn scan(path: PathBuf) -> Result<()> {
    let repos = scanner::scan_repositories(path)?;

    if repos.is_empty() {
        println!("No repos found.");
        return Ok(());
    }

    let mut db = Database::load()?;
    db.add_repos(repos);
    db.save()?;
    println!("Repositories added to local database.");
    Ok(())
}

pub fn check(path: PathBuf, args: &Args) -> Result<()> {
    let db = Database::load()?;
    print_report(vec![path], db.entries, args)
}

pub fn list() -> Result<()> {
    let db = Database::load()?;
    println!("Tracked repos:");
    for repo in db.repositories {
        println!(" - {:?}", repo);
    }
    Ok(())
}

pub fn note(message: String) -> Result<()> {
    let mut db = Database::load()?;
    db.add_entry(message);
    db.save()?;
    println!("Note saved. It will appear in your next daily report.");
    Ok(())
}

pub fn config(
    set_key: Option<String>,
    set_url: Option<String>,
    set_model: Option<String>,
    set_prompt: Option<String>,
    set_lang: Option<String>,
) -> Result<()> {
    let cfg_name = "yestergit";
    let mut app_config: settings::AppConfig = confy::load(cfg_name, "config")?;
    let mut changed = false;

    if let Some(v) = set_url {
        app_config.ai.api_url = v;
        println!("API URL updated.");
        changed = true;
    }

    if let Some(v) = set_key {
        app_config.ai.api_key = v;
        println!("API Key updated.");
        changed = true;
    }

    if let Some(v) = set_model {
        app_config.ai.model = v;
        println!("AI Model updated.");
        changed = true;
    }

    if let Some(v) = set_prompt {
        app_config.ai.prompt = v;
        println!("Prompt changed.");
        changed = true;
    }

    if let Some(v) = set_lang {
        app_config.ai.language = v;
        println!("Language changed.");
        changed = true;
    }

    if changed {
        confy::store(cfg_name, "config", &app_config)?;
        println!("Settings saved.");
    } else {
        let path = confy::get_configuration_file_path(cfg_name, "config")?;
        println!("Config file: {:?}", path);
        if let Ok(db_path) = Database::get_path() {
             println!("Database file: {:?}", db_path);
        }
        println!("{:#?}", app_config);
    }
    Ok(())
}

pub fn summarize(args: &Args) -> Result<()> {
    let db = Database::load()?;
    let cfg_name = "yestergit";
    let app_config: settings::AppConfig = confy::load(cfg_name, "config")?;

    let logs = collect_logs_as_string(db.repositories, db.entries, args)?;
    if logs.trim().is_empty() {
        println!("There are no logs.");
        return Ok(());
    }

    println!("AI generating summary... ({})", app_config.ai.model);

    match ai::generate_summary(&app_config, logs) {
        Ok(summary) => {
            println!("\n{}", "--- Daily Report ---".bold().green());
            println!("{}", summary);
            println!("{}", "-----------------".green());
        }
        Err(e) => eprintln!("Failed to generate report: {}", e),
    }
    Ok(())
}

pub fn report_all(args: &Args) -> Result<()> {
    let db = Database::load()?;
    print_report(db.repositories, db.entries, args)
}

fn get_since_date(days_arg: Option<u64>) -> DateTime<Utc> {
    let local_now = Local::now();
    let today_midnight = local_now
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_local_timezone(Local)
        .unwrap();

    let days_to_go_back = days_arg.unwrap_or(if local_now.weekday() == chrono::Weekday::Mon {
        3
    } else {
        1
    });

    let since_local = today_midnight - Duration::days(days_to_go_back as i64);
    since_local.with_timezone(&Utc)
}

fn print_report(repos: Vec<PathBuf>, entries: Vec<ManuelEntry>, args: &Args) -> Result<()> {
    let since_utc = get_since_date(args.days);
    println!(
        "Reports since {}",
        since_utc.with_timezone(&Local).format("%d/%m %H:%M")
    );

    let mut all_events: Vec<TimelineEvent> = repos
        .par_iter()
        .map(
            |repo_path| match git_ops::fetch_commits(repo_path, since_utc, args.author.clone()) {
                Ok(logs) => {
                    let repo_name = std::fs::canonicalize(repo_path)
                        .ok()
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                        .unwrap_or_else(|| repo_path.to_string_lossy().to_string());

                    logs.into_iter()
                        .map(|c| TimelineEvent::Commit(c, repo_name.clone()))
                        .collect()
                }
                Err(_) => Vec::new(),
            },
        )
        .flatten()
        .collect();

    for entry in entries {
        if entry.date > since_utc {
            all_events.push(TimelineEvent::Note(entry));
        }
    }

    all_events.sort_by_key(|a| a.date());

    if all_events.is_empty() {
        println!("No events for this time.");
        return Ok(());
    }

    let mut table_rows = Vec::new();
    for event in all_events {
        let row = match event {
            TimelineEvent::Commit(c, repo_name) => ReportRow {
                time: c.date.format("%d/%m %H:%M").to_string(),
                event_type: "Git".to_string(),
                source: repo_name,
                message: c.message.to_string(),
                hash: c.hash,
            },
            TimelineEvent::Note(n) => ReportRow {
                time: DateTime::<Local>::from(n.date)
                    .format("%d/%m %H:%M")
                    .to_string(),
                event_type: "Note".to_string(),
                source: "-".to_string(),
                message: n.message,
                hash: "-".to_string(),
            },
        };
        table_rows.push(row);
    }
    println!("{}", Table::new(table_rows));
    Ok(())
}

fn collect_logs_as_string(
    repos: Vec<PathBuf>,
    entries: Vec<ManuelEntry>,
    args: &Args,
) -> Result<String> {
    let since_utc = get_since_date(args.days);

    let repo_logs: Vec<String> = repos
        .par_iter()
        .map(|repo_path| {
            let mut chunk = String::new();
            if let Ok(commits) = git_ops::fetch_commits(repo_path, since_utc, args.author.clone())
                && !commits.is_empty() {
                    let repo_name = repo_path.file_name().unwrap_or_default().to_string_lossy();
                    chunk.push_str(&format!("Project: {}\n", repo_name));

                    for c in commits {
                        chunk.push_str(&format!("- {}\n", c.message.trim()));
                    }
                    chunk.push('\n');
                }
            chunk
        })
        .collect();
    let mut clean_logs = String::new();
    for log in repo_logs {
        clean_logs.push_str(&log);
    }

    let notes: Vec<String> = entries
        .into_iter()
        .filter(|e| e.date > since_utc)
        .map(|e| format!("Note: {}\n", e.message.trim()))
        .collect();

    if !notes.is_empty() {
        clean_logs.push_str("\n --- Manual Notes --\n");
        for note in notes {
            clean_logs.push_str(&note);
        }
    }

    Ok(clean_logs)
}
