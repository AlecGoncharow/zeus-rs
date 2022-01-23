pub use super::Vector;
use crate::Vec3;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use std::simd::f32x2;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub fn new(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            x: x.into() as f32,
            y: y.into() as f32,
        }
    }

    #[inline]
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, 0.0)
    }
}

#[cfg(not(any(
    target_feature = "sse",
    target_feature = "sse2",
    target_feature = "neon"
)))]
mod math {
    use super::*;

    impl Vector for Vec2 {
        #[inline]
        fn dot(&self, other: &Self) -> f32 {
            self.x * other.x + self.y * other.y
        }

        #[inline]
        fn make_comp_mul(&self, other: &Self) -> Self {
            Self {
                x: self.x * other.x,
                y: self.y * other.y,
            }
        }

        #[inline]
        fn comp_mul(&mut self, other: &Self) {
            *self = self.make_comp_mul(other);
        }
    }

    impl Add for Vec2 {
        type Output = Self;

        #[inline]
        fn add(self, other: Self) -> Self {
            Self {
                x: self.x + other.x,
                y: self.y + other.y,
            }
        }
    }

    impl AddAssign for Vec2 {
        #[inline]
        fn add_assign(&mut self, other: Self) {
            *self = Self {
                x: self.x + other.x,
                y: self.y + other.y,
            }
        }
    }

    impl Sub for Vec2 {
        type Output = Self;

        #[inline]
        fn sub(self, other: Self) -> Self {
            Self {
                x: self.x - other.x,
                y: self.y - other.y,
            }
        }
    }

    impl SubAssign for Vec2 {
        #[inline]
        fn sub_assign(&mut self, other: Self) {
            *self = Self {
                x: self.x - other.x,
                y: self.y - other.y,
            }
        }
    }

    impl Mul<Vec2> for f32 {
        type Output = Vec2;

        #[inline]
        fn mul(self, vec: Vec2) -> Vec2 {
            Vec2 {
                x: vec.x * self,
                y: vec.y * self,
            }
        }
    }

    impl<T: Into<f32> + Copy> Mul<T> for Vec2 {
        type Output = Self;

        #[inline]
        fn mul(self, scalar: T) -> Self {
            Self {
                x: self.x * scalar.into(),
                y: self.y * scalar.into(),
            }
        }
    }

    impl<T: Into<f32> + Copy> MulAssign<T> for Vec2 {
        #[inline]
        fn mul_assign(&mut self, scalar: T) {
            *self = Self {
                x: self.x * scalar.into(),
                y: self.y * scalar.into(),
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

    simd_vector!(Vec2, f32x2);
}

impl<T: Into<f64>> From<(T, T)> for Vec2 {
    fn from(tuple: (T, T)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn do_math() {
        let mut v1 = Vec2::new(1, 4);
        let mut v2 = Vec2::new(5, 3);
        let sum = v1 + v2;

        assert_eq!(6., sum.x);
        assert_eq!(7., sum.y);

        v1 += v2;
        assert_eq!(6., v1.x);
        assert_eq!(7., v1.y);

        let diff = v1 - v2;
        assert_eq!(1., diff.x);
        assert_eq!(4., diff.y);

        v1 -= v2;
        assert_eq!(1., v1.x);
        assert_eq!(4., v1.y);

        v1 = Vec2::new(1, 0);
        v2 = Vec2::new(1, 0);

        assert_eq!(1.0, v1.dot(&v2));

        v2.x = 0.;
        v2.y = 1.;

        assert_eq!(0.0, v1.dot(&v2));

        v1 = Vec2::new(1, 5);

        let scaled = 5. * v1;
        assert_eq!(Vec2::new(5, 25), scaled);

        let scaled = v1 * 5.;
        assert_eq!(Vec2::new(5, 25), scaled);

        v1 *= 5.0;
        assert_eq!(Vec2::new(5, 25), v1);
    }
}
