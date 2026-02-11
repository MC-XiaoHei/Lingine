mod bundle;
pub mod mosaic;
mod reader;

use crate::scanner::DataCatalog;
use anyhow::Result;
pub use bundle::LayerBundle;

pub fn load_assets(catalog: &DataCatalog) -> Result<LayerBundle> {
    let layers = LayerBundle::from_catalog(catalog);
    Ok(layers)
}