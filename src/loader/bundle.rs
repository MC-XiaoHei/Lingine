use super::mosaic::MosaicSource;
use crate::scanner::{AlosTile, DataCatalog, EsaTile, SoilTile};
use geo::Rect;
use std::path::PathBuf;

pub struct LayerBundle {
    pub elevation: MosaicSource,
    pub landcover: MosaicSource,
    pub sand: MosaicSource,
    pub clay: MosaicSource,
    pub soc: MosaicSource,
    pub ph: MosaicSource,
}

impl LayerBundle {
    pub fn from_catalog(catalog: &DataCatalog) -> Self {
        Self {
            elevation: MosaicSource::new(map_alos(&catalog.alos)),
            landcover: MosaicSource::new(map_esa(&catalog.esa)),
            sand: MosaicSource::new(map_soil(&catalog.soil, |t| &t.sand_top)),
            clay: MosaicSource::new(map_soil(&catalog.soil, |t| &t.clay_top)),
            soc: MosaicSource::new(map_soil(&catalog.soil, |t| &t.soc_top)),
            ph: MosaicSource::new(map_soil(&catalog.soil, |t| &t.ph_top)),
        }
    }
}

fn map_alos(tiles: &[AlosTile]) -> Vec<(String, Rect<f64>, PathBuf)> {
    tiles
        .iter()
        .map(|t| (t.id.clone(), t.bounds, t.path_dem.clone()))
        .collect()
}

fn map_esa(tiles: &[EsaTile]) -> Vec<(String, Rect<f64>, PathBuf)> {
    tiles
        .iter()
        .map(|t| (t.id.clone(), t.bounds, t.path_map.clone()))
        .collect()
}

fn map_soil<F>(tiles: &[SoilTile], selector: F) -> Vec<(String, Rect<f64>, PathBuf)>
where
    F: Fn(&SoilTile) -> &PathBuf,
{
    tiles
        .iter()
        .map(|t| (t.id.clone(), t.bounds, selector(t).clone()))
        .collect()
}
