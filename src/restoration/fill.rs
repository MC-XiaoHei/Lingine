use indicatif::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

const UNIT_LEN: usize = 256;

pub fn fill_voids_continuous(
    data: &mut Vec<Option<f32>>,
    width: usize,
    height: usize,
    iters: u64,
    bar: &ProgressBar,
) {
    if width < UNIT_LEN || height < UNIT_LEN {
        fill_voids_core(data, width, height, UNIT_LEN as u64, bar, continuous_strategy);
        return;
    }

    let (small_w, small_h) = ((width + 1) / 2, (height + 1) / 2);
    let mut small_data = downsample_continuous(data, width, height, small_w, small_h, bar);

    fill_voids_continuous(&mut small_data, small_w, small_h, iters, bar);

    upsample_merge_continuous_bilinear(data, width, height, &small_data, small_w, small_h, bar);

    fill_voids_core(data, width, height, iters, bar, continuous_strategy);
}

fn downsample_continuous(
    src: &[Option<f32>],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    bar: &ProgressBar,
) -> Vec<Option<f32>> {
    let mut dst = vec![None; dst_w * dst_h];
    dst.par_chunks_mut(dst_w).enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            let sx = x * 2;
            let sy = y * 2;
            let mut sum = 0.0;
            let mut count = 0;
            for dy in 0..2 {
                for dx in 0..2 {
                    if sx + dx < src_w && sy + dy < src_h {
                        if let Some(v) = src[(sy + dy) * src_w + (sx + dx)] {
                            sum += v;
                            count += 1;
                        }
                    }
                }
            }
            if count > 0 {
                *pixel = Some(sum / count as f32);
            }
        }
        bar.inc(1);
    });
    dst
}

fn upsample_merge_continuous_bilinear(
    target: &mut [Option<f32>],
    t_w: usize,
    _t_h: usize,
    source: &[Option<f32>],
    s_w: usize,
    s_h: usize,
    bar: &ProgressBar,
) {
    target.par_chunks_mut(t_w).enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            if pixel.is_some() { continue; }

            let src_x = (x as f32 + 0.5) / 2.0 - 0.5;
            let src_y = (y as f32 + 0.5) / 2.0 - 0.5;
            let x0 = src_x.floor() as isize;
            let y0 = src_y.floor() as isize;
            let wx = src_x - x0 as f32;
            let wy = src_y - y0 as f32;

            let mut sum = 0.0;
            let mut weight_sum = 0.0;

            for dy in 0..=1 {
                for dx in 0..=1 {
                    let sy = y0 + dy;
                    let sx = x0 + dx;
                    if sx >= 0 && sx < s_w as isize && sy >= 0 && sy < s_h as isize {
                        let idx = (sy as usize) * s_w + (sx as usize);
                        if let Some(val) = source[idx] {
                            let w_x = if dx == 0 { 1.0 - wx } else { wx };
                            let w_y = if dy == 0 { 1.0 - wy } else { wy };
                            let w = w_x * w_y;
                            sum += val * w;
                            weight_sum += w;
                        }
                    }
                }
            }
            if weight_sum > 0.0 {
                *pixel = Some(sum / weight_sum);
            }
        }
        bar.inc(1);
    });
}

fn continuous_strategy(neighbors: &[f32]) -> Option<f32> {
    let sum: f32 = neighbors.iter().sum();
    let count = neighbors.len() as f32;
    if count > 0.0 { Some(sum / count) } else { None }
}

pub fn fill_voids_discrete(
    data: &mut Vec<Option<u8>>,
    width: usize,
    height: usize,
    iters: u64,
    bar: &ProgressBar,
) {
    if width < UNIT_LEN || height < UNIT_LEN {
        fill_voids_core(data, width, height, UNIT_LEN as u64, bar, discrete_strategy);
        return;
    }
    let (small_w, small_h) = ((width + 1) / 2, (height + 1) / 2);
    let mut small_data = downsample_discrete(data, width, height, small_w, small_h, bar);

    fill_voids_discrete(&mut small_data, small_w, small_h, iters, bar);
    upsample_merge_discrete(data, width, height, &small_data, small_w, bar);
    fill_voids_core(data, width, height, iters, bar, discrete_strategy);
}

fn downsample_discrete(
    src: &[Option<u8>],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    bar: &ProgressBar,
) -> Vec<Option<u8>> {
    let mut dst = vec![None; dst_w * dst_h];
    dst.par_chunks_mut(dst_w).enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            let sx = x * 2;
            let sy = y * 2;
            let mut counts = HashMap::with_capacity(4);
            for dy in 0..2 {
                for dx in 0..2 {
                    if sx + dx < src_w && sy + dy < src_h {
                        if let Some(v) = src[(sy + dy) * src_w + (sx + dx)] {
                            *counts.entry(v).or_insert(0) += 1;
                        }
                    }
                }
            }
            *pixel = counts.into_iter().max_by_key(|&(_, c)| c).map(|(v, _)| v);
        }
        bar.inc(1);
    });
    dst
}

fn upsample_merge_discrete(
    target: &mut [Option<u8>],
    t_w: usize,
    _t_h: usize,
    source: &[Option<u8>],
    s_w: usize,
    bar: &ProgressBar,
) {
    target.par_chunks_mut(t_w).enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            if pixel.is_some() { continue; }
            let sx = x / 2;
            let sy = y / 2;
            if let Some(val) = source.get(sy * s_w + sx).copied().flatten() {
                *pixel = Some(val);
            }
        }
        bar.inc(1);
    });
}

fn discrete_strategy(neighbors: &[u8]) -> Option<u8> {
    let mut counts = HashMap::with_capacity(neighbors.len());
    for &val in neighbors { *counts.entry(val).or_insert(0) += 1; }
    counts.into_iter().max_by_key(|&(_, count)| count).map(|(val, _)| val)
}

fn fill_voids_core<T, F>(
    data: &mut Vec<Option<T>>,
    width: usize,
    height: usize,
    max_iterations: u64,
    bar: &ProgressBar,
    strategy: F,
) where
    T: Copy + Send + Sync + PartialEq,
    F: Fn(&[T]) -> Option<T> + Sync + Send,
{
    let mut aux = data.clone();

    for i in 0..max_iterations {
        let changed = AtomicBool::new(false);

        aux.par_chunks_mut(width).enumerate().for_each(|(y, write_row)| {
            let read_source = &data;
            for (x, target_cell) in write_row.iter_mut().enumerate() {
                let current_idx = y * width + x;
                if let Some(v) = read_source[current_idx] {
                    *target_cell = Some(v);
                    continue;
                }
                let mut neighbors = Vec::with_capacity(8);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let ny = y as isize + dy;
                        let nx = x as isize + dx;
                        if ny >= 0 && ny < height as isize && nx >= 0 && nx < width as isize {
                            let idx = (ny as usize) * width + (nx as usize);
                            if let Some(val) = read_source[idx] {
                                neighbors.push(val);
                            }
                        }
                    }
                }
                if !neighbors.is_empty() {
                    if let Some(new_val) = strategy(&neighbors) {
                        *target_cell = Some(new_val);
                        changed.store(true, Ordering::Relaxed);
                        continue;
                    }
                }
                *target_cell = None;
            }
            bar.inc(1);
        });

        std::mem::swap(data, &mut aux);

        if !changed.load(Ordering::Relaxed) {
            let remaining = max_iterations - 1 - i;
            if remaining > 0 {
                bar.inc(remaining * height as u64);
            }
            break;
        }
    }
}