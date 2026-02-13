pub mod climate;
pub mod geometry;
pub mod hydro;

use crate::core::context::SpatialContext;
use crate::core::terrain::TerrainGrid;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use climate::calc_hli;
use geometry::calc_geometry;
use hydro::{calc_flow_accumulation, calc_twi_final};
use indicatif::MultiProgress;

#[derive(Debug)]
pub struct PhysicsMap {
    pub slope: Vec<f32>,
    pub aspect: Vec<f32>,
    pub tpi: Vec<f32>,
    pub twi: Vec<f32>,
    pub hli: Vec<f32>,
}

pub fn physics_analyze(grid: &TerrainGrid, ctx: &SpatialContext) -> Result<PhysicsMap> {
    let multi_bar = MultiProgress::new();
    let total_pixels = (grid.width * grid.height) as u64;

    let bar_geom = multi_bar.add(create_progress_bar(total_pixels, "Geometry Analysis"));

    let bar_hydro = multi_bar.add(create_progress_bar(
        total_pixels * 2,
        "Hydrology Simulation",
    ));

    let bar_clim = multi_bar.add(create_progress_bar(total_pixels, "Climate Modeling"));

    let ((slope, aspect, tpi, hli), flow_acc) = rayon::join(
        || {
            let (s, a, t) = calc_geometry(grid, &bar_geom);
            bar_geom.finish();

            let h = calc_hli(grid, &s, &a, &bar_clim, ctx);
            bar_clim.finish();

            (s, a, t, h)
        },
        || calc_flow_accumulation(grid, &bar_hydro),
    );

    let twi = calc_twi_final(&flow_acc, &slope, &bar_hydro);
    bar_hydro.finish();

    Ok(PhysicsMap {
        slope,
        aspect,
        tpi,
        twi,
        hli,
    })
}
