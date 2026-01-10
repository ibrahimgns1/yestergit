use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManuelEntry {
    pub message: String,
    pub date: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Database {
    pub repositories: Vec<PathBuf>,
    #[serde(default)]
    pub entries: Vec<ManuelEntry>,
}

impl Database {
    pub fn load() -> Result<Self> {
        let path = get_db_path()?;

        if !path.exists() {
            return Ok(Database::default());
        }

        let content = fs::read_to_string(&path).context("Db can not be read.")?;
        let db: Database = serde_json::from_str(&content).context("Json error")?;

        Ok(db)
    }

    pub fn save(&self) -> Result<()> {
        let path = get_db_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).context("Can not saved to db.")?;

        Ok(())
    }

    pub fn add_repos(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if let Ok(abs_path) = fs::canonicalize(&path)
                && !self.repositories.contains(&abs_path)
            {
                self.repositories.push(abs_path);
            }
        }
    }

    pub fn add_entry(&mut self, message: String) {
        self.entries.push(ManuelEntry {
            message,
            date: Utc::now(),
        });
    }
}

fn get_db_path() -> Result<PathBuf> {
    if let Ok(env_path) = env::var("RECALL_DB_PATH") {
        return Ok(PathBuf::from(env_path));
    }
    let proj_dirs = ProjectDirs::from("com", "yestergit-cli", "yestergit")
        .context("Config path does not exists.")?;

    Ok(proj_dirs.config_dir().join("db.json"))
}
