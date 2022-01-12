use crate::Mat4;

use super::Mat2;
use super::Vec3;

/// column major xyz
#[derive(Debug, Clone, Copy)]
pub struct Mat3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Mat3 {
    #[inline]
    pub fn new(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            x: Vec3::new(self.x.x, self.y.x, self.z.x),
            y: Vec3::new(self.x.y, self.y.y, self.z.y),
            z: Vec3::new(self.x.z, self.y.z, self.z.z),
        }
    }

    #[inline]
    pub fn determinate(&self) -> f32 {
        let a = Mat2::new((self.y.y, self.y.z).into(), (self.z.y, self.z.z).into());
        let b = Mat2::new((self.x.y, self.x.z).into(), (self.z.y, self.z.z).into());
        let c = Mat2::new((self.x.y, self.x.z).into(), (self.y.y, self.y.z).into());

        self.x.x * a.determinate() - self.y.x * b.determinate() + self.z.x * c.determinate()
    }

    #[inline]
    pub fn mat4(self) -> Mat4 {
        self.into()
    }
}

impl From<Mat3> for Mat4 {
    #[inline]
    fn from(three: Mat3) -> Self {
        Mat4::new(
            three.x.vec4_with(0),
            three.y.vec4_with(0),
            three.z.vec4_with(0),
            (0, 0, 0, 1).into(),
        )
    }
}
