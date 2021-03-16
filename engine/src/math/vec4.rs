use super::Dim;
use crate::math::Vec3;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[inline]
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>, w: impl Into<f64>) -> Self {
        Self {
            x: x.into() as f32,
            y: y.into() as f32,
            z: z.into() as f32,
            w: w.into() as f32,
        }
    }
    #[inline]
    pub fn new_from_one(x: impl Into<f64> + Copy) -> Self {
        Self::new(x, x, x, x)
    }

    #[inline]
    pub fn from_vec3(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: 1.0,
        }
    }
    #[inline]
    pub fn gamma_two(&self) -> Self {
        Self {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
            w: self.w.sqrt(),
        }
    }

    #[inline]
    pub fn make_comp_mul(&self, rhs: &Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }

    #[inline]
    pub fn comp_mul(&mut self, other: &Self) {
        *self = self.make_comp_mul(other);
    }

    #[inline]
    pub fn make_comp_div(&self, other: &Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
            w: self.w / other.w,
        }
    }

    #[inline]
    pub fn comp_div(&mut self, other: &Self) {
        *self = self.make_comp_div(other);
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /*
    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }
    */
    #[inline]
    pub fn squared_mag(&self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.squared_mag().sqrt()
    }

    pub fn make_unit_vector(&self) -> Self {
        let scalar = 1.0 / self.magnitude();
        scalar * *self
    }

    #[inline]
    pub fn normalize(&mut self) {
        *self = self.make_unit_vector();
    }

    #[inline]
    pub fn zero_out_insignificant(&self, delta: f32) -> Self {
        Self {
            x: if self.x.abs() < delta { 0.0 } else { self.x },
            y: if self.y.abs() < delta { 0.0 } else { self.y },
            z: if self.z.abs() < delta { 0.0 } else { self.z },
            w: if self.w.abs() < delta { 0.0 } else { self.w },
        }
    }

    #[inline]
    pub fn clamp(&self, min: impl Into<f32>, max: impl Into<f32>) -> Self {
        let min = min.into();
        let max = max.into();
        Self {
            x: if self.x < min {
                min
            } else if self.x > max {
                max
            } else {
                self.x
            },
            y: if self.y < min {
                min
            } else if self.y > max {
                max
            } else {
                self.y
            },
            z: if self.z < min {
                min
            } else if self.z > max {
                max
            } else {
                self.z
            },
            w: if self.w < min {
                min
            } else if self.w > max {
                max
            } else {
                self.w
            },
        }
    }

    pub fn truncate(&self, dim: Dim) -> Vec3 {
        match dim {
            Dim::X => (self.y, self.z, self.w).into(),
            Dim::Y => (self.x, self.z, self.w).into(),
            Dim::Z => (self.x, self.y, self.w).into(),
            Dim::W => (self.x, self.y, self.z).into(),
        }
    }
}

impl<T: Into<f64>> From<(T, T, T, T)> for Vec4 {
    fn from(tuple: (T, T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}

impl From<Vec4> for [f32; 4] {
    fn from(vec: Vec4) -> Self {
        [vec.x as f32, vec.y as f32, vec.z as f32, vec.w as f32]
    }
}

impl Add for Vec4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Vec4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self,
            y: vec.y * self,
            z: vec.z * self,
            w: vec.w * self,
        }
    }
}

impl Mul<Vec4> for f64 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self as f32,
            y: vec.y * self as f32,
            z: vec.z * self as f32,
            w: vec.w * self as f32,
        }
    }
}

impl Mul<Vec4> for i32 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self as f32,
            y: vec.y * self as f32,
            z: vec.z * self as f32,
            w: vec.w * self as f32,
        }
    }
}

impl Mul<Vec4> for u32 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self as f32,
            y: vec.y * self as f32,
            z: vec.z * self as f32,
            w: vec.w * self as f32,
        }
    }
}

impl<T: Into<f32> + Copy> Mul<T> for Vec4 {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
            w: self.w * scalar.into(),
        }
    }
}

impl<T: Into<f32> + Copy> MulAssign<T> for Vec4 {
    fn mul_assign(&mut self, scalar: T) {
        *self = Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
            w: self.w * scalar.into(),
        }
    }
}
