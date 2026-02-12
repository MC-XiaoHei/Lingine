use geo::Rect;
use std::fmt;
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

impl CoverageResult {
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }
}

impl fmt::Display for CoverageResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "All datasets fully cover the ROI."),
            Self::Partial {
                alos_coverage,
                esa_coverage,
                soil_coverage,
            } => {
                writeln!(
                    f,
                    "Dataset insufficient, the current datasets does not cover the request ROI"
                )?;
                writeln!(f, "Coverages are:")?;
                writeln!(f, "Alos Palsar: {:.2}%", alos_coverage * 100.0)?;
                writeln!(f, "Esa WorldCover: {:.2}%", esa_coverage * 100.0)?;
                write!(f, "Soil Grids: {:.2}%", soil_coverage * 100.0)
            }
        }
    }
}
