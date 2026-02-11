use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct TerrainPixel {
    pub elevation: Option<f32>,
    pub landcover: Option<u8>,
    pub sand: Option<f32>,
    pub clay: Option<f32>,
    pub soc: Option<f32>,
    pub ph: Option<f32>,
}

pub struct RowViewMut<'a> {
    elevation: &'a mut [Option<f32>],
    landcover: &'a mut [Option<u8>],
    sand: &'a mut [Option<f32>],
    clay: &'a mut [Option<f32>],
    soc: &'a mut [Option<f32>],
    ph: &'a mut [Option<f32>],
}

impl<'a> RowViewMut<'a> {
    #[inline]
    pub fn set(&mut self, x: usize, pixel: TerrainPixel) {
        self.elevation[x] = pixel.elevation;
        self.landcover[x] = pixel.landcover;
        self.sand[x] = pixel.sand;
        self.clay[x] = pixel.clay;
        self.soc[x] = pixel.soc;
        self.ph[x] = pixel.ph;
    }
}

#[derive(Debug)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,

    pub elevation: Vec<Option<f32>>,
    pub landcover: Vec<Option<u8>>,
    pub sand: Vec<Option<f32>>,
    pub clay: Vec<Option<f32>>,
    pub soc: Vec<Option<f32>>,
    pub ph: Vec<Option<f32>>,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            elevation: vec![None; len],
            landcover: vec![None; len],
            sand: vec![None; len],
            clay: vec![None; len],
            soc: vec![None; len],
            ph: vec![None; len],
        }
    }

    pub fn par_rows_mut(&mut self) -> impl IndexedParallelIterator<Item = RowViewMut<'_>> {
        let w = self.width;
        (
            self.elevation.par_chunks_mut(w),
            self.landcover.par_chunks_mut(w),
            self.sand.par_chunks_mut(w),
            self.clay.par_chunks_mut(w),
            self.soc.par_chunks_mut(w),
            self.ph.par_chunks_mut(w),
        )
            .into_par_iter()
            .map(|(e, l, s, c, so, p)| RowViewMut {
                elevation: e,
                landcover: l,
                sand: s,
                clay: c,
                soc: so,
                ph: p,
            })
    }
}