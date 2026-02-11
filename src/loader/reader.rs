use crate::utils::dataset::DatasetEx;
use anyhow::{Context, Result, anyhow};
use gdal::{Dataset, Metadata};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ReaderSource {
    path: PathBuf,
    inv_transform: [f64; 6],
    needs_half_pixel_shift: bool,
    no_data_value: Option<f64>,
}

impl ReaderSource {
    pub fn new(path: PathBuf) -> Result<Self> {
        let dataset = Dataset::open_dataset(&path)?;

        let gt = dataset
            .geo_transform()
            .context(format!("GeoTransform missing for {path:?}"))?;
        let det = gt[1] * gt[5] - gt[2] * gt[4];
        if det.abs() < 1e-10 {
            return Err(anyhow!("Invalid GeoTransform for {path:?}"));
        }

        let inv_transform = [
            0.0,
            gt[5] / det,
            -gt[2] / det,
            0.0,
            -gt[4] / det,
            gt[1] / det,
        ];

        let needs_half_pixel_shift = dataset
            .metadata_item("AREA_OR_POINT", "")
            .map(|s| s == "Area")
            .unwrap_or(false);

        let no_data_value = dataset.rasterband(1)?.no_data_value();

        Ok(Self {
            path,
            inv_transform,
            needs_half_pixel_shift,
            no_data_value,
        })
    }

    pub fn open_session(&self) -> Result<ReaderSession> {
        let dataset = Dataset::open_dataset(&self.path)?;

        const NO_CACHE: isize = -9999;

        Ok(ReaderSession {
            source: self.clone(),
            dataset,
            cache: None,
        })
    }
}

const CACHE_BLOCK_SIZE: isize = 256;

struct BlockCache {
    start_col: isize,
    start_row: isize,
    width: usize,
    height: usize,
    data: Vec<f32>,
}

pub struct ReaderSession {
    source: ReaderSource,
    dataset: Dataset,
    cache: Option<BlockCache>,
}

impl ReaderSession {
    fn get_pixel_coord(&self, lon: f64, lat: f64) -> (f64, f64) {
        let inv = self.source.inv_transform;
        let gt = self.dataset.geo_transform().unwrap_or([0.0; 6]);

        let dx = lon - gt[0];
        let dy = lat - gt[3];
        let mut u = dx * inv[1] + dy * inv[2];
        let mut v = dx * inv[4] + dy * inv[5];

        if self.source.needs_half_pixel_shift {
            u -= 0.5;
            v -= 0.5;
        }
        (u, v)
    }

    pub fn probe_nearest(&mut self, lon: f64, lat: f64) -> Option<f32> {
        let (u, v) = self.get_pixel_coord(lon, lat);
        let col = u.round() as isize;
        let row = v.round() as isize;
        self.read_safe(col, row)
    }

    pub fn probe_bilinear(&mut self, lon: f64, lat: f64) -> Option<f32> {
        let (u, v) = self.get_pixel_coord(lon, lat);
        let x = u.floor();
        let y = v.floor();
        let u_ratio = (u - x) as f32;
        let v_ratio = (v - y) as f32;
        let u_opposite = 1.0 - u_ratio;
        let v_opposite = 1.0 - v_ratio;

        let col = x as isize;
        let row = y as isize;

        let v00 = self.read_safe(col, row)?;
        let v10 = self.read_safe(col + 1, row)?;
        let v01 = self.read_safe(col, row + 1)?;
        let v11 = self.read_safe(col + 1, row + 1)?;

        let result = (v00 * u_opposite + v10 * u_ratio) * v_opposite +
            (v01 * u_opposite + v11 * u_ratio) * v_ratio;

        Some(result)
    }

    pub fn probe_bicubic(&mut self, lon: f64, lat: f64) -> Option<f32> {
        let (u, v) = self.get_pixel_coord(lon, lat);
        let x = u.floor();
        let y = v.floor();
        let dx = (u - x) as f32;
        let dy = (v - y) as f32;

        let col = x as isize;
        let row = y as isize;

        let mut pixels = [[0.0; 4]; 4];

        for j in 0..4 {
            for i in 0..4 {
                if let Some(val) = self.read_safe(col - 1 + i, row - 1 + j) {
                    pixels[j as usize][i as usize] = val;
                } else {
                    return self.probe_bilinear(lon, lat);
                }
            }
        }

        let eval_cubic = |p: [f32; 4], t: f32| -> f32 {
            p[1] + 0.5 * t * (p[2] - p[0] + t * (2.0 * p[0] - 5.0 * p[1] + 4.0 * p[2] - p[3] + t * (3.0 * (p[1] - p[2]) + p[3] - p[0])))
        };

        let mut arr = [0.0; 4];
        for j in 0..4 {
            arr[j] = eval_cubic(pixels[j], dx);
        }

        Some(eval_cubic(arr, dy))
    }

    fn read_safe(&mut self, col: isize, row: isize) -> Option<f32> {
        let (w, h) = self.dataset.rasterband(1).ok()?.size();
        let w = w as isize;
        let h = h as isize;

        if col < 0 || row < 0 || col >= w || row >= h {
            return None;
        }

        if let Some(cache) = &self.cache {
            if col >= cache.start_col
                && col < cache.start_col + cache.width as isize
                && row >= cache.start_row
                && row < cache.start_row + cache.height as isize
            {
                let local_col = (col - cache.start_col) as usize;
                let local_row = (row - cache.start_row) as usize;
                let idx = local_row * cache.width + local_col;
                return self.apply_nodata_check(cache.data[idx]);
            }
        }

        self.load_cache_block(col, row, w, h)?;
        self.read_safe(col, row)
    }

    fn load_cache_block(&mut self, target_col: isize, target_row: isize, image_w: isize, image_h: isize) -> Option<()> {
        let start_col = (target_col / CACHE_BLOCK_SIZE) * CACHE_BLOCK_SIZE;
        let start_row = (target_row / CACHE_BLOCK_SIZE) * CACHE_BLOCK_SIZE;

        let block_w = CACHE_BLOCK_SIZE.min(image_w - start_col) as usize;
        let block_h = CACHE_BLOCK_SIZE.min(image_h - start_row) as usize;

        if block_w == 0 || block_h == 0 { return None; }

        let band = self.dataset.rasterband(1).ok()?;

        let buffer = band.read_as::<f32>(
            (start_col, start_row),
            (block_w, block_h),
            (block_w, block_h),
            None
        ).ok()?;

        self.cache = Some(BlockCache {
            start_col,
            start_row,
            width: block_w,
            height: block_h,
            data: buffer.data().to_vec(),
        });

        Some(())
    }

    #[inline]
    fn apply_nodata_check(&self, val: f32) -> Option<f32> {
        if let Some(no_data) = self.source.no_data_value {
            if (val - no_data as f32).abs() < 1e-6 {
                return None;
            }
        }
        Some(val)
    }
}
