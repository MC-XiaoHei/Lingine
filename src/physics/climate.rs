use crate::core::context::SpatialContext;
use crate::core::terrain::TerrainGrid;
use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn calc_hli(
    grid: &TerrainGrid,
    slope: &[f32],
    aspect: &[f32],
    bar: &ProgressBar,
    ctx: &SpatialContext,
) -> Vec<f32> {
    let w = grid.width;

    let sun_azimuth = 225.0_f64.to_radians();
    let sun_elev = 45.0_f64.to_radians();

    let sx = sun_elev.cos() * sun_azimuth.sin();
    let sy = sun_elev.cos() * sun_azimuth.cos();
    let sz = sun_elev.sin();

    slope
        .par_iter()
        .zip(aspect.par_iter())
        .enumerate()
        .map(|(i, (&s, &a))| {
            if i % w == 0 {
                bar.inc(w as u64);
            }

            if s < 0.01 {
                return 0.5;
            }

            let x = i % w;
            let y = i / w;
            let geo = ctx.get_geo_coord(x, y);
            let gamma = ctx.ltm.convergence_angle(geo.x, geo.y) as f32;

            let corrected_aspect = a - gamma;

            let nx = (s as f64).sin() * (corrected_aspect as f64).sin();
            let ny = (s as f64).sin() * (corrected_aspect as f64).cos();
            let nz = (s as f64).cos();

            let dot = sx * nx + sy * ny + sz * nz;
            dot.max(0.0) as f32
        })
        .collect()
}
