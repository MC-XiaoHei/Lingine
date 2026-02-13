use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn apply_median(
    data: &mut [f32],
    aux: &mut [f32],
    width: usize,
    height: usize,
    bar: &ProgressBar,
) {
    aux.copy_from_slice(data);
    let source = &aux[..];

    data.par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row_slice)| {
            if y == 0 || y == height - 1 {
                if y == 0 { bar.inc(1); }
                return;
            }

            let prev_row_idx = (y - 1) * width;
            let curr_row_idx = y * width;
            let next_row_idx = (y + 1) * width;

            for x in 1..width - 1 {
                if source[curr_row_idx + x].is_nan() {
                    continue;
                }

                let mut window = [0.0; 9];
                let mut count = 0;

                let mut push = |idx: usize| {
                    let val = source[idx];
                    if !val.is_nan() {
                        window[count] = val;
                        count += 1;
                    }
                };

                let x_m = x - 1;
                let x_p = x + 1;

                push(prev_row_idx + x_m);
                push(prev_row_idx + x);
                push(prev_row_idx + x_p);

                push(curr_row_idx + x_m);
                push(curr_row_idx + x);
                push(curr_row_idx + x_p);

                push(next_row_idx + x_m);
                push(next_row_idx + x);
                push(next_row_idx + x_p);

                if count > 0 {
                    let valid_slice = &mut window[0..count];
                    insertion_sort(valid_slice);
                    row_slice[x] = valid_slice[count / 2];
                }
            }

            bar.inc(1);
        });
}

#[inline(always)]
fn insertion_sort(arr: &mut [f32]) {
    let len = arr.len();
    for i in 1..len {
        let key = arr[i];
        let mut j = i;

        while j > 0 && arr[j - 1] > key {
            arr[j] = arr[j - 1];
            j -= 1;
        }
        arr[j] = key;
    }
}