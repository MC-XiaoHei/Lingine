use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct TerrainPixel {
    pub elevation: f32,

    pub hh: f32,
    pub hv: f32,
    pub inc: f32,
    pub ls: f32,

    pub landcover: Option<u8>,

    pub sand: f32,
    pub sand_sub: f32,
    pub clay: f32,
    pub clay_sub: f32,
    pub ph: f32,
    pub ph_sub: f32,
    pub soc: f32,
}

pub struct RowViewMut<'a> {
    elevation: &'a mut [f32],
    hh: &'a mut [f32],
    hv: &'a mut [f32],
    inc: &'a mut [f32],
    ls: &'a mut [f32],
    landcover: &'a mut [Option<u8>],
    sand: &'a mut [f32],
    sand_sub: &'a mut [f32],
    clay: &'a mut [f32],
    clay_sub: &'a mut [f32],
    ph: &'a mut [f32],
    ph_sub: &'a mut [f32],
    soc: &'a mut [f32],
}

impl<'a> RowViewMut<'a> {
    #[inline]
    pub fn set(&mut self, x: usize, pixel: TerrainPixel) {
        self.elevation[x] = pixel.elevation;
        self.hh[x] = pixel.hh;
        self.hv[x] = pixel.hv;
        self.inc[x] = pixel.inc;
        self.ls[x] = pixel.ls;
        self.landcover[x] = pixel.landcover;
        self.sand[x] = pixel.sand;
        self.sand_sub[x] = pixel.sand_sub;
        self.clay[x] = pixel.clay;
        self.clay_sub[x] = pixel.clay_sub;
        self.ph[x] = pixel.ph;
        self.ph_sub[x] = pixel.ph_sub;
        self.soc[x] = pixel.soc;
    }
}

#[derive(Debug)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,

    pub min_elevation: f32,
    pub max_elevation: f32,

    pub elevation: Vec<f32>,
    pub hh: Vec<f32>,
    pub hv: Vec<f32>,
    pub inc: Vec<f32>,
    pub ls: Vec<f32>,

    pub landcover: Vec<Option<u8>>,

    pub sand: Vec<f32>,
    pub sand_sub: Vec<f32>,
    pub clay: Vec<f32>,
    pub clay_sub: Vec<f32>,
    pub ph: Vec<f32>,
    pub ph_sub: Vec<f32>,
    pub soc: Vec<f32>,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            min_elevation: f32::MAX,
            max_elevation: f32::MIN,
            elevation: vec![f32::NAN; len],
            hh: vec![f32::NAN; len],
            hv: vec![f32::NAN; len],
            inc: vec![f32::NAN; len],
            ls: vec![f32::NAN; len],
            landcover: vec![None; len],
            sand: vec![f32::NAN; len],
            sand_sub: vec![f32::NAN; len],
            clay: vec![f32::NAN; len],
            clay_sub: vec![f32::NAN; len],
            ph: vec![f32::NAN; len],
            ph_sub: vec![f32::NAN; len],
            soc: vec![f32::NAN; len],
        }
    }

    pub fn par_rows_mut(&mut self) -> impl IndexedParallelIterator<Item = RowViewMut<'_>> {
        let w = self.width;
        (
            (
                self.elevation.par_chunks_mut(w),
                self.hh.par_chunks_mut(w),
                self.hv.par_chunks_mut(w),
                self.inc.par_chunks_mut(w),
                self.ls.par_chunks_mut(w),
                self.landcover.par_chunks_mut(w),
            ),
            (
                self.sand.par_chunks_mut(w),
                self.sand_sub.par_chunks_mut(w),
                self.clay.par_chunks_mut(w),
                self.clay_sub.par_chunks_mut(w),
                self.ph.par_chunks_mut(w),
                self.ph_sub.par_chunks_mut(w),
                self.soc.par_chunks_mut(w),
            ),
        )
            .into_par_iter()
            .map(
                |((e, hh, hv, inc, ls, lc), (s, ss, c, cs, p, ps, soc))| RowViewMut {
                    elevation: e,
                    hh,
                    hv,
                    inc,
                    ls,
                    landcover: lc,
                    sand: s,
                    sand_sub: ss,
                    clay: c,
                    clay_sub: cs,
                    ph: p,
                    ph_sub: ps,
                    soc,
                },
            )
    }
}
