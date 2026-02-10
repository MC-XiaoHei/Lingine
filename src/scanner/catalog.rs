use super::{alos, esa, soil, types::*};
use anyhow::Result;
use geo::{Area, BooleanOps, MultiPolygon, Polygon, Rect};
use std::path::PathBuf;

impl DataCatalog {
    pub async fn scan(root: PathBuf) -> Result<Self> {
        let (alos_res, esa_res, soil_res) = tokio::try_join!(
            alos::scan(root.join("alos_palsar")),
            esa::scan(root.join("esa_world_cover")),
            soil::scan(root.join("soil_grids")),
        )?;

        Ok(Self {
            alos: alos_res,
            esa: esa_res,
            soil: soil_res,
        })
    }

    pub fn check_coverage(&self, rect: Rect<f64>) -> ValidationReport {
        let target = Polygon::from(rect);

        let ratios = [
            ("ALOS Palsar", self.calc_ratio(&self.alos_polys(), &target)),
            ("ESA WorldCover", self.calc_ratio(&self.esa_polys(), &target)),
            ("SoilGrid", self.calc_ratio(&self.soil_polys(), &target)),
        ];

        let details = ratios
            .iter()
            .filter(|(_, ratio)| *ratio < 0.999)
            .map(|(name, ratio)| {
                format!("{} incomplete, now coverage: {:.2}%", name, ratio * 100.0)
            })
            .collect::<Vec<_>>();

        ValidationReport {
            is_ready: details.is_empty(),
            details,
        }
    }

    fn calc_ratio(&self, source: &[Polygon<f64>], target: &Polygon<f64>) -> f64 {
        let mut mp = MultiPolygon::new(vec![]);
        for p in source {
            mp = mp.union(&MultiPolygon::from(p.clone()));
        }
        let intersection = mp.intersection(&MultiPolygon::from(target.clone()));
        intersection.unsigned_area() / target.unsigned_area()
    }

    fn alos_polys(&self) -> Vec<Polygon<f64>> {
        self.alos.iter().map(|s| s.bounds.into()).collect()
    }

    fn esa_polys(&self) -> Vec<Polygon<f64>> {
        self.esa.iter().map(|s| s.bounds.into()).collect()
    }

    fn soil_polys(&self) -> Vec<Polygon<f64>> {
        self.soil.iter().map(|s| s.bounds.into()).collect()
    }
}
