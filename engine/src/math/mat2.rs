use super::Vec2;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// Column major
pub struct Mat2 {
    pub x: Vec2,
    pub y: Vec2,
}

impl Mat2 {
    #[inline]
    pub fn new(x: Vec2, y: Vec2) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn determinate(&self) -> f32 {
        (self.x.x * self.y.y) - (self.x.y * self.y.x)
    }
}

impl Add for Mat2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Mat2 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Mat2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Mat2 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<Mat2> for f32 {
    type Output = Mat2;

    fn mul(self, mat: Mat2) -> Mat2 {
        Mat2 {
            x: mat.x * self,
            y: mat.y * self,
        }
    }
}

impl<T: Into<f32> + Copy> Mul<T> for Mat2 {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
        }
    }
}

impl<T: Into<f32> + Copy> MulAssign<T> for Mat2 {
    fn mul_assign(&mut self, scalar: T) {
        *self = Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
        }
    }
}
