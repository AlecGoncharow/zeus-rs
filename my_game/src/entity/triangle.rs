use super::plane::Plane;
use my_engine::math::Vec3;

pub struct Triangle {
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        Self { p1, p2, p3 }
    }

    pub fn plane(&self) -> Option<Plane> {
        Plane::new(self.p1, self.p2, self.p3)
    }
}
