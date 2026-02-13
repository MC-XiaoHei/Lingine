pub mod fill;
pub mod median;

use crate::core::terrain::TerrainGrid;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use fill::{fill_voids_continuous, fill_voids_discrete};
use median::apply_median;

const UNIT_LEN: usize = 256;

pub fn terrain_restoration(grid: &mut TerrainGrid) -> Result<()> {
    let h = grid.height;
    let w = grid.width;

    const ITERS_SMOOTH: u64 = 5;

    let count_continuous = get_continuous_layers(grid).len() as u64;
    let count_discrete = get_discrete_layers(grid).len() as u64;
    let count_median = get_median_layers(grid).len() as u64;

    let ticks_per_fill = calc_fill_ticks(w, h, ITERS_SMOOTH);
    let total_rows =
        (count_continuous + count_discrete) * ticks_per_fill + (count_median * h as u64);

    let bar = create_progress_bar(total_rows, "Terrain Restoration");

    let mut f32_aux_buffer = vec![f32::NAN; w * h];

    get_continuous_layers(grid).into_iter().for_each(|layer| {
        fill_voids_continuous(layer, &mut f32_aux_buffer, w, h, ITERS_SMOOTH, &bar);
    });

    let mut u8_aux_buffer = vec![Some(0u8); w * h];

    get_discrete_layers(grid).into_iter().for_each(|layer| {
        fill_voids_discrete(layer, &mut u8_aux_buffer, w, h, ITERS_SMOOTH, &bar);
    });

    drop(u8_aux_buffer);

    get_median_layers(grid).into_iter().for_each(|layer| {
        apply_median(layer, &mut f32_aux_buffer, w, h, &bar);
    });

    bar.finish();
    Ok(())
}

fn get_continuous_layers(g: &mut TerrainGrid) -> Vec<&mut Vec<f32>> {
    vec![
        &mut g.elevation,
        &mut g.hh,
        &mut g.hv,
        &mut g.inc,
        &mut g.ls,
        &mut g.sand,
        &mut g.clay,
        &mut g.soc,
        &mut g.ph,
        &mut g.sand_sub,
        &mut g.clay_sub,
        &mut g.ph_sub,
    ]
}

fn get_discrete_layers(g: &mut TerrainGrid) -> Vec<&mut Vec<Option<u8>>> {
    vec![&mut g.landcover]
}

fn get_median_layers(g: &mut TerrainGrid) -> Vec<&mut Vec<f32>> {
    vec![&mut g.sand, &mut g.clay, &mut g.soc, &mut g.ph]
}

fn calc_fill_ticks(w: usize, h: usize, iters: u64) -> u64 {
    if w < UNIT_LEN || h < UNIT_LEN {
        return (h as u64) * (UNIT_LEN as u64);
    }

    let next_w = w.div_ceil(2);
    let next_h = h.div_ceil(2);

    let mut ticks = 0;

    ticks += next_h as u64;
    ticks += calc_fill_ticks(next_w, next_h, iters);
    ticks += h as u64;
    ticks += (h as u64) * iters;

    ticks
}
