use super::reader::{ReaderSession, ReaderSource};
use crate::core::raster::{Bicubic, Bilinear, Interpolator, NearestNeighbor};
use anyhow::Result;
use geo::{Contains, Coord, Rect};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct TileEntry {
    id: String,
    bounds: Rect<f64>,
    reader_source: ReaderSource,
}

#[derive(Clone)]
pub struct MosaicSource {
    tiles: Arc<Vec<TileEntry>>,
}

impl MosaicSource {
    pub fn new(items: Vec<(String, Rect<f64>, PathBuf)>) -> Self {
        let mut entries = Vec::new();
        for (id, bounds, path) in items {
            if let Ok(src) = ReaderSource::new(path) {
                entries.push(TileEntry {
                    id,
                    bounds,
                    reader_source: src,
                });
            }
        }
        entries.sort_by(|a, b| a.id.cmp(&b.id));

        Self {
            tiles: Arc::new(entries),
        }
    }

    pub fn open_session(&self) -> MosaicSession {
        MosaicSession {
            source: self.clone(),
            active_session: None,
            active_tile_id: None,
        }
    }
}

pub struct MosaicSession {
    source: MosaicSource,
    active_session: Option<ReaderSession>,
    active_tile_id: Option<String>,
}

impl MosaicSession {
    pub fn fetch_nearest(&mut self, lon: f64, lat: f64) -> Result<Option<f32>> {
        self.fetch_impl(lon, lat, &NearestNeighbor)
    }

    pub fn fetch_bilinear(&mut self, lon: f64, lat: f64) -> Result<Option<f32>> {
        self.fetch_impl(lon, lat, &Bilinear)
    }

    pub fn fetch_bicubic(&mut self, lon: f64, lat: f64) -> Result<Option<f32>> {
        self.fetch_impl(lon, lat, &Bicubic)
    }

    fn fetch_impl<T: Interpolator>(
        &mut self,
        lon: f64,
        lat: f64,
        strategy: &T,
    ) -> Result<Option<f32>> {
        let coord = Coord { x: lon, y: lat };

        let target = self
            .source
            .tiles
            .iter()
            .rev()
            .find(|t| t.bounds.contains(&coord));

        if let Some(tile) = target {
            if self.active_tile_id.as_ref() != Some(&tile.id) {
                let session = tile.reader_source.open_session()?;
                self.active_session = Some(session);
                self.active_tile_id = Some(tile.id.clone());
            }

            if let Some(reader) = &mut self.active_session {
                return Ok(reader.sample(lon, lat, strategy));
            }
        }
        Ok(None)
    }
}
