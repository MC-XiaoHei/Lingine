use crate::utils::dataset::DatasetEx;
use anyhow::{Context, Result, anyhow};
use gdal::Dataset;
use gdal::spatial_ref::{AxisMappingStrategy, CoordTransform, SpatialRef};
use geo::{Coord, Rect};
use std::path::Path;
use tokio::task::spawn_blocking;

pub async fn extract_bounds_async(path: &Path) -> Result<Rect<f64>> {
    let path = path.to_path_buf();
    spawn_blocking(move || Ok::<Rect, anyhow::Error>(extract_bounds(&path)?)).await?
}

fn extract_bounds(path: &Path) -> Result<Rect<f64>> {
    let dataset = Dataset::open_dataset(path)?;

    let transform = create_projection_transform(&dataset)?;
    let pixel_corners = get_pixel_corners(&dataset);
    let source_coords = apply_geo_transform(&pixel_corners, &dataset)?;
    let wgs84_coords = reproject_to_wgs84(source_coords, &transform)?;

    compute_bounding_box(&wgs84_coords)
}

struct PointSet {
    x: Vec<f64>,
    y: Vec<f64>,
}

fn create_projection_transform(dataset: &Dataset) -> Result<CoordTransform> {
    let mut source_srs = determine_source_srs(dataset)?;
    source_srs.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);

    let mut target_srs = SpatialRef::from_epsg(4326)?;
    target_srs.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);

    CoordTransform::new(&source_srs, &target_srs)
        .context("Failed to create coordinate transform pipeline")
}

fn determine_source_srs(dataset: &Dataset) -> Result<SpatialRef> {
    let wkt = dataset.projection();

    if wkt.is_empty() {
        Err(anyhow!("No WKT in file header"))
    } else {
        SpatialRef::from_wkt(&wkt).context("Invalid WKT in file header")
    }
}

fn get_pixel_corners(dataset: &Dataset) -> PointSet {
    let (width, height) = dataset.raster_size();
    let w = width as f64;
    let h = height as f64;

    PointSet {
        x: vec![0.0, w, w, 0.0],
        y: vec![0.0, 0.0, h, h],
    }
}

fn apply_geo_transform(pixels: &PointSet, dataset: &Dataset) -> Result<PointSet> {
    let gt = dataset.geo_transform().context("GeoTransform missing")?;

    let mut geo_x = Vec::with_capacity(4);
    let mut geo_y = Vec::with_capacity(4);

    for i in 0..4 {
        let px = pixels.x[i];
        let py = pixels.y[i];

        // X_geo = GT[0] + X_pix * GT[1] + Y_pix * GT[2]
        let x = gt[0] + px * gt[1] + py * gt[2];
        let y = gt[3] + px * gt[4] + py * gt[5];

        geo_x.push(x);
        geo_y.push(y);
    }

    Ok(PointSet { x: geo_x, y: geo_y })
}

fn reproject_to_wgs84(mut coords: PointSet, transform: &CoordTransform) -> Result<PointSet> {
    let mut z = vec![0.0; 4];

    transform
        .transform_coords(&mut coords.x, &mut coords.y, &mut z)
        .context("GDAL Reprojection failed")?;

    Ok(coords)
}

fn compute_bounding_box(coords: &PointSet) -> Result<Rect<f64>> {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for i in 0..4 {
        let x = coords.x[i];
        let y = coords.y[i];

        if !x.is_finite() || !y.is_finite() {
            return Err(anyhow!("Reprojection resulted in non-finite coordinates"));
        }

        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }

    Ok(Rect::new(
        Coord { x: min_x, y: min_y },
        Coord { x: max_x, y: max_y },
    ))
}
