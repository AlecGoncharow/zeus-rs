use super::Dim;
use super::Mat3;
use crate::math::Vec3;
use crate::math::Vec4;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// Column major xyzw
#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Mat4 {
    #[inline]
    pub fn new(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn identity() -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            x: Vec4::new(self.x.x, self.y.x, self.z.x, self.w.x),
            y: Vec4::new(self.x.y, self.y.y, self.z.y, self.w.y),
            z: Vec4::new(self.x.z, self.y.z, self.z.z, self.w.z),
            w: Vec4::new(self.x.w, self.y.w, self.z.w, self.w.w),
        }
    }

    /// https://sites.google.com/site/glennmurray/Home/rotation-matrices-and-formulas/rotation-about-an-arbitrary-axis-in-3-dimensions
    pub fn rotation(theta: f64, axis: Vec3) -> Self {
        let axis = axis.make_unit_vector();
        let u = axis.x;
        let v = axis.y;
        let w = axis.z;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        /* ROW MAJOR
        Self {
            x: Vec4::new(
                u * u + (1.0 - u * u) * cos_theta,
                u * v * (1.0 - cos_theta) - w * sin_theta,
                u * w * (1.0 - cos_theta) + v * sin_theta,
                0.0,
            )
            .zero_out_insignificant(0.00005),
            y: Vec4::new(
                v * u * (1.0 - cos_theta) + w * sin_theta,
                v * v + (1.0 - v * v) * cos_theta,
                v * w * (1.0 - cos_theta) - u * sin_theta,
                0.0,
            )
            .zero_out_insignificant(0.00005),
            z: Vec4::new(
                w * u * (1.0 - cos_theta) - v * sin_theta,
                w * v * (1.0 - cos_theta) + u * sin_theta,
                w * w + (1.0 - w * w) * cos_theta,
                0.0,
            )
            .zero_out_insignificant(0.00005),
            w: Vec4::new(0, 0, 0, 1),
        }
        */

        // column major
        Self {
            x: Vec4::new(
                u * u + (1.0 - u * u) * cos_theta,
                v * u * (1.0 - cos_theta) + w * sin_theta,
                w * u * (1.0 - cos_theta) - v * sin_theta,
                0.0,
            )
            .zero_out_insignificant(0.00005),
            y: Vec4::new(
                u * v * (1.0 - cos_theta) - w * sin_theta,
                v * v + (1.0 - v * v) * cos_theta,
                w * v * (1.0 - cos_theta) + u * sin_theta,
                0.0,
            )
            .zero_out_insignificant(0.00005),
            z: Vec4::new(
                u * w * (1.0 - cos_theta) + v * sin_theta,
                v * w * (1.0 - cos_theta) - u * sin_theta,
                w * w + (1.0 - w * w) * cos_theta,
                0.0,
            )
            .zero_out_insignificant(0.00005),
            w: Vec4::new(0, 0, 0, 1),
        }
    }

    pub fn rotation_from_degrees(degrees: f64, axis: Vec3) -> Self {
        Self::rotation(degrees.to_radians(), axis)
    }
    #[inline]
    pub fn scalar(
        x_scalar: impl Into<f64>,
        y_scalar: impl Into<f64>,
        z_scalar: impl Into<f64>,
    ) -> Self {
        Self {
            x: Vec4::new(x_scalar, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, y_scalar, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, z_scalar, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    #[inline]
    pub fn scalar_from_one(scalar: impl Into<f64> + Copy) -> Self {
        Self::scalar(scalar, scalar, scalar)
    }

    #[inline]
    pub fn translation<T>((x_tr, y_tr, z_tr): (T, T, T)) -> Self
    where
        T: Into<f64>,
    {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(x_tr, y_tr, z_tr, 1.0),
        }
    }

    #[inline]
    pub fn matrix_of_minors(&self) -> Self {
        Self {
            x: Vec4 {
                x: Mat3::new(
                    self.y.truncate(Dim::X),
                    self.z.truncate(Dim::X),
                    self.w.truncate(Dim::X),
                )
                .determinate(),
                y: Mat3::new(
                    self.y.truncate(Dim::Y),
                    self.z.truncate(Dim::Y),
                    self.w.truncate(Dim::Y),
                )
                .determinate(),
                z: Mat3::new(
                    self.y.truncate(Dim::Z),
                    self.z.truncate(Dim::Z),
                    self.w.truncate(Dim::Z),
                )
                .determinate(),
                w: Mat3::new(
                    self.y.truncate(Dim::W),
                    self.z.truncate(Dim::W),
                    self.w.truncate(Dim::W),
                )
                .determinate(),
            },

            y: Vec4 {
                x: Mat3::new(
                    self.x.truncate(Dim::X),
                    self.z.truncate(Dim::X),
                    self.w.truncate(Dim::X),
                )
                .determinate(),
                y: Mat3::new(
                    self.x.truncate(Dim::Y),
                    self.z.truncate(Dim::Y),
                    self.w.truncate(Dim::Y),
                )
                .determinate(),

                z: Mat3::new(
                    self.x.truncate(Dim::Z),
                    self.z.truncate(Dim::Z),
                    self.w.truncate(Dim::Z),
                )
                .determinate(),
                w: Mat3::new(
                    self.x.truncate(Dim::W),
                    self.z.truncate(Dim::W),
                    self.w.truncate(Dim::W),
                )
                .determinate(),
            },

            z: Vec4 {
                x: Mat3::new(
                    self.x.truncate(Dim::X),
                    self.y.truncate(Dim::X),
                    self.w.truncate(Dim::X),
                )
                .determinate(),
                y: Mat3::new(
                    self.x.truncate(Dim::Y),
                    self.y.truncate(Dim::Y),
                    self.w.truncate(Dim::Y),
                )
                .determinate(),
                z: Mat3::new(
                    self.x.truncate(Dim::Z),
                    self.y.truncate(Dim::Z),
                    self.w.truncate(Dim::Z),
                )
                .determinate(),
                w: Mat3::new(
                    self.x.truncate(Dim::W),
                    self.y.truncate(Dim::W),
                    self.w.truncate(Dim::W),
                )
                .determinate(),
            },

            w: Vec4 {
                x: Mat3::new(
                    self.x.truncate(Dim::X),
                    self.y.truncate(Dim::X),
                    self.z.truncate(Dim::X),
                )
                .determinate(),
                y: Mat3::new(
                    self.x.truncate(Dim::Y),
                    self.y.truncate(Dim::Y),
                    self.z.truncate(Dim::Y),
                )
                .determinate(),
                z: Mat3::new(
                    self.x.truncate(Dim::Z),
                    self.y.truncate(Dim::Z),
                    self.z.truncate(Dim::Z),
                )
                .determinate(),
                w: Mat3::new(
                    self.x.truncate(Dim::W),
                    self.y.truncate(Dim::W),
                    self.z.truncate(Dim::W),
                )
                .determinate(),
            },
        }
    }

    #[inline]
    pub fn matrix_of_cofactors(&self) -> Self {
        let mut mat = *self;

        mat.x.y *= -1.0;
        mat.x.w *= -1.0;

        mat.y.x *= -1.0;
        mat.y.z *= -1.0;

        mat.z.y *= -1.0;
        mat.z.w *= -1.0;

        mat.w.x *= -1.0;
        mat.w.z *= -1.0;

        mat
    }

    // based on https://www.mathsisfun.com/algebra/matrix-inverse-minors-cofactors-adjugate.html
    pub fn invert(&self) -> Option<Self> {
        let minors = self.matrix_of_minors();
        let cofactors = minors.matrix_of_cofactors();

        let determinate = self.x.x * cofactors.x.x
            + self.y.x * cofactors.y.x
            + self.z.x * cofactors.z.x
            + self.w.x * cofactors.w.x;

        if determinate == 0.0 {
            return None;
        }

        Some(1.0 / determinate * cofactors.transpose())
    }
}

impl Add for Mat4 {
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

impl AddAssign for Mat4 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Mat4 {
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

impl SubAssign for Mat4 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul<Mat4> for f64 {
    type Output = Mat4;

    fn mul(self, mat: Mat4) -> Mat4 {
        Mat4 {
            x: mat.x * self,
            y: mat.y * self,
            z: mat.z * self,
            w: mat.w * self,
        }
    }
}

impl<T: Into<f64> + Copy> Mul<T> for Mat4 {
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

impl<T: Into<f64> + Copy> MulAssign<T> for Mat4 {
    fn mul_assign(&mut self, scalar: T) {
        *self = Self {
            x: self.x * scalar.into(),
            y: self.y * scalar.into(),
            z: self.z * scalar.into(),
            w: self.w * scalar.into(),
        }
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            x: &self * rhs.x,
            y: &self * rhs.y,
            z: &self * rhs.z,
            w: &self * rhs.w,
        }
    }
}

// follows real math semantics, vector has to be rhs
impl Mul<Vec4> for &Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        let self_tr = self.transpose();
        Vec4 {
            x: self_tr.x.dot(&rhs),
            y: self_tr.y.dot(&rhs),
            z: self_tr.z.dot(&rhs),
            w: self_tr.w.dot(&rhs),
        }
    }
}

// TODO think about how expensive this op is
impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        let self_tr = self.transpose();
        Vec4 {
            x: self_tr.x.dot(&rhs),
            y: self_tr.y.dot(&rhs),
            z: self_tr.z.dot(&rhs),
            w: self_tr.w.dot(&rhs),
        }
    }
}

