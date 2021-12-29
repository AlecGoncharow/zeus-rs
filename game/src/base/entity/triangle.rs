use super::plane::Plane;
use pantheon::math::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub p0: Vec3,
    pub p1: Vec3,
    pub p2: Vec3,
}

impl Triangle {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3) -> Self {
        Self { p0, p1, p2 }
    }

    #[allow(dead_code)]
    pub fn plane(&self) -> Option<Plane> {
        Plane::new(self.p0, self.p1, self.p2)
    }
}
