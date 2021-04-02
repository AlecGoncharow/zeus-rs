use super::component::*;
use super::plane::Plane;
use super::triangle::Triangle;
use super::Entity;
use crate::camera::Camera;
use pantheon::context::Context;
use pantheon::graphics::color::Color;
use pantheon::graphics::mode::DrawMode;
use pantheon::graphics::Drawable;
use pantheon::graphics::PolygonMode;
use pantheon::graphics::Topology;
use pantheon::input::mouse;
use pantheon::math::*;
use pantheon::winit::event::MouseButton;

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

pub fn get_cube_verts(size: f32) -> [Vec3; 8] {
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

pub fn cube_normals() -> [Vec3; 8] {
    [
        (0, 0, 1).into(),  // front
        (1, 0, 0).into(),  // right
        (0, 1, 0).into(),  // bottom
        (-1, 0, 0).into(), // left
        (0, -1, 0).into(), // top
        (0, 0, -1).into(), // back
        (0, 0, 0).into(),
        (0, 0, 0).into(),
    ]
}

#[rustfmt::skip]
fn cube_indices() -> [u16; 36] {
    [
        // front
        0, 1, 2,
        0, 3, 2,
        // right
        1, 2, 6,
        1, 5, 6,
        // bottom
        2, 3, 7,
        2, 6, 7,
        // left
        3, 0, 4,
        3, 7, 4,
        // top
        4, 0, 1,
        4, 5, 1,
        // back
        5, 4, 7,
        5, 6, 7,
    ]
}

pub fn cuboid_default_colors() -> [Color; 8] {
    // from https://en.wikibooks.org/wiki/OpenGL_Programming/Modern_OpenGL_Tutorial_05#Adding_the_3rd_dimension
    [
        // front colors
        (1.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        (0.0, 0.0, 1.0).into(),
        (1.0, 1.0, 1.0).into(),
        // back colors
        (1.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        (0.0, 0.0, 1.0).into(),
        (1.0, 1.0, 1.0).into(),
    ]
}

#[derive(Debug, Copy, Clone)]
pub struct Cuboid {
    vertices: [(Vec3, Color, Vec3); 8],
    faces: [(Triangle, Triangle); 6],
    indices: [u16; 36],
    pub draw_mode: PolygonMode,
    pub position: Vec3,
    pub rotation: Mat4,
    pub moused_over: bool,
}

impl Cuboid {
    pub fn cube(size: f32, position: Vec3, draw_mode: Option<PolygonMode>) -> Cuboid {
        // TODO normals
        let pos = get_cube_verts(size);
        //let colors = cuboid_default_colors();
        let colors = [Color::new(60, 60, 60); 8];
        let normals = cube_normals();

        let mut vertices = [(
            Vec3::new_from_one(1),
            Color::new(0, 0, 0),
            Vec3::new_from_one(1),
        ); 8];

        for i in 0..8 {
            vertices[i] = (pos[i], colors[i], normals[i]);
        }

        let faces = [
            (
                Triangle::new(pos[0], pos[1], pos[2]),
                Triangle::new(pos[2], pos[3], pos[0]),
            ),
            (
                Triangle::new(pos[1], pos[5], pos[6]),
                Triangle::new(pos[6], pos[2], pos[1]),
            ),
            (
                Triangle::new(pos[7], pos[6], pos[5]),
                Triangle::new(pos[5], pos[4], pos[7]),
            ),
            (
                Triangle::new(pos[4], pos[0], pos[3]),
                Triangle::new(pos[3], pos[7], pos[4]),
            ),
            (
                Triangle::new(pos[4], pos[5], pos[1]),
                Triangle::new(pos[1], pos[0], pos[4]),
            ),
            (
                Triangle::new(pos[3], pos[2], pos[6]),
                Triangle::new(pos[6], pos[7], pos[3]),
            ),
        ];
        Cuboid {
            vertices,
            faces,
            indices: cube_indices(),
            draw_mode: draw_mode.unwrap_or(PolygonMode::Fill),
            position,
            rotation: Mat4::identity(),
            moused_over: false,
        }
    }

    pub fn set_color(&mut self, new_color: Color) {
        for (_, color, _) in self.vertices.iter_mut() {
            *color = new_color;
        }
    }

    pub fn invert_surface_norms(&mut self) {
        for (_, _, norm) in self.vertices.iter_mut() {
            *norm *= -1.;
        }
    }
}

impl Entity for Cuboid {
    fn update(&mut self, ctx: &mut Context) {
        self.moused_over = false;
        let delta_time = ctx.timer_context.delta_time();
        self.rotate(delta_time * 0.2 * std::f32::consts::PI, (1, 1, 1).into());
    }
}

impl DrawComponent for Cuboid {
    fn draw(&mut self, ctx: &mut Context) {
        /*
        let mut color: Color = (0, 0, 0).into();
        if self.moused_over {
            color = (255, 255, 255).into();
        }

        let mut face_lines = vec![];
        for (tri_one, tri_two) in self.faces.iter() {
            face_lines.push((tri_one.p0, color));
            face_lines.push((tri_one.p1, color));
            face_lines.push((tri_one.p2, color));
            face_lines.push((tri_two.p0, color));
            face_lines.push((tri_two.p1, color));
            face_lines.push((tri_two.p2, color));
        }
        */

        ctx.gfx_context.model_transform = self.model_matrix();
        //ctx.draw(Topology::TriangleList(PolygonMode::Line), &face_lines);

        ctx.draw_indexed(self.draw_mode(), &self.vertices, self.indices().unwrap());
    }

    fn debug_draw(&mut self, ctx: &mut Context) {
        let mut lines = vec![];
        let color = Color::new(0, 0, 0);
        let end_color = Color::new(255, 0, 255);

        for (vert, _, norm) in self.vertices.iter().copied() {
            lines.push((vert, color));
            lines.push((vert + (3. * norm), end_color));
        }

        ctx.gfx_context.model_transform = Mat4::translation::<f32>(self.position.into());

        ctx.draw(
            DrawMode::Normal(Topology::LineList(PolygonMode::Fill)),
            &lines,
        );
    }
}

// impl AsComponent for Cuboid {}

impl MouseComponent for Cuboid {
    fn click_start(&mut self, _ctx: &mut Context) {}
    fn click_end(&mut self, _ctx: &mut Context) {}

    fn mouse_over(&mut self, ctx: &mut Context, pos: Vec3, camera: &Camera) {
        self.moused_over = true;

        if mouse::button_pressed(ctx, MouseButton::Left) {
            //@TODO this is impossible in 3D space dont @ me
            // fix to be in relation to a ground plane
            let delta = mouse::delta(ctx);
            println!("delta: {:#?}", delta);

            let ndc_x = delta.x / ctx.gfx_context.window_dims.width;
            let ndc_y = delta.y / ctx.gfx_context.window_dims.height;

            let delta_x = 2.0 * (pos - camera.origin).magnitude() * ndc_x * camera.u;
            let delta_y = 2.0 * (pos - camera.origin).magnitude() * ndc_y * camera.v;
            let trans = delta_x + delta_y;

            self.translate(trans.into());
        }
    }

    fn check_collision(
        &mut self,
        _ctx: &mut Context,
        camera_origin: Vec3,
        mouse_direction: Vec3,
    ) -> Option<MousePick> {
        let mut to_return: Option<Vec3> = None;
        let mut final_t = 0.0;
        let model = self.model_matrix();
        for (tri_one, tri_two) in self.faces.iter_mut() {
            let p0 = model * Vec4::from_vec3(tri_one.p0);
            let p1 = model * Vec4::from_vec3(tri_one.p1);
            let p2 = model * Vec4::from_vec3(tri_one.p2);
            let p0 = p0.truncate(Dim::W);
            let p1 = p1.truncate(Dim::W);
            let p2 = p2.truncate(Dim::W);
            let plane = Plane::new(p0, p1, p2).unwrap();
            // http://antongerdelan.net/opengl/raycasting.html
            // solve for t = -(O dot norm) + (d) / (norm dot direction)
            // plane:  point dot norm + d = 0 -> point dot norm = -d
            let t = -((camera_origin.dot(&plane.norm)) - (plane.norm.dot(&plane.point)))
                / (mouse_direction.dot(&plane.norm));

            if t > 0.0 {
                // On plane, check for triangle bounds
                let point = (t * mouse_direction) + camera_origin;

                // BARYCENTRIC TEST
                // ref http://blackpawn.com/texts/pointinpoly/default.html
                // and https://en.wikipedia.org/wiki/Barycentric_coordinate_system
                let v0 = p2 - p0;
                let v1 = p1 - p0;
                let v2 = point - p0;

                let dot00 = v0.dot(&v0);
                let dot01 = v0.dot(&v1);
                let dot02 = v0.dot(&v2);
                let dot11 = v1.dot(&v1);
                let dot12 = v1.dot(&v2);

                let inv_denom = 1.0 / ((dot00 * dot11) - (dot01.powi(2)));

                let u = ((dot11 * dot02) - (dot01 * dot12)) * inv_denom;
                let v = ((dot00 * dot12) - (dot01 * dot02)) * inv_denom;

                if u < 0.0 || v < 0.0 || (u + v > 1.0) {
                    // not in first
                    let p0 = model * Vec4::from_vec3(tri_two.p0);
                    let p1 = model * Vec4::from_vec3(tri_two.p1);
                    let p2 = model * Vec4::from_vec3(tri_two.p2);
                    let p0 = p0.truncate(Dim::W);
                    let p1 = p1.truncate(Dim::W);
                    let p2 = p2.truncate(Dim::W);

                    let v0 = p2 - p0;
                    let v1 = p1 - p0;
                    let v2 = point - p0;

                    let dot00 = v0.dot(&v0);
                    let dot01 = v0.dot(&v1);
                    let dot02 = v0.dot(&v2);
                    let dot11 = v1.dot(&v1);
                    let dot12 = v1.dot(&v2);

                    let inv_denom = 1.0 / ((dot00 * dot11) - (dot01.powi(2)));

                    let u = ((dot11 * dot02) - (dot01 * dot12)) * inv_denom;
                    let v = ((dot00 * dot12) - (dot01 * dot02)) * inv_denom;

                    if u < 0.0 || v < 0.0 || (u + v > 1.0) {
                        // both failed, continu
                        continue;
                    }
                }

                if let Some(other) = to_return {
                    if (point - camera_origin).magnitude() < (other - camera_origin).magnitude() {
                        to_return = Some(point);
                        final_t = t;
                    }
                } else {
                    to_return = Some(point);
                    final_t = t;
                }
            }
        }

        to_return.map(move |point| MousePick::new(self, point, final_t))
    }
}

impl Drawable for Cuboid {
    fn model_matrix(&self) -> Mat4 {
        Mat4::translation::<f32>(self.position.into()) * self.rotation
    }

    /// vertex buffer values (Position, Color)
    fn vertices(&self) -> &[(Vec3, Color, Vec3)] {
        &self.vertices
    }

    /// index buffer values
    fn indices(&self) -> Option<&[u16]> {
        Some(&self.indices)
    }

    fn draw_mode(&self) -> DrawMode {
        DrawMode::Shaded(Topology::TriangleList(self.draw_mode))
    }

    fn rotate(&mut self, theta: f32, axis: Vec3) {
        let rot = Mat4::rotation(theta, axis);
        self.rotation = rot * self.rotation;

        for (_, _, norm) in self.vertices.iter_mut() {
            *norm = (rot * Vec4::from_vec3(*norm)).truncate(Dim::W);
        }
    }

    fn translate(&mut self, tuple: (f32, f32, f32)) {
        self.position += tuple.into();
    }
}
