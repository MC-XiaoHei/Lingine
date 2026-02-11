mod alignment;
mod core;
mod denoise;
mod loader;
mod scanner;
mod utils;

use crate::scanner::CoverageResult::Partial;
use crate::scanner::scan_datasets;
use anyhow::Result;
use geo::{Coord, Rect};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = main_func().await {
        eprintln!("Error: {e}");
    }
    Ok(())
}

async fn main_func() -> Result<()> {
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
    let cov_report = data_catalog.check_coverage(roi);
    if let Partial {
        alos_coverage,
        esa_coverage,
        soil_coverage,
    } = cov_report
    {
        println!("Dataset insufficient, the current datasets does not cover the request ROI");
        println!("Coverages are:");
        println!("Alos Palsar: {:.2}%", alos_coverage * 100.0);
        println!("Esa WorldCover: {:.2}%", esa_coverage * 100.0);
        println!("Soil Grids: {:.2}%", soil_coverage * 100.0);
        return Ok(());
    }
    let assets = loader::load_assets(&data_catalog)?;
    let mut terrain = alignment::create_and_align(&assets, roi)?;

    denoise::denoise(&mut terrain)?;

    Ok(())
}
