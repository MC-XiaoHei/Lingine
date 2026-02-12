mod alignment;
mod core;
mod denoise;
mod loader;
mod physics;
mod scanner;
mod utils;

use crate::scanner::scan_datasets;
use anyhow::{Result, anyhow};
use geo::{Coord, Rect};

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

    let data_catalog = scan_datasets().await?;

    let coverage = data_catalog.check_coverage(roi);
    if !coverage.is_full() {
        println!("{coverage}");
        return Ok(());
    }

    let assets = loader::load_assets(&data_catalog)?;
    let mut terrain = alignment::create_and_align(&assets, roi)?;

    denoise::denoise(&mut terrain)?;

    let valid_count = terrain.elevation.iter().flatten().count();
    let total_count = terrain.elevation.len();

    if valid_count == 0 {
        return Err(anyhow!(
            "Critical Error: Terrain is empty! Reader failed to load data."
        ));
    }

    let fill_rate = valid_count as f64 / total_count as f64 * 100.0;
    println!(
        "DEBUG: Terrain Elevation Fill Rate: {:.2}% ({}/{})",
        fill_rate, valid_count, total_count
    );

    let physics_map = physics::analyze(&terrain)?;

    let avg_slope: f32 = physics_map.slope.iter().sum::<f32>()
        / physics_map.slope.len() as f32
        / std::f32::consts::PI;
    println!("Average Slope: {:.4}π rad", avg_slope);

    Ok(())
}
