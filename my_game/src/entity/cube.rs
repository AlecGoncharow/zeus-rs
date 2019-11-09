use super::component::*;
use super::plane::Plane;
use super::triangle::Triangle;
use super::Entity;
use my_engine::context::Context;
use my_engine::graphics::Drawable;
use my_engine::graphics::PolygonMode;
use my_engine::graphics::Topology;
use my_engine::math::*;

pub fn _get_unit_cube_verts() -> [Vec3; 8] {
    [
        // front
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        // back
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
    ]
}

pub fn get_cube_verts(size: f64) -> [Vec3; 8] {
    [
        // front
        size * Vec3::new(-0.5, -0.5, 0.5),
        size * Vec3::new(0.5, -0.5, 0.5),
        size * Vec3::new(0.5, 0.5, 0.5),
        size * Vec3::new(-0.5, 0.5, 0.5),
        // back
        size * Vec3::new(-0.5, -0.5, -0.5),
        size * Vec3::new(0.5, -0.5, -0.5),
        size * Vec3::new(0.5, 0.5, -0.5),
        size * Vec3::new(-0.5, 0.5, -0.5),
    ]
}

#[rustfmt::skip]
fn cube_indices() -> [u16; 36] {
    [
        // front
        0, 1, 2,
        2, 3, 0,
        // right
        1, 5, 6,
        6, 2, 1,
        // back
        7, 6, 5,
        5, 4, 7,
        // left
        4, 0, 3,
        3, 7, 4,
        // bottom
        4, 5, 1,
        1, 0, 4,
        // top
        3, 2, 6,
        6, 7, 3
    ]
}

pub fn cuboid_default_colors() -> [Vec3; 8] {
    // from https://en.wikibooks.org/wiki/OpenGL_Programming/Modern_OpenGL_Tutorial_05#Adding_the_3rd_dimension
    [
        // front colors
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        // back colors
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
    ]
}

pub struct Cuboid {
    vertices: Vec<(Vec3, Vec3)>,
    planes: [(Plane, Triangle, Triangle); 6],
    indices: [u16; 36],
    pub draw_mode: PolygonMode,
    pub position: Vec3,
    pub rotation: Mat4,
    pub moused_over: bool,
}

impl Cuboid {
    pub fn cube(size: f64, position: Vec3, draw_mode: Option<PolygonMode>) -> Cuboid {
        // TODO normals
        let pos = get_cube_verts(size);
        let colors = cuboid_default_colors();

        let mut vertices = vec![];

        for i in 0..8 {
            vertices.push((pos[i], colors[i]));
        }

        let planes = [
            (
                Plane::new(pos[0], pos[1], pos[2]).unwrap(),
                Triangle::new(pos[0], pos[1], pos[2]),
                Triangle::new(pos[2], pos[3], pos[0]),
            ),
            (
                Plane::new(pos[1], pos[5], pos[6]).unwrap(),
                Triangle::new(pos[1], pos[5], pos[6]),
                Triangle::new(pos[6], pos[2], pos[1]),
            ),
            (
                Plane::new(pos[7], pos[6], pos[5]).unwrap(),
                Triangle::new(pos[7], pos[6], pos[5]),
                Triangle::new(pos[5], pos[4], pos[7]),
            ),
            (
                Plane::new(pos[4], pos[0], pos[3]).unwrap(),
                Triangle::new(pos[4], pos[0], pos[3]),
                Triangle::new(pos[3], pos[7], pos[4]),
            ),
            (
                Plane::new(pos[4], pos[5], pos[1]).unwrap(),
                Triangle::new(pos[4], pos[5], pos[1]),
                Triangle::new(pos[1], pos[0], pos[4]),
            ),
            (
                Plane::new(pos[3], pos[2], pos[6]).unwrap(),
                Triangle::new(pos[3], pos[2], pos[6]),
                Triangle::new(pos[6], pos[7], pos[3]),
            ),
        ];

        Cuboid {
            vertices,
            planes,
            indices: cube_indices(),
            draw_mode: draw_mode.unwrap_or(PolygonMode::Fill),
            position,
            rotation: Mat4::identity(),
            moused_over: false,
        }
    }
}

impl Entity for Cuboid {
    fn update(&mut self, _ctx: &mut Context) {
        self.moused_over = false;
    }
}

impl DrawComponent for Cuboid {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.gfx_context.model_transform = self.model_matrix();

        if self.moused_over {
            ctx.gfx_context.model_transform = self.model_matrix() * Mat4::scalar_from_one(1.1);
        }

        ctx.draw_indexed(&self.draw_mode(), self.vertices(), self.indices().unwrap());
    }
}

impl AsComponent for Cuboid {}

