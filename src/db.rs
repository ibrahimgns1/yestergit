use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
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
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;

        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let mut temp_file = tempfile::Builder::new()
            .prefix("yestergit_db_")
            .suffix(".tmp")
            .tempfile_in(parent)
            .context("Failed to create temp file for db save")?;

        temp_file
            .write_all(content.as_bytes())
            .context("Failed to write to temp db file")?;

        temp_file.flush().context("Failed to flush temp db file")?;

        temp_file.persist(path).context("Failed to replace db file")?;

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

    pub fn get_path() -> Result<PathBuf> {
        get_db_path()
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_to_atomic() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_db.json");

        let mut db = Database::default();
        db.add_entry("Test Note".to_string());

        db.save_to(&db_path).unwrap();
        assert!(db_path.exists());

        let content = fs::read_to_string(&db_path).unwrap();
        assert!(content.contains("Test Note"));

        db.add_entry("Second Note".to_string());
        db.save_to(&db_path).unwrap();

        let content_new = fs::read_to_string(&db_path).unwrap();
        assert!(content_new.contains("Second Note"));
    }
}
