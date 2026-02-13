use crate::core::terrain::TerrainGrid;
use crate::utils::tap::PipeEx;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::f64::consts::PI;

pub fn calc_geometry(grid: &TerrainGrid, bar: &ProgressBar) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let w = grid.width;
    let h = grid.height;
    let len = w * h;

    type GeometryDataRow = (usize, Vec<f32>, Vec<f32>, Vec<f32>);

    let rows: Vec<GeometryDataRow> = (0..h)
        .into_par_iter()
        .map(|y| {
            let mut row_slope = vec![0.0; w];
            let mut row_aspect = vec![0.0; w];
            let mut row_tpi = vec![0.0; w];

            if y == 0 || y == h - 1 {
                bar.inc(w as u64);
                return (y, row_slope, row_aspect, row_tpi);
            }

            for x in 1..w - 1 {
                let idx = y * w + x;

                if grid.elevation[idx].is_nan() {
                    continue;
                }

                let center_z = grid.elevation[idx];
                let get = |dx: isize, dy: isize| -> f32 {
                    let nx = (x as isize + dx) as usize;
                    let ny = (y as isize + dy) as usize;
                    grid.elevation[ny * w + nx].pipe_when(|v| v.is_nan(), |_| center_z)
                };

                let z1 = get(-1, -1);
                let z2 = get(0, -1);
                let z3 = get(1, -1);
                let z4 = get(-1, 0);
                let z5 = center_z;
                let z6 = get(1, 0);
                let z7 = get(-1, 1);
                let z8 = get(0, 1);
                let z9 = get(1, 1);

                let cell_size = 1.0;

                let dz_dx = ((z3 + 2.0 * z6 + z9) - (z1 + 2.0 * z4 + z7)) / (8.0 * cell_size);
                let dz_dy = ((z7 + 2.0 * z8 + z9) - (z1 + 2.0 * z2 + z3)) / (8.0 * cell_size);

                let rise = (dz_dx * dz_dx + dz_dy * dz_dy).sqrt();
                row_slope[x] = rise.atan();

                let mut aspect = dz_dy.atan2(-dz_dx);
                if aspect < 0.0 {
                    aspect += 2.0 * PI as f32;
                }
                row_aspect[x] = aspect;

                let mean = (z1 + z2 + z3 + z4 + z6 + z7 + z8 + z9) / 8.0;
                row_tpi[x] = z5 - mean;
            }
            bar.inc(w as u64);
            (y, row_slope, row_aspect, row_tpi)
        })
        .collect();

    let mut slope = Vec::with_capacity(len);
    let mut aspect = Vec::with_capacity(len);
    let mut tpi = Vec::with_capacity(len);

    for (_, r_s, r_a, r_t) in rows {
        slope.extend(r_s);
        aspect.extend(r_a);
        tpi.extend(r_t);
    }

    (slope, aspect, tpi)
}
