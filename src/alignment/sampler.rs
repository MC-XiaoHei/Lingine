use crate::core::terrain::TerrainPixel;
use crate::loader::bundle::LayerBundle;
use crate::loader::mosaic::MosaicSession;

pub struct SamplingSession {
    elevation: MosaicSession,
    hh: MosaicSession,
    hv: MosaicSession,
    inc: MosaicSession,
    ls: MosaicSession,

    landcover: MosaicSession,

    sand: MosaicSession,
    sand_sub: MosaicSession,
    clay: MosaicSession,
    clay_sub: MosaicSession,
    ph: MosaicSession,
    ph_sub: MosaicSession,
    soc: MosaicSession,
}

impl SamplingSession {
    pub fn new(bundle: &LayerBundle) -> Self {
        Self {
            elevation: bundle.elevation.open_session(),
            hh: bundle.hh.open_session(),
            hv: bundle.hv.open_session(),
            inc: bundle.inc.open_session(),
            ls: bundle.ls.open_session(),

            landcover: bundle.landcover.open_session(),

            sand: bundle.sand.open_session(),
            clay: bundle.clay.open_session(),
            ph: bundle.ph.open_session(),
            soc: bundle.soc.open_session(),

            sand_sub: bundle.sand_sub.open_session(),
            clay_sub: bundle.clay_sub.open_session(),
            ph_sub: bundle.ph_sub.open_session(),
        }
    }

    #[inline]
    pub fn sample(&mut self, lon: f64, lat: f64) -> TerrainPixel {
        TerrainPixel {
            elevation: self.elevation.fetch_bicubic(lon, lat).ok().flatten(),

            hh: self.hh.fetch_bilinear(lon, lat).ok().flatten(),
            hv: self.hv.fetch_bilinear(lon, lat).ok().flatten(),

            inc: self.inc.fetch_bilinear(lon, lat).ok().flatten(),
            ls: self.ls.fetch_bilinear(lon, lat).ok().flatten(),

            landcover: self
                .landcover
                .fetch_nearest(lon, lat)
                .ok()
                .flatten()
                .map(|v| v as u8),

            sand: self.sand.fetch_bilinear(lon, lat).ok().flatten(),
            clay: self.clay.fetch_bilinear(lon, lat).ok().flatten(),
            ph: self.ph.fetch_bilinear(lon, lat).ok().flatten(),
            soc: self.soc.fetch_bilinear(lon, lat).ok().flatten(),

            sand_sub: self.sand_sub.fetch_bilinear(lon, lat).ok().flatten(),
            clay_sub: self.clay_sub.fetch_bilinear(lon, lat).ok().flatten(),
            ph_sub: self.ph_sub.fetch_bilinear(lon, lat).ok().flatten(),
        }
    }
}
