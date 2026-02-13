mod alignment;
mod core;
mod loader;
mod physics;
mod restoration;
mod scanner;
mod utils;

use crate::core::validator::{validate_data_catalog, validate_terrain_grid};
use crate::scanner::scan_datasets;
use crate::utils::tap::{TryPipe, TryTap};
use alignment::align_and_resample;
use anyhow::Result;
use core::context::SpatialContext;
use geo::{Coord, Rect};
use loader::load_layers;
use physics::physics_analyze;
use restoration::terrain_restoration;
use tap::Tap;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run_pipeline().await {
        eprintln!("Error: {e}");
    }
    Ok(())
}

async fn run_pipeline() -> Result<()> {
    let roi = Rect::new(
        Coord {
            x: 94.02376,
            y: 30.15698,
        },
        Coord {
            x: 93.84993,
            y: 29.97956,
        },
    );

    let ctx = SpatialContext::analyze(roi).tap(|ctx| println!("{ctx}"));

    let terrain = scan_datasets()
        .await?
        .try_tap(|c| validate_data_catalog(c, roi))?
        .try_pipe(|c| load_layers(&c))?
        .try_pipe(|assets| align_and_resample(&assets, &ctx))?
        .try_tap_mut(terrain_restoration)?
        .try_tap(validate_terrain_grid)?;

    let physics_map = physics_analyze(&terrain, &ctx)?;

    let avg_slope: f32 = physics_map.slope.iter().sum::<f32>()
        / physics_map.slope.len() as f32
        / std::f32::consts::PI;
    println!("Average Slope: {:.4}π rad", avg_slope);

    Ok(())
}
