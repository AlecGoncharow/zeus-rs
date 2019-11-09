use my_engine::math::Vec3;

pub struct Plane {
    pub point: Vec3,
    pub norm: Vec3,
}

impl Plane {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Option<Self> {
        let norm = (p2 - p1).cross(&(p3 - p1));

        if norm.magnitude() == 0.0 {
            None
        } else {
            Some(Plane { point: p1, norm })
        }
    }
}
