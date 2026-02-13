mod sampler;

use crate::core::context::SpatialContext;
use crate::core::terrain::TerrainGrid;
use crate::loader::bundle::LayerBundle;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use rayon::prelude::*;

pub fn align_and_resample(assets: &LayerBundle, ctx: &SpatialContext) -> Result<TerrainGrid> {
    let mut grid = TerrainGrid::new(ctx.width, ctx.height);
    let bar = create_progress_bar(ctx.total_pixels, "Layers Alignment & Resample");

    grid.par_rows_mut().enumerate().for_each_init(
        || sampler::SamplingSession::new(assets),
        |session, (y, mut row)| {
            for x in 0..ctx.width {
                let geo = ctx.get_geo_coord(x, y);
                let pixel = session.sample(geo.x, geo.y);
                row.set(x, pixel);
            }
            bar.inc(ctx.width as u64);
        },
    );

    bar.finish();
    Ok(grid)
}
