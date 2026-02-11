pub mod context;
pub mod projection;
mod sampler;

use crate::core::terrain::TerrainGrid;
use crate::loader::LayerBundle;
use crate::utils::progress::create_progress_bar;
use anyhow::Result;
use context::SpatialContext;
use geo::Rect;
use rayon::prelude::*;

pub fn create_and_align(assets: &LayerBundle, roi: Rect<f64>) -> Result<TerrainGrid> {
    let ctx = SpatialContext::analyze(roi);
    print_summary(&ctx);

    let mut grid = TerrainGrid::new(ctx.width, ctx.height);
    let bar = create_progress_bar(ctx.total_pixels, "Alignment & Resampling Layers");

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

    bar.finish_with_message("Layers Alignment & Resampling Complete");
    Ok(grid)
}

fn print_summary(ctx: &SpatialContext) {
    println!(
        "Physical Dimensions: {:.2}m x {:.2}m",
        ctx.roi_meters.width().abs(),
        ctx.roi_meters.height().abs()
    );
    println!("Grid Resolution: {} x {}", ctx.width, ctx.height);
    println!("Total Voxels: {}", ctx.total_pixels);
}