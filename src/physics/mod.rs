pub mod climate;
pub mod geometry;
pub mod hydro;

use crate::core::terrain::TerrainGrid;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use indicatif::MultiProgress;

#[derive(Debug)]
pub struct PhysicsMap {
    pub slope: Vec<f32>,
    pub aspect: Vec<f32>,
    pub tpi: Vec<f32>,
    pub twi: Vec<f32>,
    pub hli: Vec<f32>,
}

pub fn physics_analyze(grid: &TerrainGrid) -> Result<PhysicsMap> {
    let multi_bar = MultiProgress::new();
    let total_pixels = (grid.width * grid.height) as u64;

    let bar_geom = multi_bar.add(create_progress_bar(
        total_pixels,
        "Geometry Analysis (Slope/Aspect/TPI)",
    ));

    let bar_hydro = multi_bar.add(create_progress_bar(
        total_pixels * 2,
        "Hydrology Simulation (Flow/Topology/TWI)",
    ));

    let bar_clim = multi_bar.add(create_progress_bar(total_pixels, "Climate Modeling (HLI)"));

    let ((slope, aspect, tpi, hli), flow_acc) = rayon::join(
        || {
            let (s, a, t) = geometry::calc_geometry(grid, &bar_geom);
            bar_geom.finish();

            let h = climate::calc_hli(grid, &s, &a, &bar_clim);
            bar_clim.finish();

            (s, a, t, h)
        },
        || {
            let acc = hydro::calc_flow_accumulation(grid, &bar_hydro);
            acc
        },
    );

    let twi = hydro::calc_twi_final(&flow_acc, &slope, &bar_hydro);
    bar_hydro.finish();

    Ok(PhysicsMap {
        slope,
        aspect,
        tpi,
        twi,
        hli,
    })
}
