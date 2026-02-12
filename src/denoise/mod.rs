mod median;

use crate::core::terrain::TerrainGrid;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use median::apply_median;

pub fn denoise(grid: &mut TerrainGrid) -> Result<()> {
    let h = grid.height;
    let bar = create_progress_bar((h * 4) as u64, "Layers Denoise");

    apply_median(&mut grid.sand, grid.width, grid.height, &bar);
    apply_median(&mut grid.clay, grid.width, grid.height, &bar);
    apply_median(&mut grid.soc, grid.width, grid.height, &bar);
    apply_median(&mut grid.ph, grid.width, grid.height, &bar);

    bar.finish();
    Ok(())
}
