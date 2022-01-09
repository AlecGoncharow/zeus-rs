use pantheon::math::prelude::*;
use pantheon::Color;

pub struct Infinite {
    pub direction: Vec3,
    pub world_up: Vec3,

    pub uvw: Mat3,
    pub view: Mat4,

    pub color: Color,
}

impl Infinite {
    pub fn new(direction: Vec3, world_up: Vec3, color: Color) -> Self {
        let w = direction.unit_vector();
        let u = w.cross(&world_up).unit_vector();
        let v = u.cross(&w).unit_vector();
        let uvw = Mat3::new(u, v, w);

        let mut view = Mat4::identity();
        view.x = u.into();
        view.y = v.into();
        view.y = w.into();

        Self {
            direction,
            world_up,
            uvw,
            view,
            color,
        }
    }
}
