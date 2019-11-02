use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[inline]
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    #[inline]
    pub fn new_from_one(x: impl Into<f64> + Copy) -> Self {
        Self::new(x, x, x)
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
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
    pub fn refract(&self, n: &Self, ni_over_nt: impl Into<f64> + Copy) -> Option<Self> {
        let ni_over_nt = ni_over_nt.into();
        let uv = self.make_unit_vector();
        let dt = uv.dot(n);
        let discriminant = 1.0 - ((ni_over_nt * ni_over_nt) * (1.0 - (dt * dt)));
        if discriminant > 0.0 {
            // source https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
            // @TODO understand what this is actually calculating
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
    pub fn clamp(&self, min: f64, max: f64) -> Self {
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
        }
    }
}

impl<T: Into<f64>> From<(T, T, T)> for Vec3 {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
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

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Vec3 {
        Vec3 {
            x: vec.x * self,
            y: vec.y * self,
            z: vec.z * self,
        }
    }
}

impl<T: Into<f64> + Copy> Mul<T> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
        }
    }
}

impl<T: Into<f64> + Copy> MulAssign<T> for Vec3 {
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

        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());

        let vec = Vec3::new(0.1, 0.0, 0.1);

        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        let vec = Vec3::new(0.1, 0.1, 0.1);

        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        let vec = Vec3::new(10, 0.0, 0.0);

        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());

        let vec = Vec3::new(10, 0.0, 10);

        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
        let vec = Vec3::new(10, 10, 10);

        println!("{:#?}, {:#?}", vec, vec.make_unit_vector());
    }
}
