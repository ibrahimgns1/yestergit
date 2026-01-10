use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') && s != "." && s != "..")
        .unwrap_or(false)
}

fn scan_worker(root: PathBuf) -> Vec<PathBuf> {
    let mut repos = Vec::new();

    if !root.exists() {
        return repos;
    }

    let mut it = WalkDir::new(&root).into_iter();

    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(e)) => e,
        };

        if is_hidden(&entry) {
            if entry.file_type().is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        if entry.file_type().is_dir() && entry.path().join(".git").exists() {
            repos.push(entry.path().to_path_buf());
            it.skip_current_dir();
        }
    }
    repos
}

pub fn scan_repositories(root: PathBuf) -> Result<Vec<PathBuf>> {
    if root.join(".git").exists() {
        if let Ok(abs) = fs::canonicalize(&root) {
            return Ok(vec![abs]);
        }
        return Ok(vec![root]);
    }

    let entries: Vec<PathBuf> = fs::read_dir(&root)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();

    let repos: Vec<PathBuf> = entries
        .par_iter()
        .map(|path| scan_worker(path.clone()))
        .flatten()
        .collect();

    Ok(repos)
}
