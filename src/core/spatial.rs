use anyhow::{Result, anyhow};
use geo::Coord;

#[derive(Debug, Clone, Copy)]
pub struct GeoTransform {
    forward_matrix: [f64; 6],
    inverse_matrix: [f64; 6],
}

impl GeoTransform {
    pub fn from_gdal(gt: [f64; 6]) -> Result<Self> {
        let determinant = gt[1] * gt[5] - gt[2] * gt[4];

        if determinant.abs() < 1e-10 {
            return Err(anyhow!("Invalid GeoTransform: Determinant is zero"));
        }

        let inv_det = 1.0 / determinant;

        let inverse_matrix = [
            (gt[2] * gt[3] - gt[5] * gt[0]) * inv_det,
            gt[5] * inv_det,
            -gt[2] * inv_det,
            (gt[4] * gt[0] - gt[1] * gt[3]) * inv_det,
            -gt[4] * inv_det,
            gt[1] * inv_det,
        ];

        Ok(Self {
            forward_matrix: gt,
            inverse_matrix,
        })
    }

    #[inline]
    pub fn geo_to_pixel(&self, x: f64, y: f64) -> Coord<f64> {
        let inv = self.inverse_matrix;
        let u = inv[0] + x * inv[1] + y * inv[2];
        let v = inv[3] + x * inv[4] + y * inv[5];
        Coord { x: u, y: v }
    }

    #[inline]
    pub fn pixel_to_geo(&self, x: f64, y: f64) -> Coord<f64> {
        let fwd = self.forward_matrix;
        let lx = fwd[0] + x * fwd[1] + y * fwd[2];
        let ly = fwd[3] + x * fwd[4] + y * fwd[5];
        Coord { x: lx, y: ly }
    }
}
