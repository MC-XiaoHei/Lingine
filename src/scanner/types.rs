use geo::Rect;
use std::path::PathBuf;

pub struct DataCatalog {
    pub alos: Vec<AlosTile>,
    pub esa: Vec<EsaTile>,
    pub soil: Vec<SoilTile>,
}

#[derive(Debug, Clone)]
pub struct AlosTile {
    pub id: String,
    pub bounds: Rect<f64>,

    pub path_dem: PathBuf,
    pub path_hh: PathBuf,
    pub path_hv: PathBuf,
    pub path_inc: PathBuf,
    pub path_ls: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EsaTile {
    pub id: String,
    pub bounds: Rect<f64>,

    pub path_map: PathBuf,
    pub path_quality: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SoilTile {
    pub id: String,
    pub bounds: Rect<f64>,

    pub sand_top: PathBuf,
    pub sand_sub: PathBuf,
    pub clay_top: PathBuf,
    pub clay_sub: PathBuf,
    pub soc_top: PathBuf,
    pub ph_top: PathBuf,
    pub ph_sub: PathBuf,
}

#[derive(Debug)]
pub enum CoverageResult {
    Full,
    Partial {
        alos_coverage: f64,
        esa_coverage: f64,
        soil_coverage: f64,
    },
}

#[derive(Debug)]
pub struct ValidationReport {
    pub is_ready: bool,
    pub details: Vec<String>,
}
