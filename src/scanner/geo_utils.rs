use crate::core::spatial::GeoTransform;
use crate::utils::dataset::DatasetEx;
use anyhow::{Context, Result, anyhow};
use gdal::Dataset;
use gdal::spatial_ref::{AxisMappingStrategy, CoordTransform, SpatialRef};
use geo::{Coord, Rect};
use std::path::Path;
use tokio::task::spawn_blocking;

pub async fn extract_bounds_async(path: &Path) -> Result<Rect<f64>> {
    let path_owned = path.to_path_buf();
    spawn_blocking(move || extract_bounds(&path_owned)).await?
}

fn extract_bounds(path: &Path) -> Result<Rect<f64>> {
    let dataset = Dataset::open_dataset(path)?;
    let transform_pipeline = create_projection_pipeline(&dataset)?;

    let gt_array = dataset.geo_transform().context("GeoTransform missing")?;
    let geo_transform = GeoTransform::from_gdal(gt_array)?;

    let (w, h) = dataset.raster_size();

    let corners_native = vec![
        geo_transform.pixel_to_geo(0.0, 0.0),
        geo_transform.pixel_to_geo(w as f64, 0.0),
        geo_transform.pixel_to_geo(w as f64, h as f64),
        geo_transform.pixel_to_geo(0.0, h as f64),
    ];

    let corners_wgs84 = reproject_points(corners_native, &transform_pipeline)?;
    compute_bounding_box(&corners_wgs84)
}

fn reproject_points(
    coords: Vec<Coord<f64>>,
    transform: &CoordTransform,
) -> Result<Vec<Coord<f64>>> {
    let mut x: Vec<f64> = coords.iter().map(|c| c.x).collect();
    let mut y: Vec<f64> = coords.iter().map(|c| c.y).collect();
    let mut z: Vec<f64> = vec![0.0; coords.len()];

    transform
        .transform_coords(&mut x, &mut y, &mut z)
        .context("GDAL Reprojection failed")?;

    Ok(x.into_iter()
        .zip(y.into_iter())
        .map(|(rx, ry)| Coord { x: rx, y: ry })
        .collect())
}

fn create_projection_pipeline(dataset: &Dataset) -> Result<CoordTransform> {
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

fn compute_bounding_box(coords: &[Coord<f64>]) -> Result<Rect<f64>> {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for p in coords {
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(anyhow!("Non-finite coordinates after reprojection"));
        }
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }

    Ok(Rect::new(
        Coord { x: min_x, y: min_y },
        Coord { x: max_x, y: max_y },
    ))
}
