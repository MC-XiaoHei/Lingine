use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct TerrainPixel {
    pub elevation: Option<f32>,

    pub hh: Option<f32>,
    pub hv: Option<f32>,
    pub inc: Option<f32>,
    pub ls: Option<f32>,

    pub landcover: Option<u8>,

    pub sand: Option<f32>,
    pub sand_sub: Option<f32>,
    pub clay: Option<f32>,
    pub clay_sub: Option<f32>,
    pub ph: Option<f32>,
    pub ph_sub: Option<f32>,
    pub soc: Option<f32>,
}

pub struct RowViewMut<'a> {
    elevation: &'a mut [Option<f32>],
    hh: &'a mut [Option<f32>],
    hv: &'a mut [Option<f32>],
    inc: &'a mut [Option<f32>],
    ls: &'a mut [Option<f32>],
    landcover: &'a mut [Option<u8>],
    sand: &'a mut [Option<f32>],
    sand_sub: &'a mut [Option<f32>],
    clay: &'a mut [Option<f32>],
    clay_sub: &'a mut [Option<f32>],
    ph: &'a mut [Option<f32>],
    ph_sub: &'a mut [Option<f32>],
    soc: &'a mut [Option<f32>],
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

    pub elevation: Vec<Option<f32>>,
    pub hh: Vec<Option<f32>>,
    pub hv: Vec<Option<f32>>,
    pub inc: Vec<Option<f32>>,
    pub ls: Vec<Option<f32>>,

    pub landcover: Vec<Option<u8>>,

    pub sand: Vec<Option<f32>>,
    pub sand_sub: Vec<Option<f32>>,
    pub clay: Vec<Option<f32>>,
    pub clay_sub: Vec<Option<f32>>,
    pub ph: Vec<Option<f32>>,
    pub ph_sub: Vec<Option<f32>>,
    pub soc: Vec<Option<f32>>,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            elevation: vec![None; len],
            hh: vec![None; len],
            hv: vec![None; len],
            inc: vec![None; len],
            ls: vec![None; len],
            landcover: vec![None; len],
            sand: vec![None; len],
            sand_sub: vec![None; len],
            clay: vec![None; len],
            clay_sub: vec![None; len],
            ph: vec![None; len],
            ph_sub: vec![None; len],
            soc: vec![None; len],
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