///T,T
impl<T: Into<f64>> From<(((T, T, T, T), (T, T, T, T), (T, T, T, T), (T, T, T, T)))> for Mat4 {
    fn from(tuple: ((T, T, T, T), (T, T, T, T), (T, T, T, T), (T, T, T, T))) -> Self {
        Self::new(
            tuple.0.into(),
            tuple.1.into(),
            tuple.2.into(),
            tuple.3.into(),
        )
    }
}

impl From<Mat4> for [[f32; 4]; 4] {
    fn from(mat: Mat4) -> [[f32; 4]; 4] {
        [
            [
                mat.x.x as f32,
                mat.x.y as f32,
                mat.x.z as f32,
                mat.x.w as f32,
            ],
            [
                mat.y.x as f32,
                mat.y.y as f32,
                mat.y.z as f32,
                mat.y.w as f32,
            ],
            [
                mat.z.x as f32,
                mat.z.y as f32,
                mat.z.z as f32,
                mat.z.w as f32,
            ],
            [
                mat.w.x as f32,
                mat.w.y as f32,
                mat.w.z as f32,
                mat.w.w as f32,
            ],
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::math::*;
    #[test]
    fn test_mult() {
        let vec = Vec4::new(1.0, 2, 3.0, 1);

        let mat = Mat4::scalar_from_one(5.0);

        println!("{:#?}", mat * vec);

        /*
         *  should output:
         *  [5.0, 10.0, 15.0, 1.0]
         */

        let mat = Mat4::scalar_from_one(5.0);
        let rhs = Mat4::identity();

        /*
         *  should output:
         *  [5.0, 0.0, 0.0, 0.0]
         *  [0.0, 5.0, 0.0, 0.0]
         *  [0.0, 0.0, 5.0, 0.0]
         *  [0.0, 0.0, 0.0, 1.0]
         */
        println!("{:#?}", mat * rhs);

        let lhs: Mat4 = (
            (5.0, 7.0, 9.0, 10.0),
            (2.0, 3.0, 3.0, 8.0),
            (8.0, 10.0, 2.0, 3.0),
            (3.0, 3.0, 4.0, 8.0),
        )
            .into();

        let rhs = Mat4::new(
            (3, 10, 12, 18).into(),
            (12, 1, 4, 9).into(),
            (9, 10, 12, 2).into(),
            (3, 12, 4, 10).into(),
        );

        /*
         *  should output:
         *  [210, 267, 236, 271]
         *  [ 93, 149, 104, 149]
         *  [171, 146, 172, 268]
         *  [105, 169, 128, 169]
         */

        println!("{:#?}", lhs * rhs);

        let vec = Vec4::new(1.0, 2, 3.0, 1);

        let mat = Mat4::translation((4, 10, 25).into());

        println!("trans: {:#?}", mat * vec);
    }

    #[test]
    fn test_rotation() {
        let vec = Vec4::new(1, 0, 0, 1);

        let rotation = Mat4::rotation_from_degrees(90.0, (0, 0, 1).into());

        println!(
            "rotation mat: {:#?},  output:{:#?}",
            rotation,
            rotation * vec
        );

        let vec = Vec4::new(1, 0, 0, 1);

        let rotation = Mat4::rotation_from_degrees(53.1, (0, 0, 1).into());

        println!(
            "rotation mat: {:#?},  output:{:#?}",
            rotation,
            rotation * vec
        );
    }

    #[test]
    fn invert() {
        let mat: Mat4 = (
            (4.0, 0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0, 0.0),
            (0.0, 2.0, 2.0, 0.0),
            (0.0, 0.0, 0.0, 1.0),
        )
            .into();

        /*
         *  should output:
         *  [ 0.25, 0.0, 0.0, 0.0]
         *  [ 0.0 ,-1.0, 1.0, 0.0]
         *  [ 0.0 , 0.5, 0.0, 0.0]
         *  [-0.25, 0.0, 0.0, 1.0]
         */
        println!("invert: {:#?}, out: {:#?}", mat, mat.invert());
    }
}
