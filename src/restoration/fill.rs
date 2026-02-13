use crate::restoration::UNIT_LEN;
use crate::utils::float::FloatEx;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
// =================================================================================
// Public Interfaces
// =================================================================================

pub fn fill_voids_continuous(
    data: &mut Vec<f32>,
    width: usize,
    height: usize,
    iters: u64,
    bar: &ProgressBar,
) {
    // 定义 Continuous (f32) 的行为闭包
    // 1. 有效性检查：不是 NAN
    let is_valid = |v: &f32| v.is_not_nan();

    // 2. 无效值定义：NAN
    let invalid_val = || f32::NAN;

    // 3. 聚合/缩减策略：平均值
    let reduce_avg = |vals: &[f32]| {
        let sum: f32 = vals.iter().sum();
        sum / vals.len() as f32
    };

    // 4. 插值策略：双线性插值
    let interp_bilinear = |src: &[f32], sw: usize, sh: usize, x: usize, y: usize| {
        // 将目标坐标映射回源坐标系
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
                if sx >= 0 && sx < sw as isize && sy >= 0 && sy < sh as isize {
                    let idx = (sy as usize) * sw + (sx as usize);
                    let val = src[idx];
                    if val.is_not_nan() {
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
            sum / weight_sum
        } else {
            f32::NAN
        }
    };

    // 执行通用管线
    fill_voids_pipeline(
        data,
        width,
        height,
        iters,
        bar,
        is_valid,
        invalid_val,
        reduce_avg,
        interp_bilinear,
    );
}

pub fn fill_voids_discrete(
    data: &mut Vec<Option<u8>>,
    width: usize,
    height: usize,
    iters: u64,
    bar: &ProgressBar,
) {
    // 定义 Discrete (Option<u8>) 的行为闭包
    // 1. 有效性检查：是 Some
    let is_valid = |v: &Option<u8>| v.is_some();

    // 2. 无效值定义：None
    let invalid_val = || None;

    // 3. 聚合/缩减策略：众数 (Mode)
    let strategy_mode = |vals: &[Option<u8>]| {
        let mut counts = HashMap::with_capacity(vals.len());
        for v in vals {
            if let Some(inner) = v {
                *counts.entry(*inner).or_insert(0) += 1;
            }
        }
        counts
            .into_iter()
            .max_by_key(|&(_, c)| c)
            .map(|(v, _)| Some(v))
            .unwrap_or(None)
    };

    // 4. 插值策略：最近邻 (直接取对应坐标)
    let interp_nearest = |src: &[Option<u8>], sw: usize, _sh: usize, x: usize, y: usize| {
        let sx = x / 2;
        let sy = y / 2;
        src.get(sy * sw + sx).copied().flatten()
    };

    // 执行通用管线
    fill_voids_pipeline(
        data,
        width,
        height,
        iters,
        bar,
        is_valid,
        invalid_val,
        strategy_mode,
        interp_nearest,
    );
}

// =================================================================================
// Generic Pipeline Skeleton
// =================================================================================

#[allow(clippy::too_many_arguments)]
fn fill_voids_pipeline<T, FValid, FInvalid, FReduce, FInterp>(
    data: &mut Vec<T>,
    width: usize,
    height: usize,
    iters: u64,
    bar: &ProgressBar,
    is_valid: FValid,
    invalid_val: FInvalid,
    reducer: FReduce,
    interpolator: FInterp,
) where
    T: Copy + Send + Sync + PartialEq,
    FValid: Fn(&T) -> bool + Sync + Send + Copy,
    FInvalid: Fn() -> T + Sync + Send + Copy,
    FReduce: Fn(&[T]) -> T + Sync + Send + Copy,
    FInterp: Fn(&[T], usize, usize, usize, usize) -> T + Sync + Send + Copy,
{
    // 递归终止条件
    if width < UNIT_LEN || height < UNIT_LEN {
        iterate_core(
            data,
            width,
            height,
            UNIT_LEN as u64,
            bar,
            is_valid,
            invalid_val,
            reducer,
        );
        return;
    }

    let (small_w, small_h) = (width.div_ceil(2), height.div_ceil(2));

    // 1. 下采样 (Generic Downsample)
    let mut small_data = downsample_generic(
        data,
        width,
        height,
        small_w,
        small_h,
        bar,
        is_valid,
        invalid_val,
        reducer,
    );

    // 2. 递归处理
    fill_voids_pipeline(
        &mut small_data,
        small_w,
        small_h,
        iters,
        bar,
        is_valid,
        invalid_val,
        reducer,
        interpolator,
    );

    // 3. 上采样合并 (Generic Upsample Merge)
    upsample_merge_generic(
        data,
        width,
        height,
        &small_data,
        small_w,
        small_h,
        bar,
        is_valid,
        interpolator,
    );

    // 4. 迭代修复 (Generic Iterate Core)
    iterate_core(
        data,
        width,
        height,
        iters,
        bar,
        is_valid,
        invalid_val,
        reducer,
    );
}

// =================================================================================
// Generic Algorithm Implementations
// =================================================================================

fn downsample_generic<T, FValid, FInvalid, FReduce>(
    src: &[T],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    bar: &ProgressBar,
    is_valid: FValid,
    invalid_val: FInvalid,
    reducer: FReduce,
) -> Vec<T>
where
    T: Copy + Send + Sync,
    FValid: Fn(&T) -> bool + Sync + Send,
    FInvalid: Fn() -> T + Sync + Send,
    FReduce: Fn(&[T]) -> T + Sync + Send,
{
    let mut dst = vec![invalid_val(); dst_w * dst_h];

    dst.par_chunks_mut(dst_w).enumerate().for_each(|(y, row)| {
        let mut neighbors = Vec::with_capacity(4);
        for (x, pixel) in row.iter_mut().enumerate() {
            let sx = x * 2;
            let sy = y * 2;
            neighbors.clear();

            for dy in 0..2 {
                for dx in 0..2 {
                    if sx + dx < src_w && sy + dy < src_h {
                        let val = src[(sy + dy) * src_w + (sx + dx)];
                        if is_valid(&val) {
                            neighbors.push(val);
                        }
                    }
                }
            }

            if !neighbors.is_empty() {
                *pixel = reducer(&neighbors);
            }
        }
        bar.inc(1);
    });

    dst
}

fn upsample_merge_generic<T, FValid, FInterp>(
    target: &mut [T],
    t_w: usize,
    _t_h: usize,
    source: &[T],
    s_w: usize,
    s_h: usize,
    bar: &ProgressBar,
    is_valid: FValid,
    interpolator: FInterp,
) where
    T: Copy + Send + Sync,
    FValid: Fn(&T) -> bool + Sync + Send,
    FInterp: Fn(&[T], usize, usize, usize, usize) -> T + Sync + Send,
{
    target.par_chunks_mut(t_w).enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            // 如果目标点已经有效，跳过
            if is_valid(pixel) {
                continue;
            }

            // 使用插值函数计算新值
            let val = interpolator(source, s_w, s_h, x, y);

            // 只有当插值结果有效时才写入（避免覆盖为无效值，虽然逻辑上不应该发生）
            if is_valid(&val) {
                *pixel = val;
            }
        }
        bar.inc(1);
    });
}

