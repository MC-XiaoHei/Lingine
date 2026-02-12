use super::mosaic::MosaicSource;
use crate::scanner::{AlosTile, DataCatalog, EsaTile, SoilTile};
use geo::Rect;
use std::path::PathBuf;

pub struct LayerBundle {
    pub elevation: MosaicSource,
    pub hh: MosaicSource,
    pub hv: MosaicSource,
    pub inc: MosaicSource,
    pub ls: MosaicSource,

    pub landcover: MosaicSource,

    pub sand: MosaicSource,
    pub sand_sub: MosaicSource,
    pub clay: MosaicSource,
    pub clay_sub: MosaicSource,
    pub ph: MosaicSource,
    pub ph_sub: MosaicSource,
    pub soc: MosaicSource,
}

impl LayerBundle {
    pub fn from_catalog(catalog: &DataCatalog) -> Self {
        Self {
            elevation: MosaicSource::new(map_alos(&catalog.alos, |t| &t.path_dem)),
            hh: MosaicSource::new(map_alos(&catalog.alos, |t| &t.path_hh)),
            hv: MosaicSource::new(map_alos(&catalog.alos, |t| &t.path_hv)),
            inc: MosaicSource::new(map_alos(&catalog.alos, |t| &t.path_inc)),
            ls: MosaicSource::new(map_alos(&catalog.alos, |t| &t.path_ls)),

            landcover: MosaicSource::new(map_esa(&catalog.esa, |t| &t.path_map)),

            sand: MosaicSource::new(map_soil(&catalog.soil, |t| &t.sand_top)),
            sand_sub: MosaicSource::new(map_soil(&catalog.soil, |t| &t.sand_sub)),
            clay: MosaicSource::new(map_soil(&catalog.soil, |t| &t.clay_top)),
            clay_sub: MosaicSource::new(map_soil(&catalog.soil, |t| &t.clay_sub)),
            ph: MosaicSource::new(map_soil(&catalog.soil, |t| &t.ph_top)),
            ph_sub: MosaicSource::new(map_soil(&catalog.soil, |t| &t.ph_sub)),
            soc: MosaicSource::new(map_soil(&catalog.soil, |t| &t.soc_top)),
        }
    }
}

fn map_alos<F>(tiles: &[AlosTile], selector: F) -> Vec<(String, Rect<f64>, PathBuf)>
where
    F: Fn(&AlosTile) -> &PathBuf,
{
    tiles
        .iter()
        .map(|t| (t.id.clone(), t.bounds, selector(t).clone()))
        .collect()
}

fn map_esa<F>(tiles: &[EsaTile], selector: F) -> Vec<(String, Rect<f64>, PathBuf)>
where
    F: Fn(&EsaTile) -> &PathBuf,
{
    tiles
        .iter()
        .map(|t| (t.id.clone(), t.bounds, selector(t).clone()))
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
