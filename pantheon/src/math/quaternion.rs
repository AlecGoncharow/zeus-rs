use super::Vec3;

use std::ops::{Add, Mul};

#[derive(Debug, Copy, Clone)]
pub struct Quaternion {
    pub vector: Vec3,
    pub scalar: f32,
}

impl Quaternion {
    /// returns unit Quaternion
    #[inline]
    pub fn rotation(radians: f32, axis: Vec3) -> Self {
        let axis = axis.unit_vector();
        let r_2 = radians / 2.0;

        Self {
            vector: r_2.sin() * axis,
            scalar: r_2.cos(),
        }
    }

    /// returns unit Quaternion
    #[inline]
    pub fn rotation_from_degrees(degrees: f32, axis: Vec3) -> Self {
        Self::rotation(degrees.to_radians(), axis)
    }

    #[inline]
    pub fn from_parts(vector: Vec3, scalar: f32) -> Self {
        Self { vector, scalar }
    }

    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            vector: -1.0 * self.vector,
            scalar: self.scalar,
        }
    }

    #[inline]
    pub fn squared_mag(&self) -> f32 {
        self.vector.squared_mag() + self.scalar.powi(2)
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.squared_mag().sqrt()
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let inv_sq_mag = 1.0 / self.squared_mag();
        Self {
            vector: -1.0 * inv_sq_mag * self.vector,
            scalar: inv_sq_mag * self.scalar,
        }
    }

    #[inline]
    pub fn rotate(&self, point: &Vec3) -> Vec3 {
        let q_point = Quaternion::from_parts(*point, 0.0);
        let conjugate = self.conjugate();
        let v_prime = *self * q_point * conjugate;
        v_prime.vector
    }

    #[inline]
    pub fn lerp(&self, to: &Self, t: f32) -> Self {
        t * self + (1.0 - t) * to
    }

    #[inline]
    pub fn nlerp(&self, to: &Self, t: f32) -> Self {
        let r = self.lerp(to, t);

        (1.0 / r.magnitude()) * r
    }
}

impl Mul<Quaternion> for f32 {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        Self::Output {
            vector: self * rhs.vector,
            scalar: self * rhs.scalar,
        }
    }
}

impl Mul<&Quaternion> for f32 {
    type Output = Quaternion;

    fn mul(self, rhs: &Quaternion) -> Self::Output {
        Self::Output {
            vector: self * rhs.vector,
            scalar: self * rhs.scalar,
        }
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            vector: self.vector.cross(&rhs.vector)
                + self.scalar * rhs.vector
                + rhs.scalar * self.vector,
            scalar: self.scalar * rhs.scalar - self.vector.dot(&rhs.vector),
        }
    }
}

impl Mul for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            vector: self.vector.cross(&rhs.vector)
                + self.scalar * rhs.vector
                + rhs.scalar * self.vector,
            scalar: self.scalar * rhs.scalar - self.vector.dot(&rhs.vector),
        }
    }
}

impl Add for Quaternion {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            vector: self.vector + rhs.vector,
            scalar: self.scalar + rhs.scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conjugate() {
        let q1 = Quaternion::rotation_from_degrees(45.2, (1, 4, 2).into());
        println!("{:#?}", q1);
        let sq_mag = q1.squared_mag();

        let conj = q1.conjugate();

        let q1_conj = q1 * conj;

        println!("{:#?}, {:#?}", sq_mag, q1_conj);
        assert_eq!(
            q1.vector.unit_vector(),
            q1.vector.unit_vector().unit_vector()
        );
        assert_eq!(sq_mag, q1_conj.scalar);

        let q1 = Quaternion::rotation_from_degrees(45.2, (1, 0, 0).into());
        println!("{:#?}", q1);

        let sq_mag = q1.squared_mag();

        let conj = q1.conjugate();

        let q1_conj = q1 * conj;

        println!("{:#?}, {:#?}", sq_mag, q1_conj);
        assert_eq!(sq_mag, q1_conj.scalar);
    }

    #[test]
    fn test_rotation() {
        let point = Vec3::new(1, 0, 0);

        let rotation = Quaternion::rotation_from_degrees(90.0, (0, 0, 1).into());

        let expected = Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        println!(
            "rotation quat: {:#?},  output:{:#?} expected: {:#?}",
            rotation,
            rotation.rotate(&point),
            expected
        );

        println!(
            "diff: {:#?}",
            (expected - rotation.rotate(&point)).magnitude()
        );
        assert!(expected.approx_eq(&rotation.rotate(&point)));

        let point = Vec3::new(1, 0, 0);

        let rotation = Quaternion::rotation_from_degrees(53.1, (0, 1, 0).into());

        let expected = Vec3 {
            x: 0.60042024,
            y: 0.0,
            z: -0.79968464,
        };
        println!(
            "rotation quat: {:#?},  output:{:#?} expected: {:#?}",
            rotation,
            rotation.rotate(&point),
            expected
        );

        println!(
            "diff: {:#?}",
            (expected - rotation.rotate(&point)).magnitude()
        );
        assert!(expected.approx_eq(&rotation.rotate(&point)));
    }

    #[test]
    fn test_lerp() {
        // @TODO
    }
}
