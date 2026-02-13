pub mod bundle;
pub mod mosaic;
mod reader;

use crate::loader::bundle::LayerBundle;
use crate::scanner::types::DataCatalog;
use anyhow::Result;

pub fn load_layers(catalog: &DataCatalog) -> Result<LayerBundle> {
    let layers = LayerBundle::from_catalog(catalog);
    Ok(layers)
}
