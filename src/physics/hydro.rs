use crate::core::terrain::TerrainGrid;
use crate::utils::float::FloatEx;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};

pub fn calc_flow_accumulation(grid: &TerrainGrid, bar: &ProgressBar) -> Vec<f32> {
    let width = grid.width;
    let height = grid.height;
    let total_pixels = width * height;

    let downstream_map = compute_downstream_map(grid, bar);
    let in_degree_map = compute_in_degree(&downstream_map, total_pixels);

    perform_topological_accumulation(&downstream_map, in_degree_map, grid, total_pixels)
}

fn compute_downstream_map(grid: &TerrainGrid, bar: &ProgressBar) -> Vec<Option<u32>> {
    let width = grid.width;
    let height = grid.height;

    let rows: Vec<Option<u32>> = (0..height)
        .into_par_iter()
        .map(|y| {
            if y % 100 == 0 {
                bar.inc(width as u64 * 100);
            }
            if y == 0 || y == height - 1 {
                return vec![None; width];
            }

            let mut row_result = Vec::with_capacity(width);

            row_result.push(None);

            for x in 1..width - 1 {
                let idx = y * width + x;
                let current_z = grid.elevation[idx];

                if current_z.is_nan() {
                    row_result.push(None);
                } else {
                    row_result.push(find_lowest_neighbor_unrolled(grid, x, y, current_z));
                }
            }

            row_result.push(None);

            row_result
        })
        .flatten()
        .collect();

    rows
}

#[inline(always)]
fn find_lowest_neighbor_unrolled(
    grid: &TerrainGrid,
    x: usize,
    y: usize,
    current_z: f32,
) -> Option<u32> {
    let w = grid.width;
    let idx = y * w + x;

    #[rustfmt::skip]
    let n_idxes = [
        idx - w - 1, idx - w, idx - w + 1,
        idx - 1,              idx + 1,
        idx + w - 1, idx + w, idx + w + 1,
    ];

    let mut min_z = current_z;
    let mut min_idx = u32::MAX;

    for &ni in &n_idxes {
        let nz = grid.elevation[ni];

        let mask = nz < min_z;

        if mask {
            min_z = nz;
            min_idx = ni as u32;
        }
    }

    if min_idx == u32::MAX {
        None
    } else {
        Some(min_idx)
    }
}

fn compute_in_degree(downstream_map: &[Option<u32>], count: usize) -> Vec<AtomicU8> {
    let degrees: Vec<AtomicU8> = (0..count)
        .into_par_iter()
        .map(|_| AtomicU8::new(0))
        .collect();

    downstream_map.par_iter().for_each(|target_opt| {
        if let Some(target) = target_opt {
            degrees[*target as usize].fetch_add(1, Ordering::Relaxed);
        }
    });

    degrees
}

fn perform_topological_accumulation(
    downstream_map: &[Option<u32>],
    in_degree_map: Vec<AtomicU8>,
    grid: &TerrainGrid,
    count: usize,
) -> Vec<f32> {
    let mut accumulation = vec![1.0; count];
    let mut processing_stack = Vec::with_capacity(count / 10);

    for (index, item) in in_degree_map.iter().enumerate().take(count) {
        if item.load(Ordering::Relaxed) == 0 && grid.elevation[index].is_not_nan() {
            processing_stack.push(index);
        }
    }

    while let Some(current_index) = processing_stack.pop() {
        if let Some(target) = downstream_map[current_index] {
            let target_index = target as usize;
            accumulation[target_index] += accumulation[current_index];

            if in_degree_map[target_index].fetch_sub(1, Ordering::Relaxed) == 1 {
                processing_stack.push(target_index);
            }
        }
    }

    accumulation
}

pub fn calc_twi_final(flow: &[f32], slope: &[f32], bar: &ProgressBar) -> Vec<f32> {
    let chunk_size = 10_000;

    flow.par_iter()
        .zip(slope.par_iter())
        .enumerate()
        .map(|(index, (flow_val, slope_val))| {
            if index % chunk_size == 0 {
                bar.inc(chunk_size as u64);
            }

            let tan_slope = slope_val.tan().max(0.001);
            (flow_val / tan_slope).ln().max(0.0)
        })
        .collect()
}
