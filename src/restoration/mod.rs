pub mod fill;
pub mod median;

use crate::core::terrain::TerrainGrid;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use fill::{fill_voids_continuous, fill_voids_discrete};
use median::apply_median;

pub fn terrain_restoration(grid: &mut TerrainGrid) -> Result<()> {
    let h = grid.height;
    let w = grid.width;

    const COUNT_CONTINUOUS: u64 = 12;
    const COUNT_DISCRETE: u64 = 1;
    const COUNT_MEDIAN: u64 = 4;

    const PYRAMID_WORK_FACTOR: u64 = 14;
    const MEDIAN_WORK_FACTOR: u64 = 1;

    let total_rows = (COUNT_CONTINUOUS + COUNT_DISCRETE) * (h as u64 * PYRAMID_WORK_FACTOR)
        + COUNT_MEDIAN * (h as u64 * MEDIAN_WORK_FACTOR);

    let bar = create_progress_bar(total_rows, "Terrain Restoring");

    {
        let mut continuous_layers = vec![
            &mut grid.elevation,
            &mut grid.hh,
            &mut grid.hv,
            &mut grid.inc,
            &mut grid.ls,
            &mut grid.sand,
            &mut grid.clay,
            &mut grid.soc,
            &mut grid.ph,
            &mut grid.sand_sub,
            &mut grid.clay_sub,
            &mut grid.ph_sub,
        ];

        let mut discrete_layers = vec![&mut grid.landcover];

        continuous_layers.iter_mut().for_each(|layer| {
            fill_voids_continuous(layer, w, h, 5, &bar);
        });
        discrete_layers.iter_mut().for_each(|layer| {
            fill_voids_discrete(layer, w, h, 5, &bar);
        });
    }

    {
        let mut median_layers = vec![&mut grid.sand, &mut grid.clay, &mut grid.soc, &mut grid.ph];

        let process_median = |layer: &mut Vec<Option<f32>>| {
            apply_median(layer, w, h, &bar);
        };

        median_layers
            .iter_mut()
            .for_each(|layer| process_median(layer));
    }

    bar.finish();
    Ok(())
}
