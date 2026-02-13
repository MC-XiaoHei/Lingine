use crate::core::projection::AdaptiveLtm;
use geo::{Coord, Rect};
use std::fmt::Display;

pub struct SpatialContext {
    pub ltm: AdaptiveLtm,
    pub roi_meters: Rect<f64>,
    pub width: usize,
    pub height: usize,
    pub total_pixels: u64,
}

impl SpatialContext {
    pub fn analyze(roi_geo: Rect<f64>) -> Self {
        let center = roi_geo.center();
        let ltm = AdaptiveLtm::new(center);

        let p_min = ltm.project(roi_geo.min().x, roi_geo.min().y);
        let p_max = ltm.project(roi_geo.max().x, roi_geo.max().y);
        let roi_meters = Rect::new(p_min, p_max);

        let width = roi_meters.width().abs().round() as usize;
        let height = roi_meters.height().abs().round() as usize;
        let total_pixels = (width * height) as u64;

        Self {
            ltm,
            roi_meters,
            width,
            height,
            total_pixels,
        }
    }

    #[inline]
    pub fn get_geo_coord(&self, x: usize, y: usize) -> Coord<f64> {
        let mx = self.roi_meters.min().x + x as f64 + 0.5;
        let my = self.roi_meters.min().y + y as f64 + 0.5;
        self.ltm.unproject(mx, my)
    }
}

impl Display for SpatialContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Physical Dimensions: {:.2}m x {:.2}m",
            self.roi_meters.width().abs(),
            self.roi_meters.height().abs()
        )?;
        write!(f, "Grid Resolution: {} x {}", self.width, self.height)?;
        write!(f, "Total Voxels: {}", self.total_pixels)?;
        Ok(())
    }
}
