mod alos;
mod catalog;
mod esa;
mod geo_utils;
mod path_utils;
mod soil;
mod task_utils;
mod types;

use anyhow::Result;
use std::path::PathBuf;
pub use types::*;

const DATASETS_PATH: &str = "datasets";

pub async fn scan_datasets() -> Result<DataCatalog> {
    println!("Scanning Datasets...");
    DataCatalog::scan(PathBuf::from(DATASETS_PATH)).await
}
