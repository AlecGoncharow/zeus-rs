use crate::math::Vec4;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

// @TODO think about vector equality, may need to implement floating point epsilon comparisons
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline]
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Self {
        Self {
            x: x.into() as f32,
            y: y.into() as f32,
            z: z.into() as f32,
        }
    }

    #[inline]
    pub fn new_from_one(x: impl Into<f64> + Copy) -> Self {
        Self::new(x, x, x)
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }

    #[inline]
    pub fn gamma_two(&self) -> Self {
        Self {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
        }
    }

    #[inline]
    pub fn refract(&self, n: &Self, ni_over_nt: impl Into<f32> + Copy) -> Option<Self> {
        let ni_over_nt = ni_over_nt.into();
        let uv = self.unit_vector();
        let dt = uv.dot(n);
        let discriminant = 1.0 - ((ni_over_nt * ni_over_nt) * (1.0 - (dt * dt)));
        if discriminant > 0.0 {
            // source https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
            Some((ni_over_nt * (uv - (dt * *n))) - (discriminant.sqrt() * *n))
        } else {
            None
        }
    }

    #[inline]
    pub fn reflect(&self, orthogonal_unit_vector: &Self) -> Self {
        *self - (2.0 * (self.dot(orthogonal_unit_vector) * *orthogonal_unit_vector))
    }

    #[inline]
    pub fn make_comp_mul(&self, other: &Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
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

    /// checked normalization, only performs scalar multiply of vector if not already mag ~= 1.0
    #[inline]
    pub fn unit_vector(&self) -> Self {
        let mag = self.magnitude();

        if (mag - 1.0).abs() > f32::EPSILON {
            let scalar = 1.0 / mag;
            scalar * *self
        } else {
            *self
        }
    }

    #[inline]
    pub fn make_unit_vector(&self) -> Self {
        let scalar = 1.0 / self.magnitude();
        scalar * *self
    }

    #[inline]
    pub fn normalize(&mut self) {
        *self = self.unit_vector();
    }

    #[inline]
    pub fn vec4(self) -> Vec4 {
        Vec4::from_vec3(self)
    }

    #[inline]
    pub fn clamp(&self, min: f32, max: f32) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
            z: self.z.clamp(min, max),
        }
    }

    #[inline]
    pub fn approx_eq(&self, other: &Self) -> bool {
        (*self - *other).magnitude() <= f32::EPSILON
    }
}

impl<T: Into<f64>> From<(T, T, T)> for Vec3 {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

impl From<Vec3> for (f32, f32, f32) {
    fn from(vec: Vec3) -> Self {
        (vec.x, vec.y, vec.z)
    }
}

impl From<Vec3> for (f64, f64, f64) {
    fn from(vec: Vec3) -> Self {
        (vec.x as f64, vec.y as f64, vec.z as f64)
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(vec: Vec3) -> Self {
        [vec.x, vec.y, vec.z]
    }
}

impl From<Vec3> for [f64; 3] {
    fn from(vec: Vec3) -> Self {
        [vec.x as f64, vec.y as f64, vec.z as f64]
    }
}

impl From<Vec3> for Vec4 {
    fn from(vec: Vec3) -> Self {
        Self::new(vec.x, vec.y, vec.z, 1.0)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Vec3 {
        Vec3 {
            x: vec.x * self,
            y: vec.y * self,
            z: vec.z * self,
        }
    }
}

impl<T: Into<f32> + Copy> Mul<T> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
        }
    }
}

impl<T: Into<f32> + Copy> MulAssign<T> for Vec3 {
    fn mul_assign(&mut self, scalar: T) {
        *self = Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::*;

    #[test]
    fn test_norm() {
        let vec = Vec3::new(0.1, 0.0, 0.0);
        let expected = Vec3::new(1, 0, 0);
        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        assert_eq!(expected, vec.make_unit_vector());

        let vec = Vec3::new(0.1, 0.0, 0.1);
        let expected = Vec3 {
            x: 0.70710677,
            y: 0.0,
            z: 0.70710677,
        };
        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        assert_eq!(expected, vec.make_unit_vector());

        let vec = Vec3::new(0.1, 0.1, 0.1);
        let expected = Vec3 {
            x: 0.5773503,
            y: 0.5773503,
            z: 0.5773503,
        };
        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        assert_eq!(expected, vec.make_unit_vector());

        let vec = Vec3::new(10, 0.0, 0.0);
        let expected = Vec3::new(1, 0, 0);
        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        assert_eq!(expected, vec.make_unit_vector());

        let vec = Vec3::new(10, 0.0, 10);
        let expected = Vec3 {
            x: 0.7071068,
            y: 0.0,
            z: 0.7071068,
        };
        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        assert_eq!(expected, vec.make_unit_vector());

        let vec = Vec3::new(10, 10, 10);
        let expected = Vec3 {
            x: 0.5773502,
            y: 0.5773502,
            z: 0.5773502,
        };
        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        assert_eq!(expected, vec.make_unit_vector());
    }

    #[test]
    fn test_approx_eq() {
        let unit = Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        let approx_unit = Vec3 {
            x: 0.0,
            y: 0.99999994,
            z: 0.0,
        };

        assert!(approx_unit.approx_eq(&unit));

        let unit = Vec3 {
            x: 0.60042024,
            y: 0.0,
            z: -0.79968464,
        };

        let approx_unit = Vec3 {
            x: 0.6004203,
            y: 0.0,
            z: -0.79968464,
        };

        assert!(approx_unit.approx_eq(&unit));

        let unit = Vec3 {
            x: -0.79968464,
            y: 0.0,
            z: 0.60042024,
        };

        let approx_unit = Vec3 {
            x: 0.6004203,
            y: -0.79968464,
            z: 0.0,
        };

        assert!(!approx_unit.approx_eq(&unit));
    }
}
