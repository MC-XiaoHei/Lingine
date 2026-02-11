use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn get_entries(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut dir = fs::read_dir(path).await?;
    let mut entries = Vec::new();
    while let Some(entry) = dir.next_entry().await? {
        if entry.path().is_dir() {
            entries.push(entry.path());
        }
    }
    Ok(entries)
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
