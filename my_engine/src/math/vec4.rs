use crate::math::Vec3;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_vec3(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: 1.0,
        }
    }

    #[inline]
    pub fn new_from_one(x: f32) -> Self {
        Self::new(x, x, x, x)
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
    pub fn clamp(&self, min: f32, max: f32) -> Self {
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

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(tuple: (f32, f32, f32, f32)) -> Self {
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

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, scalar: f32) {
        *self = Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}
