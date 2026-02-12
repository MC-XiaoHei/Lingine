use crate::core::raster::{Interpolator, PixelSource};
use crate::core::spatial::GeoTransform;
use crate::utils::dataset::DatasetEx;
use anyhow::{Context, Result};
use gdal::{Dataset, Metadata};
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ReaderSource {
    path: PathBuf,
    pub transform: GeoTransform,
    needs_half_pixel_shift: bool,
    no_data_value: Option<f64>,
    preloaded_data: Option<Arc<Vec<f32>>>,
    pub width: usize,
    pub height: usize,
}

impl ReaderSource {
    pub fn new(path: PathBuf) -> Result<Self> {
        let dataset = Dataset::open_dataset(&path)?;

        let gt_array = dataset
            .geo_transform()
            .context(format!("GeoTransform missing for {path:?}"))?;

        let transform = GeoTransform::from_gdal(gt_array)?;

        let needs_half_pixel_shift = dataset
            .metadata_item("AREA_OR_POINT", "")
            .map(|s| s == "Area")
            .unwrap_or(false);

        let band = dataset.rasterband(1)?;
        let no_data_value = band.no_data_value();
        let (w, h) = band.size();

        let mem_size = w * h * 4;
        const PRELOAD_THRESHOLD: usize = 4 * 1024 * 1024 * 1024;

        let preloaded_data = if mem_size < PRELOAD_THRESHOLD {
            match band.read_as::<f32>((0, 0), (w, h), (w, h), None) {
                Ok(buffer) => Some(Arc::new(buffer.data().to_vec())),
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(Self {
            path,
            transform,
            needs_half_pixel_shift,
            no_data_value,
            preloaded_data,
            width: w,
            height: h,
        })
    }

    pub fn open_session(&self) -> Result<ReaderSession> {
        let dataset = if self.preloaded_data.is_none() {
            Some(Dataset::open_dataset(&self.path)?)
        } else {
            None
        };

        Ok(ReaderSession {
            source: self.clone(),
            dataset,
            cache: RefCell::new(Vec::with_capacity(8)),
        })
    }
}

const CACHE_BLOCK_SIZE: isize = 512;
const MAX_CACHE_ENTRIES: usize = 8;

struct BlockCache {
    start_col: isize,
    start_row: isize,
    width: usize,
    height: usize,
    data: Vec<f32>,
}

pub struct ReaderSession {
    source: ReaderSource,
    dataset: Option<Dataset>,
    cache: RefCell<Vec<BlockCache>>,
}

impl ReaderSession {
    pub fn sample<T: Interpolator>(&self, lon: f64, lat: f64, strategy: &T) -> Option<f32> {
        let mut px = self.source.transform.geo_to_pixel(lon, lat);

        if self.source.needs_half_pixel_shift {
            px.x -= 0.5;
            px.y -= 0.5;
        }

        strategy.sample(self, px.x, px.y)
    }

    fn read_from_disk(&self, col: isize, row: isize, w: isize, h: isize) -> Option<f32> {
        if let Some(index) = self.find_cache_index(col, row) {
            self.promote_cache_entry(index);
            return self.read_from_cache_head(col, row);
        }

        if self.load_block_to_head(col, row, w, h).is_some() {
            return self.read_from_cache_head(col, row);
        }

        None
    }

    fn find_cache_index(&self, col: isize, row: isize) -> Option<usize> {
        self.cache.borrow().iter().position(|c| {
            col >= c.start_col
                && col < c.start_col + c.width as isize
                && row >= c.start_row
                && row < c.start_row + c.height as isize
        })
    }

    fn promote_cache_entry(&self, index: usize) {
        if index == 0 {
            return;
        }
        let mut cache = self.cache.borrow_mut();
        let block = cache.remove(index);
        cache.insert(0, block);
    }

    fn read_from_cache_head(&self, col: isize, row: isize) -> Option<f32> {
        let cache = self.cache.borrow();
        if cache.is_empty() {
            return None;
        }

        let block = &cache[0];
        let local_col = (col - block.start_col) as usize;
        let local_row = (row - block.start_row) as usize;
        let idx = local_row * block.width + local_col;

        self.apply_nodata_check(block.data[idx])
    }

    fn load_block_to_head(
        &self,
        target_col: isize,
        target_row: isize,
        image_w: isize,
        image_h: isize,
    ) -> Option<()> {
        let ds = self.dataset.as_ref()?;
        let band = ds.rasterband(1).ok()?;

        let start_col = (target_col / CACHE_BLOCK_SIZE) * CACHE_BLOCK_SIZE;
        let start_row = (target_row / CACHE_BLOCK_SIZE) * CACHE_BLOCK_SIZE;

        let block_w = CACHE_BLOCK_SIZE.min(image_w - start_col) as usize;
        let block_h = CACHE_BLOCK_SIZE.min(image_h - start_row) as usize;

        if block_w == 0 || block_h == 0 {
            return None;
        }

        let buffer = band
            .read_as::<f32>(
                (start_col, start_row),
                (block_w, block_h),
                (block_w, block_h),
                None,
            )
            .ok()?;

        let new_block = BlockCache {
            start_col,
            start_row,
            width: block_w,
            height: block_h,
            data: buffer.data().to_vec(),
        };

        let mut cache = self.cache.borrow_mut();
        cache.insert(0, new_block);

        if cache.len() > MAX_CACHE_ENTRIES {
            cache.pop();
        }

        Some(())
    }

    #[inline(always)]
    fn apply_nodata_check(&self, val: f32) -> Option<f32> {
        if let Some(no_data) = self.source.no_data_value {
            if (val - no_data as f32).abs() < 1e-6 {
                return None;
            }
        }
        Some(val)
    }
}

impl PixelSource for ReaderSession {
    fn read_at(&self, col: isize, row: isize) -> Option<f32> {
        let w = self.source.width as isize;
        let h = self.source.height as isize;

        if col < 0 || row < 0 || col >= w || row >= h {
            return None;
        }

        if let Some(data) = &self.source.preloaded_data {
            let idx = (row as usize) * self.source.width + (col as usize);
            return self.apply_nodata_check(data[idx]);
        }

        self.read_from_disk(col, row, w, h)
    }

    fn width(&self) -> usize {
        self.source.width
    }

    fn height(&self) -> usize {
        self.source.height
    }
}
