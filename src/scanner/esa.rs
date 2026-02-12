use super::geo_utils::extract_bounds_async;
use super::types::EsaTile;
use crate::scanner::path_utils::get_entries;
use crate::scanner::task_utils::run_all;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn scan(root: PathBuf) -> Result<Vec<EsaTile>> {
    if !root.exists() {
        return Ok(vec![]);
    }

    let tasks: Vec<_> = get_entries(&root)
        .await?
        .into_iter()
        .map(|path| tokio::spawn(async move { try_load_tile(path).await }))
        .collect();

    run_all(tasks).await
}

async fn try_load_tile(path: PathBuf) -> Result<Option<EsaTile>> {
    let id = get_id(&path)?;
    let map_file = path.join(get_map_file_name(&id));
    ensure_file_exists(&map_file).await?;
    let quality_file = path.join(get_input_quality_file_name(&id));
    ensure_file_exists(&quality_file).await?;
    let bounds = extract_bounds_async(&map_file).await?;

    Ok(Some(EsaTile {
        id,
        bounds,
        path_map: map_file,
    }))
}

fn get_id(path: &Path) -> Result<String> {
    let file_name = path
        .file_name()
        .ok_or(anyhow!("Failed to get file name for {:?}", path))?;
    Ok(file_name.to_string_lossy().to_string())
}

fn get_map_file_name(id: &str) -> String {
    format!("{}.tif", id)
}

fn get_input_quality_file_name(id: &str) -> String {
    id.replace("_Map", "_InputQuality.tif")
}

async fn ensure_file_exists(path: &Path) -> Result<()> {
    if fs::try_exists(path).await.is_err() {
        return Err(anyhow!("File {:?} not exists", path));
    }
    if let Ok(metadata) = fs::metadata(path).await {
        if metadata.is_file() {
            Ok(())
        } else {
            Err(anyhow!("File {:?} is not a file", path))
        }
    } else {
        Err(anyhow!("File {:?} not exists", path))
    }
}
