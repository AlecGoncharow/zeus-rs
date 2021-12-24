pub const AMP: f32 = 10.;
pub const ROUGHNESS: f32 = 0.35;
pub const OCTAVES: i32 = 3;

pub const X_SEED_SCALAR: isize = 49_632;
pub const Y_SEED_SCALAR: isize = 325_176;

use rand::Rng;
use rand::SeedableRng;

pub struct Perlin {
    pub seed: isize,
    pub roughness: f32,
    pub octaves: i32,
    pub amplitude: f32,
}

impl Perlin {
    pub fn new(seed: isize, roughness: f32, octaves: i32, amplitude: f32) -> Self {
        Self {
            seed,
            roughness,
            octaves,
            amplitude,
        }
    }

    pub fn rng_seed(roughness: f32, octaves: i32, amplitude: f32) -> Self {
        let mut small_rng = rand::rngs::SmallRng::from_entropy();
        let seed: isize = (small_rng.gen::<f64>() * 1_000_000_000.0) as isize;
        Self::new(seed, roughness, octaves, amplitude)
    }

    pub fn perlin_noise(&self, x: isize, y: isize) -> f32 {
        let mut total: f32 = 0.;
        let d: f32 = 2.0f32.powi(self.octaves - 1);

        for i in 0..self.octaves {
            let freq: f32 = 2.0f32.powi(i) / d;
            let amp: f32 = self.roughness.powi(i) * self.amplitude;
            total += self.interp_noise(x as f32 * freq, y as f32 * freq) * amp;
        }

        total
    }

    fn smooth_noise(&self, x: isize, y: isize) -> f32 {
        let corners = (self.noise(x - 1, y - 1)
            + self.noise(x + 1, y - 1)
            + self.noise(x - 1, y + 1)
            + self.noise(x + 1, y + 1))
            / 16.0;
        let sides = (self.noise(x - 1, y)
            + self.noise(x + 1, y)
            + self.noise(x, y - 1)
            + self.noise(x, y + 1))
            / 8.0;
        let center = self.noise(x, y) / 4.0;

        corners + sides + center
    }

    fn noise(&self, x: isize, y: isize) -> f32 {
        let seed = x * X_SEED_SCALAR + y * Y_SEED_SCALAR + self.seed;
        rand::rngs::SmallRng::seed_from_u64(seed as u64).gen::<f32>() * 2.0 - 1.0
    }

    fn interp_noise(&self, x: f32, y: f32) -> f32 {
        let x_floor = x as isize;
        let frac_x = x - x_floor as f32;
        let y_floor = y as isize;
        let frac_y = y - y_floor as f32;

        let v1 = self.smooth_noise(x_floor, y_floor);
        let v2 = self.smooth_noise(x_floor + 1, y_floor);
        let v3 = self.smooth_noise(x_floor, y_floor + 1);
        let v4 = self.smooth_noise(x_floor + 1, y_floor + 1);

        let i1 = Self::interpolate(v1, v2, frac_x);
        let i2 = Self::interpolate(v3, v4, frac_x);

        Self::interpolate(i1, i2, frac_y)
    }

    fn interpolate(a: f32, b: f32, blend: f32) -> f32 {
        let theta = blend * std::f32::consts::PI;
        let f = (1.0 - theta.cos()) * 0.5;
        a * (1. - f) + b * f
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::rng_seed(ROUGHNESS, OCTAVES, AMP)
    }
}
