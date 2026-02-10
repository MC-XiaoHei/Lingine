use super::geo_utils::extract_bounds_async;
use super::types::AlosTile;
use crate::scanner::path_utils::{get_entries, normalize_path};
use crate::scanner::task_utils::run_all;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn scan(root: PathBuf) -> Result<Vec<AlosTile>> {
    if !root.exists() {
        return Ok(vec![]);
    }

    let tasks: Vec<_> = get_entries(&root)
        .await?
        .into_iter()
        .map(|path| tokio::spawn(async move { try_load_scene(path).await }))
        .collect();

    run_all(tasks).await
}

async fn try_load_scene(path: PathBuf) -> Result<Option<AlosTile>> {
    let id = path
        .file_name()
        .ok_or(anyhow!("Cannot get file name of {}", normalize_path(&path)))?
        .to_string_lossy()
        .to_string();
    let files = AlosFiles::new(&path, &id);

    if !files.all_exist().await {
        return Ok(None);
    }

    let bounds = extract_bounds_async(&files.dem).await?;

    Ok(Some(AlosTile {
        id,
        bounds,
        path_dem: files.dem,
        path_hh: files.hh,
        path_hv: files.hv,
        path_inc: files.inc,
        path_ls: files.ls,
    }))
}

struct AlosFiles {
    dem: PathBuf,
    hh: PathBuf,
    hv: PathBuf,
    inc: PathBuf,
    ls: PathBuf,
}

impl AlosFiles {
    fn new(p: &Path, id: &str) -> Self {
        Self {
            dem: p.join(format!("{}.dem.tif", id)),
            hh: p.join(format!("{}_HH.tif", id)),
            hv: p.join(format!("{}_HV.tif", id)),
            inc: p.join(format!("{}.inc_map.tif", id)),
            ls: p.join(format!("{}.ls_map.tif", id)),
        }
    }
    async fn all_exist(&self) -> bool {
        fs::metadata(&self.dem).await.is_ok()
            && fs::metadata(&self.hh).await.is_ok()
            && fs::metadata(&self.hv).await.is_ok()
            && fs::metadata(&self.inc).await.is_ok()
            && fs::metadata(&self.ls).await.is_ok()
    }
}