fn iterate_core<T, FValid, FInvalid, FStrategy>(
    data: &mut Vec<T>,
    width: usize,
    height: usize,
    max_iterations: u64,
    bar: &ProgressBar,
    is_valid: FValid,
    invalid_val: FInvalid,
    strategy: FStrategy,
) where
    T: Copy + Send + Sync + PartialEq,
    FValid: Fn(&T) -> bool + Sync + Send,
    FInvalid: Fn() -> T + Sync + Send,
    FStrategy: Fn(&[T]) -> T + Sync + Send,
{
    let mut aux = data.clone();

    for i in 0..max_iterations {
        let changed = AtomicBool::new(false);

        aux.par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, write_row)| {
                let read_source = &data;
                let mut neighbors = Vec::with_capacity(8);

                for (x, target_cell) in write_row.iter_mut().enumerate() {
                    let current_idx = y * width + x;

                    // 如果原数据有效，直接复制保持
                    if is_valid(&read_source[current_idx]) {
                        *target_cell = read_source[current_idx];
                        continue;
                    }

                    // 收集有效邻居
                    neighbors.clear();
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let ny = y as isize + dy;
                            let nx = x as isize + dx;
                            if ny >= 0 && ny < height as isize && nx >= 0 && nx < width as isize {
                                let idx = (ny as usize) * width + (nx as usize);
                                let val = read_source[idx];
                                if is_valid(&val) {
                                    neighbors.push(val);
                                }
                            }
                        }
                    }

                    if !neighbors.is_empty() {
                        let new_val = strategy(&neighbors);
                        if is_valid(&new_val) {
                            *target_cell = new_val;
                            changed.store(true, Ordering::Relaxed);
                            continue;
                        }
                    }

                    *target_cell = invalid_val();
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
