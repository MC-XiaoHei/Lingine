mod alos;
mod catalog;
mod esa;
mod geo_utils;
mod path_utils;
mod soil;
mod task_utils;
pub mod types;

use crate::scanner::types::DataCatalog;
use anyhow::Result;
use std::path::PathBuf;

const DATASETS_PATH: &str = "datasets";

pub async fn scan_datasets() -> Result<DataCatalog> {
    DataCatalog::scan(PathBuf::from(DATASETS_PATH)).await
}
