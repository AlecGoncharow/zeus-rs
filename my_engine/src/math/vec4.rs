use crate::math::Vec3;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vec4 {
    #[inline]
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>, w: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            w: w.into(),
        }
    }
    #[inline]
    pub fn new_from_one(x: impl Into<f64> + Copy) -> Self {
        Self::new(x, x, x, x)
    }
    /*
    pub fn from_vec3(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: 1.0,
        }
    }
    */
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
    pub fn make_comp_mul(&self, other: &Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
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
    pub fn dot(&self, other: &Self) -> f64 {
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
    pub fn squared_mag(&self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
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
    pub fn clamp(&self, min: impl Into<f64>, max: impl Into<f64>) -> Self {
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
}

impl<T: Into<f64>> From<(T, T, T, T)> for Vec4 {
    fn from(tuple: (T, T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2, tuple.3)
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

impl Mul<Vec4> for f64 {
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

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self as f64,
            y: vec.y * self as f64,
            z: vec.z * self as f64,
            w: vec.w * self as f64,
        }
    }
}

impl Mul<Vec4> for i32 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self as f64,
            y: vec.y * self as f64,
            z: vec.z * self as f64,
            w: vec.w * self as f64,
        }
    }
}

impl Mul<Vec4> for u32 {
    type Output = Vec4;

    fn mul(self, vec: Vec4) -> Vec4 {
        Vec4 {
            x: vec.x * self as f64,
            y: vec.y * self as f64,
            z: vec.z * self as f64,
            w: vec.w * self as f64,
        }
    }
}

impl<T: Into<f64> + Copy> Mul<T> for Vec4 {
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

impl<T: Into<f64> + Copy> MulAssign<T> for Vec4 {
    fn mul_assign(&mut self, scalar: T) {
        *self = Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
            w: self.w * scalar.into(),
        }
    }
}
