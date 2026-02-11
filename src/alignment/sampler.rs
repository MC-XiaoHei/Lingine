use crate::core::terrain::TerrainPixel;
use crate::loader::{LayerBundle, mosaic::MosaicSession};

pub struct SamplingSession {
    elevation: MosaicSession,
    landcover: MosaicSession,
    sand: MosaicSession,
    clay: MosaicSession,
    soc: MosaicSession,
    ph: MosaicSession,
}

impl SamplingSession {
    pub fn new(bundle: &LayerBundle) -> Self {
        Self {
            elevation: bundle.elevation.open_session(),
            landcover: bundle.landcover.open_session(),
            sand: bundle.sand.open_session(),
            clay: bundle.clay.open_session(),
            soc: bundle.soc.open_session(),
            ph: bundle.ph.open_session(),
        }
    }

    #[inline]
    pub fn sample(&mut self, lon: f64, lat: f64) -> TerrainPixel {
        TerrainPixel {
            elevation: self.elevation.fetch_bicubic(lon, lat).ok().flatten(),

            landcover: self
                .landcover
                .fetch_nearest(lon, lat)
                .ok()
                .flatten()
                .map(|v| v as u8),

            sand: self.sand.fetch_bilinear(lon, lat).ok().flatten(),
            clay: self.clay.fetch_bilinear(lon, lat).ok().flatten(),
            soc: self.soc.fetch_bilinear(lon, lat).ok().flatten(),
            ph: self.ph.fetch_bilinear(lon, lat).ok().flatten(),
        }
    }
}
