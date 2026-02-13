use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn apply_median(data: &mut [Option<f32>], width: usize, height: usize, bar: &ProgressBar) {
    let source = data.to_owned();

    data.par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row_slice)| {
            if y == 0 || y == height - 1 {
                bar.inc(1);
                return;
            }

            let prev_row = (y - 1) * width;
            let curr_row = y * width;
            let next_row = (y + 1) * width;

            for x in 1..width - 1 {
                if source[curr_row + x].is_none() {
                    continue;
                }

                let mut neighbors = Vec::with_capacity(9);

                if let Some(v) = source[prev_row + x - 1] {
                    neighbors.push(v);
                }
                if let Some(v) = source[prev_row + x] {
                    neighbors.push(v);
                }
                if let Some(v) = source[prev_row + x + 1] {
                    neighbors.push(v);
                }

                if let Some(v) = source[curr_row + x - 1] {
                    neighbors.push(v);
                }
                if let Some(v) = source[curr_row + x] {
                    neighbors.push(v);
                }
                if let Some(v) = source[curr_row + x + 1] {
                    neighbors.push(v);
                }

                if let Some(v) = source[next_row + x - 1] {
                    neighbors.push(v);
                }
                if let Some(v) = source[next_row + x] {
                    neighbors.push(v);
                }
                if let Some(v) = source[next_row + x + 1] {
                    neighbors.push(v);
                }

                if !neighbors.is_empty() {
                    let mid = neighbors.len() / 2;
                    neighbors.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));
                    row_slice[x] = Some(neighbors[mid]);
                }
            }

            bar.inc(1);
        });
}
