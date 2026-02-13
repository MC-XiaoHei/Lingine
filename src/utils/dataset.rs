use anyhow::Context;
use anyhow::Result;
use gdal::Dataset;
use std::path::Path;

pub trait DatasetEx {
    fn open_dataset(path: impl AsRef<Path>) -> Result<Dataset>;
}

impl DatasetEx for Dataset {
    fn open_dataset(path: impl AsRef<Path>) -> Result<Dataset> {
        let path = path.as_ref();
        Dataset::open(path).with_context(|| format!("Failed to open GeoTIFF: {path:?}"))
    }
}