impl MouseComponent for Cuboid {
    fn click_start(&mut self, _ctx: &mut Context) {}
    fn click_end(&mut self, _ctx: &mut Context) {}

    fn mouse_over(&mut self, _pos: Vec3) {
        self.moused_over = true;
    }

    fn check_collision(
        &mut self,
        camera_origin: Vec3,
        mouse_direction: Vec3,
    ) -> Option<(&mut dyn MouseComponent, Vec3)> {
        let mut to_return: Option<Vec3> = None;
        let model = self.model_matrix();
        for (plane, tri_one, tri_two) in self.planes.iter_mut() {
            let plane_point = model * Vec4::from_vec3(plane.point);
            let t = -((camera_origin.dot(&plane.norm))
                + (plane.norm.dot(&plane_point.truncate(Dim::W))))
                / (mouse_direction.dot(&plane.norm));

            if t > 0.0 {
                // On plane, check for triangle bounds
                let point = t * mouse_direction + camera_origin;
                let p1 = model * Vec4::from_vec3(tri_one.p1);
                let p2 = model * Vec4::from_vec3(tri_one.p2);
                let p3 = model * Vec4::from_vec3(tri_one.p3);
                let mut in_bounds = true; // this gets flipped to false if fail for first triangle
                let p1 = p1.truncate(Dim::W);
                let p2 = p2.truncate(Dim::W);
                let p3 = p3.truncate(Dim::W);

                // edge 1
                let edge = p2 - p1;
                let vp = point - p1;
                let cross = edge.cross(&vp);
                if plane.norm.dot(&cross) < 0.0 {
                    in_bounds = false;
                }

                // edge 2
                let edge = p3 - p2;
                let vp = point - p2;
                let cross = edge.cross(&vp);
                if plane.norm.dot(&cross) < 0.0 {
                    in_bounds = false;
                }

                // edge 3
                let edge = p1 - p3;
                let vp = point - p3;
                let cross = edge.cross(&vp);
                if plane.norm.dot(&cross) < 0.0 {
                    in_bounds = false;
                }

                if !in_bounds {
                    // not in first
                    let p1 = model * Vec4::from_vec3(tri_two.p1);
                    let p2 = model * Vec4::from_vec3(tri_two.p2);
                    let p3 = model * Vec4::from_vec3(tri_two.p3);
                    let mut in_bounds = true;
                    let p1 = p1.truncate(Dim::W);
                    let p2 = p2.truncate(Dim::W);
                    let p3 = p3.truncate(Dim::W);

                    // edge 1
                    let edge = p2 - p1;
                    let vp = point - p1;
                    let cross = edge.cross(&vp);
                    if plane.norm.dot(&cross) < 0.0 {
                        in_bounds = false;
                    }

                    // edge 2
                    let edge = p3 - p2;
                    let vp = point - p2;
                    let cross = edge.cross(&vp);
                    if plane.norm.dot(&cross) < 0.0 {
                        in_bounds = false;
                    }

                    // edge 3
                    let edge = p1 - p3;
                    let vp = point - p3;
                    let cross = edge.cross(&vp);
                    if plane.norm.dot(&cross) < 0.0 {
                        in_bounds = false;
                    }

                    if !in_bounds {
                        // both failed, continue
                        continue;
                    }
                }

                if let Some(other) = to_return {
                    if (point - camera_origin).magnitude() < (other - camera_origin).magnitude() {
                        to_return = Some(point);
                    }
                } else {
                    to_return = Some(point);
                }
            }
        }

        if let Some(point) = to_return {
            Some((self, point))
        } else {
            None
        }
    }
}

impl AsMouseable for Cuboid {
    fn as_mouseable(&mut self) -> Option<&mut dyn MouseComponent> {
        Some(self)
    }
}

impl AsDrawable for Cuboid {
    fn as_drawable(&mut self) -> Option<&mut dyn DrawComponent> {
        Some(self)
    }
}

impl Drawable for Cuboid {
    fn model_matrix(&self) -> Mat4 {
        Mat4::translation::<f64>(self.position.into()) * self.rotation
    }

    /// vertex buffer values (Position, Color)
    fn vertices(&self) -> &Vec<(Vec3, Vec3)> {
        &self.vertices
    }

    /// index buffer values
    fn indices(&self) -> Option<&[u16]> {
        Some(&self.indices)
    }

    fn draw_mode(&self) -> Topology {
        Topology::TriangleList(self.draw_mode)
    }

    fn rotate(&mut self, theta: f64, axis: Vec3) {
        self.rotation = Mat4::rotation(theta, axis) * self.rotation;
    }

    fn translate(&mut self, tuple: (f64, f64, f64)) {
        self.position = self.position + tuple.into();
    }
}
