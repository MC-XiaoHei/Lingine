use indicatif::ProgressBar;
use rayon::prelude::*;

pub fn apply_median(
    data: &mut Vec<Option<f32>>,
    width: usize,
    height: usize,
    bar: &ProgressBar
) {
    let source = data.clone();

    data.par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row_slice)| {
            if y == 0 || y == height - 1 {
                bar.inc(1);
                return;
            }

            let prev_row_start = (y - 1) * width;
            let curr_row_start = y * width;
            let next_row_start = (y + 1) * width;

            for x in 1..width - 1 {
                let mut window = [0.0; 9];
                let mut count = 0;

                let mut push = |idx: usize| {
                    if let Some(val) = source[idx] {
                        window[count] = val;
                        count += 1;
                    }
                };

                push(prev_row_start + x - 1);
                push(prev_row_start + x);
                push(prev_row_start + x + 1);

                push(curr_row_start + x - 1);
                push(curr_row_start + x);
                push(curr_row_start + x + 1);

                push(next_row_start + x - 1);
                push(next_row_start + x);
                push(next_row_start + x + 1);

                if count > 0 {
                    let valid_slice = &mut window[0..count];
                    insertion_sort(valid_slice);
                    row_slice[x] = Some(valid_slice[count / 2]);
                } else {
                    row_slice[x] = None;
                }
            }

            bar.inc(1);
        });
}

#[inline(always)]
fn insertion_sort(arr: &mut [f32]) {
    for i in 1..arr.len() {
        let x = arr[i];
        let mut j = i;

        while j > 0 && arr[j - 1].total_cmp(&x).is_gt() {
            arr[j] = arr[j - 1];
            j -= 1;
        }
        arr[j] = x;
    }
}