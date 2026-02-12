pub mod bundle;
pub mod mosaic;
mod reader;

use anyhow::Result;
use crate::loader::bundle::LayerBundle;
use crate::scanner::types::DataCatalog;

pub fn load_layers(catalog: &DataCatalog) -> Result<LayerBundle> {
    let layers = LayerBundle::from_catalog(catalog);
    Ok(layers)
}
