use crate::core::terrain::TerrainGrid;
use indicatif::ProgressBar;
use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSlice;

pub fn compute_elevation(grid: &mut TerrainGrid, bar: &ProgressBar) {
    let (min, max) = grid
        .elevation
        .par_chunks(grid.width)
        .map(|chunk| {
            bar.inc(1);
            chunk
                .iter()
                .fold((f32::MAX, f32::MIN), |(acc_min, acc_max), &val| {
                    if val.is_nan() {
                        (acc_min, acc_max)
                    } else {
                        (acc_min.min(val), acc_max.max(val))
                    }
                })
        })
        .reduce(
            || (f32::MAX, f32::MIN),
            |(min1, max1), (min2, max2)| (min1.min(min2), max1.max(max2)),
        );

    if min != f32::MAX {
        grid.min_elevation = min;
        grid.max_elevation = max;
    }
}
