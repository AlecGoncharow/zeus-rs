use my_engine::math::Mat3;
use my_engine::math::Vec3;

pub struct Plane {
    pub point: Vec3,
    pub norm: Vec3,
}

impl Plane {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3) -> Option<Self> {
        let norm = (p1 - p0).cross(&(p2 - p0)).make_unit_vector();

        if norm.magnitude() == 0.0 {
            None
        } else {
            Some(Plane { point: p1, norm })
        }
    }
}
