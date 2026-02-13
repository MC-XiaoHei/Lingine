use indicatif::ProgressBar;
use crate::core::terrain::TerrainGrid;
use noise::{NoiseFn, Perlin};
use rayon::prelude::*;

pub fn apply_fbm(grid: &mut TerrainGrid, bar: &ProgressBar) {
    const OCTAVES: u32 = 4;
    const PERSISTENCE: f64 = 0.5;
    const LACUNARITY: f64 = 2.0;
    const BASE_SCALE: f64 = 0.02;
    const BASE_AMPLITUDE: f64 = 1.5;
    const SEED: u32 = 2024;

    let perlin = Perlin::new(SEED);
    let width = grid.width;

    grid.elevation
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, h) in row.iter_mut().enumerate() {
                if h.is_nan() {
                    continue;
                }

                let nx = x as f64;
                let ny = y as f64;

                let mut amplitude = BASE_AMPLITUDE;
                let mut frequency = BASE_SCALE;
                let mut noise_acc = 0.0;

                for _ in 0..OCTAVES {
                    let n = perlin.get([nx * frequency, ny * frequency]);
                    noise_acc += n * amplitude;

                    amplitude *= PERSISTENCE;
                    frequency *= LACUNARITY;
                }

                *h += noise_acc as f32;
            }

            bar.inc(1);
        });
}