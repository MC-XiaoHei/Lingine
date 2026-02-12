use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn apply_median(data: &mut Vec<Option<f32>>, width: usize, height: usize, bar: &ProgressBar) {
    let source = data.clone();

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

                let v_c  = source[curr_row + x];
                let v_u  = source[prev_row + x];
                let v_d  = source[next_row + x];
                let v_l  = source[curr_row + x - 1];
                let v_r  = source[curr_row + x + 1];
                let v_ul = source[prev_row + x - 1];
                let v_ur = source[prev_row + x + 1];
                let v_dl = source[next_row + x - 1];
                let v_dr = source[next_row + x + 1];

                let mut sum = 0.0;
                let mut total_weight = 0.0;

                macro_rules! acc {
                    ($val:expr, $w:expr) => {
                        if let Some(v) = $val {
                            sum += v * $w;
                            total_weight += $w;
                        }
                    };
                }

                acc!(v_c,  4.0);
                acc!(v_u,  2.0); acc!(v_d,  2.0); acc!(v_l,  2.0); acc!(v_r,  2.0);
                acc!(v_ul, 1.0); acc!(v_ur, 1.0); acc!(v_dl, 1.0); acc!(v_dr, 1.0);

                if total_weight > 0.0 {
                    row_slice[x] = Some(sum / total_weight);
                }
            }

            bar.inc(1);
        });
}