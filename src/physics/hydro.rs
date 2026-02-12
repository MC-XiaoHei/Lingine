use crate::core::terrain::TerrainGrid;
use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn calc_flow_accumulation(grid: &TerrainGrid, bar: &ProgressBar) -> Vec<f32> {
    let w = grid.width;
    let h = grid.height;
    let len = w * h;

    bar.set_message("Hydro: Filtering indices...");

    let mut indices: Vec<usize> = (0..len)
        .into_par_iter()
        .filter(|&i| grid.elevation[i].is_some())
        .collect();

    bar.set_message("Hydro: Sorting indices...");

    indices.par_sort_unstable_by(|&a, &b| {
        let ha = grid.elevation[a].unwrap();
        let hb = grid.elevation[b].unwrap();
        hb.total_cmp(&ha)
    });

    bar.inc((len / 2) as u64);
    bar.set_message("Hydro: Accumulating flow...");

    let mut flow_acc = vec![1.0f32; len];
    let report_step = (indices.len() / 1000).max(1);

    for (i, &idx) in indices.iter().enumerate() {
        if i % report_step == 0 {
            bar.inc(report_step as u64);
        }

        let x = idx % w;
        let y = idx / w;

        if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
            continue;
        }

        let z = grid.elevation[idx].unwrap();
        let mut min_z = z;
        let mut target_idx = None;

        let offsets = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for (dx, dy) in offsets {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;
            let n_idx = ny * w + nx;

            if let Some(nz) = grid.elevation[n_idx] {
                if nz < min_z {
                    min_z = nz;
                    target_idx = Some(n_idx);
                }
            }
        }

        if let Some(target) = target_idx {
            flow_acc[target] += flow_acc[idx];
        }
    }

    flow_acc
}

pub fn calc_twi_final(flow_acc: &[f32], slope: &[f32], bar: &ProgressBar) -> Vec<f32> {
    bar.set_message("Hydro: Calculating TWI...");

    let twi: Vec<f32> = flow_acc
        .par_iter()
        .zip(slope.par_iter())
        .map(|(&a, &b)| {
            let tan_b = b.tan().max(0.001);
            let val = (a / tan_b).ln();
            val.max(0.0)
        })
        .collect();

    bar.finish();
    twi
}
