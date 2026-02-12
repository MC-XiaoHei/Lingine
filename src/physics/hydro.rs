use crate::core::terrain::TerrainGrid;
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
    let total_pixels = width * height;
    let update_frequency = width;

    (0..total_pixels).into_par_iter().map(|index| {
        if index % update_frequency == 0 {
            bar.inc(update_frequency as u64);
        }

        let current_elevation = match grid.elevation[index] {
            Some(v) => v,
            None => return None,
        };

        let x = index % width;
        let y = index / width;

        if is_border(x, y, width, height) {
            return None;
        }

        find_lowest_neighbor(grid, x, y, current_elevation)
    }).collect()
}

fn is_border(x: usize, y: usize, width: usize, height: usize) -> bool {
    x == 0 || x == width - 1 || y == 0 || y == height - 1
}

fn find_lowest_neighbor(grid: &TerrainGrid, x: usize, y: usize, current_elevation: f32) -> Option<u32> {
    let width = grid.width;
    let mut min_elevation = current_elevation;
    let mut target_index = None;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;
            let neighbor_index = ny * width + nx;

            if let Some(neighbor_elevation) = grid.elevation[neighbor_index] {
                if neighbor_elevation < min_elevation {
                    min_elevation = neighbor_elevation;
                    target_index = Some(neighbor_index as u32);
                }
            }
        }
    }
    target_index
}

fn compute_in_degree(downstream_map: &[Option<u32>], count: usize) -> Vec<AtomicU8> {
    let degrees: Vec<AtomicU8> = (0..count).into_par_iter()
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
    count: usize
) -> Vec<f32> {
    let mut accumulation = vec![1.0; count];
    let mut processing_stack = Vec::with_capacity(count / 10);

    for index in 0..count {
        if in_degree_map[index].load(Ordering::Relaxed) == 0 && grid.elevation[index].is_some() {
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