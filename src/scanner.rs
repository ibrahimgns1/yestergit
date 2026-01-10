use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

///returns vector of paths that containt a .git folder.
///
pub fn scan_repos(root: PathBuf) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();

    for entry in WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            let path = entry.path();

            if path.join(".git").exists() {
                repos.push(path.to_path_buf());
            }
        }
    }
    Ok(repos)
}
