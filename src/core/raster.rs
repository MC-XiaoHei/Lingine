pub trait PixelSource {
    fn read_at(&self, col: isize, row: isize) -> Option<f32>;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait Interpolator: Send + Sync {
    fn sample(&self, source: &dyn PixelSource, u: f64, v: f64) -> Option<f32>;
}

pub struct NearestNeighbor;

impl Interpolator for NearestNeighbor {
    fn sample(&self, source: &dyn PixelSource, u: f64, v: f64) -> Option<f32> {
        let col = u.round() as isize;
        let row = v.round() as isize;
        source.read_at(col, row)
    }
}

pub struct Bilinear;

impl Interpolator for Bilinear {
    fn sample(&self, source: &dyn PixelSource, u: f64, v: f64) -> Option<f32> {
        let x = u.floor();
        let y = v.floor();
        let u_ratio = (u - x) as f32;
        let v_ratio = (v - y) as f32;

        let col = x as isize;
        let row = y as isize;

        let top_left = source.read_at(col, row)?;
        let top_right = source.read_at(col + 1, row)?;
        let bot_left = source.read_at(col, row + 1)?;
        let bot_right = source.read_at(col + 1, row + 1)?;

        let top = top_left + (top_right - top_left) * u_ratio;
        let bot = bot_left + (bot_right - bot_left) * u_ratio;

        Some(top + (bot - top) * v_ratio)
    }
}

pub struct Bicubic;

impl Interpolator for Bicubic {
    fn sample(&self, source: &dyn PixelSource, u: f64, v: f64) -> Option<f32> {
        let x = u.floor();
        let y = v.floor();
        let dx = (u - x) as f32;
        let dy = (v - y) as f32;
        let col = x as isize;
        let row = y as isize;

        if is_boundary(col, row, source.width(), source.height()) {
            return Bilinear.sample(source, u, v);
        }

        let mut window = [[0.0; 4]; 4];

        for j in 0..4 {
            for i in 0..4 {
                if let Some(val) = source.read_at(col - 1 + i, row - 1 + j) {
                    window[j as usize][i as usize] = val;
                } else {
                    return None;
                }
            }
        }

        let mut col_results = [0.0; 4];
        for j in 0..4 {
            col_results[j] = cubic_hermite(window[j], dx);
        }
        Some(cubic_hermite(col_results, dy))
    }
}

fn is_boundary(col: isize, row: isize, w: usize, h: usize) -> bool {
    col < 1 || row < 1 || col >= (w as isize - 2) || row >= (h as isize - 2)
}

fn cubic_hermite(p: [f32; 4], t: f32) -> f32 {
    let a = -0.5 * p[0] + 1.5 * p[1] - 1.5 * p[2] + 0.5 * p[3];
    let b = p[0] - 2.5 * p[1] + 2.0 * p[2] - 0.5 * p[3];
    let c = -0.5 * p[0] + 0.5 * p[2];
    let d = p[1];

    a * t * t * t + b * t * t + c * t + d
}
