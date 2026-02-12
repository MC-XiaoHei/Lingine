use crate::core::terrain::TerrainGrid;
use crate::utils::progress::create_progress_bar;
use anyhow::{Result, anyhow};
use geo::Rect;
use indicatif::ProgressBar;
use rayon::prelude::*;
use crate::scanner::types::DataCatalog;

pub fn validate_data_catalog(data_catalog: &DataCatalog, roi: Rect<f64>) -> Result<()> {
    let coverage = data_catalog.check_coverage(roi);
    if !coverage.is_full() {
        println!("{coverage}");
        Err(anyhow!("Coverage check failed"))
    } else {
        Ok(())
    }
}

pub fn validate_terrain_grid(terrain: &TerrainGrid) -> Result<()> {
    let f32_layers = [
        ("Elevation", &terrain.elevation),
        ("HH", &terrain.hh),
        ("HV", &terrain.hv),
        ("Incidence", &terrain.inc),
        ("Layover/Shadow", &terrain.ls),
        ("Sand", &terrain.sand),
        ("Clay", &terrain.clay),
        ("pH", &terrain.ph),
        ("SOC", &terrain.soc),
        ("Sand Sub", &terrain.sand_sub),
        ("Clay Sub", &terrain.clay_sub),
        ("pH Sub", &terrain.ph_sub),
    ];

    let u8_layers = [("Landcover", &terrain.landcover)];

    let total = terrain.elevation.len();
    let num_layers = u8_layers.len() + f32_layers.len();
    let total_work = total * num_layers;

    let bar = create_progress_bar(total_work as u64, "Validate Terrain Integrity");
    let chunk_size = 10_000.max(total / 100);

    for (name, data) in f32_layers {
        verify_layer(name, data, total, chunk_size, &bar)?;
    }

    for (name, data) in u8_layers {
        verify_layer(name, data, total, chunk_size, &bar)?;
    }

    bar.finish();
    Ok(())
}

fn verify_layer<T>(
    name: &str,
    data: &[Option<T>],
    total_expected: usize,
    chunk_size: usize,
    bar: &ProgressBar,
) -> Result<()>
where
    T: Sync + Send,
{
    let valid_count: usize = data
        .par_chunks(chunk_size)
        .map(|chunk| {
            bar.inc(chunk.len() as u64);
            chunk.iter().flatten().count()
        })
        .sum();

    if valid_count != total_expected {
        bar.finish_and_clear();
        return Err(anyhow!(
            "Data Integrity Error: Layer [{}] is incomplete ({}/{}). All layers must be fully populated after resampling.",
            name,
            valid_count,
            total_expected
        ));
    }
    Ok(())
}
