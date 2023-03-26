use super::Dim;
pub use super::Vector;
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
    pub fn from_vec3_with(vec: Vec3, w: impl Into<f64>) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: w.into() as f32,
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
    pub fn zero_out_insignificant(&self, delta: f32) -> Self {
        Self {
            x: if self.x.abs() < delta { 0.0 } else { self.x },
            y: if self.y.abs() < delta { 0.0 } else { self.y },
            z: if self.z.abs() < delta { 0.0 } else { self.z },
            w: if self.w.abs() < delta { 0.0 } else { self.w },
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

    pub fn vec3(&self) -> Vec3 {
        (self.x, self.y, self.z).into()
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

#[cfg(not(any(
    target_feature = "sse",
    target_feature = "sse2",
    target_feature = "neon"
)))]
mod math {
    use super::*;

    impl Vector for Vec4 {
        #[inline]
        fn make_comp_mul(&self, rhs: &Self) -> Self {
            Self {
                x: self.x * rhs.x,
                y: self.y * rhs.y,
                z: self.z * rhs.z,
                w: self.w * rhs.w,
            }
        }

        #[inline]
        fn comp_mul(&mut self, other: &Self) {
            *self = self.make_comp_mul(other);
        }

        #[inline]
        fn make_comp_div(&self, other: &Self) -> Self {
            Self {
                x: self.x / other.x,
                y: self.y / other.y,
                z: self.z / other.z,
                w: self.w / other.w,
            }
        }

        #[inline]
        fn comp_div(&mut self, other: &Self) {
            *self = self.make_comp_div(other);
        }

        #[inline]
        fn dot(&self, other: &Self) -> f32 {
            self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
        }

        #[inline]
        fn clamp(&self, min: impl Into<f32>, max: impl Into<f32>) -> Self {
            let min = min.into();
            let max = max.into();
            Self {
                x: self.x.clamp(min, max),
                y: self.y.clamp(min, max),
                z: self.z.clamp(min, max),
                w: self.w.clamp(min, max),
            }
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
}

#[cfg(any(
    target_feature = "sse",
    target_feature = "sse2",
    target_feature = "neon"
))]
mod math {
    use super::*;
    use crate::simd_vector;
    use std::simd::f32x4;

    simd_vector!(Vec4, f32x4);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn do_math() {
        let mut v1 = Vec4::new(1, 4, 24, 1);
        let mut v2 = Vec4::new(5, 3, -12, 1);
        let sum = v1 + v2;

        assert_eq!(6., sum.x);
        assert_eq!(7., sum.y);
        assert_eq!(12., sum.z);

        v1 += v2;
        assert_eq!(6., v1.x);
        assert_eq!(7., v1.y);
        assert_eq!(12., v1.z);

        let diff = v1 - v2;
        assert_eq!(1., diff.x);
        assert_eq!(4., diff.y);
        assert_eq!(24., diff.z);

        v1 -= v2;
        assert_eq!(1., v1.x);
        assert_eq!(4., v1.y);
        assert_eq!(24., v1.z);

        v1 = Vec4::new(1, 0, 0, 0);
        v2 = Vec4::new(1, 0, 0, 0);

        assert_eq!(1.0, v1.dot(&v2));

        v2.x = 0.;
        v2.y = 1.;

        assert_eq!(0.0, v1.dot(&v2));

        v1 = Vec4::new(1, 5, -100, 1);

        let scaled = 5. * v1;
        assert_eq!(Vec4::new(5, 25, -500, 5), scaled);

        let scaled = v1 * 5.;
        assert_eq!(Vec4::new(5, 25, -500, 5), scaled);

        v1 *= 5.0;
        assert_eq!(Vec4::new(5, 25, -500, 5), v1);
    }
}
