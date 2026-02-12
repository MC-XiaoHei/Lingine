use crate::core::terrain::TerrainGrid;
use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn calc_hli(grid: &TerrainGrid, slope: &[f32], aspect: &[f32], bar: &ProgressBar) -> Vec<f32> {
    let w = grid.width;

    let hli: Vec<f32> = slope
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

            // Sun Vector: Azimuth = 225 deg (SW), Elevation = 45 deg
            let sun_azimuth = 225.0_f64.to_radians();
            let sun_elev = 45.0_f64.to_radians();

            let sx = sun_elev.cos() * sun_azimuth.sin();
            let sy = sun_elev.cos() * sun_azimuth.cos();
            let sz = sun_elev.sin();

            let nx = (s as f64).sin() * (a as f64).sin();
            let ny = (s as f64).sin() * (a as f64).cos();
            let nz = (s as f64).cos();

            let dot = sx * nx + sy * ny + sz * nz;

            let val = dot.max(0.0);
            val as f32
        })
        .collect();

    hli
}
