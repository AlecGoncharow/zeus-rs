use super::component::*;
use super::plane::Plane;
use super::triangle::Triangle;
use super::Camera;
use super::Entity;
use crate::rendering;
use crate::vertex::*;
use pantheon::context::Context;
use pantheon::graphics::prelude::*;
use pantheon::graphics::Drawable;
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
fn cube_indices() -> [u32; 36] {
    [
        // front
        0, 1, 2,
        0, 2, 3,
        // right
        1, 6, 2,
        1, 5, 6,
        // bottom
        2, 7, 3,
        2, 6, 7,
        // left
        3, 4, 0,
        3, 7, 4,
        // top
        4, 1, 0,
        4, 5, 1,
        // back
        5, 4, 7,
        5, 7, 6,
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
enum CubioidVertMode {
    Shaded([ShadedVertex; 8]),
    Basic([BasicVertex; 8]),
}

#[allow(dead_code)]
impl CubioidVertMode {
    pub fn is_vertex(&self) -> bool {
        match self {
            &CubioidVertMode::Basic(_) => true,
            _ => false,
        }
    }

    pub fn as_basic(&self) -> &[BasicVertex; 8] {
        match self {
            &CubioidVertMode::Basic(ref verts) => verts,
            _ => panic!("bad usage"),
        }
    }

    pub fn try_as_basic(&self) -> Option<&[BasicVertex; 8]> {
        match self {
            &CubioidVertMode::Basic(ref verts) => Some(verts),
            _ => None,
        }
    }

    pub fn is_shaded(&self) -> bool {
        match self {
            &CubioidVertMode::Shaded(_) => true,
            _ => false,
        }
    }

    pub fn as_shaded(&self) -> &[ShadedVertex; 8] {
        match self {
            &CubioidVertMode::Shaded(ref verts) => verts,
            _ => panic!("bad usage"),
        }
    }

    pub fn try_as_shaded(&self) -> Option<&[ShadedVertex; 8]> {
        match self {
            &CubioidVertMode::Shaded(ref verts) => Some(verts),
            _ => None,
        }
    }

    pub fn try_as_shaded_mut(&mut self) -> Option<&mut [ShadedVertex; 8]> {
        match self {
            &mut CubioidVertMode::Shaded(ref mut verts) => Some(verts),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Cuboid<'a> {
    vertices: CubioidVertMode,
    faces: [(Triangle, Triangle); 6],
    indices: [u32; 36],
    draw_call_handle: Option<DrawCallHandle<'a>>,
    pub topology: Topology,
    pub position: Vec3,
    pub rotation: Mat4,
    pub moused_over: bool,
}

impl<'a> Cuboid<'a> {
    pub fn cube(
        size: f32,
        position: Vec3,
        color: Option<Color>,
        vertex_kind: VertexKind,
        topology: Option<Topology>,
    ) -> Self {
        // TODO normals
        let pos = get_cube_verts(size);
        //let colors = cuboid_default_colors();
        let color = color.unwrap_or(Color::new(60, 60, 60));
        let colors = [color; 8];
        let normals = cube_normals();

        let vertices = match vertex_kind {
            VertexKind::Basic => {
                let mut vertices = [(Vec3::new_from_one(1), Color::new(0, 0, 0)).into(); 8];

                for i in 0..8 {
                    vertices[i] = (pos[i], colors[i]).into();
                }
                CubioidVertMode::Basic(vertices)
            }
            VertexKind::Shaded => {
                let mut vertices = [(
                    Vec3::new_from_one(1),
                    Color::new(0, 0, 0),
                    Vec3::new_from_one(1),
                )
                    .into(); 8];

                for i in 0..8 {
                    vertices[i] = (pos[i], colors[i], normals[i]).into();
                }

                CubioidVertMode::Shaded(vertices)
            }
            _ => unimplemented!(),
        };

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
            draw_call_handle: None,
            topology: topology.unwrap_or(Topology::TriangleList(PolygonMode::Fill)),
            position,
            rotation: Mat4::identity(),
            moused_over: false,
        }
    }

    pub fn set_color(&mut self, new_color: Color) {
        match self.vertices {
            CubioidVertMode::Basic(ref mut verts) => {
                for vert in verts.iter_mut() {
                    vert.color = new_color;
                }
            }
            CubioidVertMode::Shaded(ref mut verts) => {
                for vert in verts.iter_mut() {
                    vert.color = new_color;
                }
            }
        }
    }

    pub fn invert_surface_norms(&mut self) {
        if let Some(verts) = self.vertices.try_as_shaded_mut() {
            for vert in verts.iter_mut() {
                vert.normal *= -1.;
            }
        }
    }
}

impl<'a> Entity for Cuboid<'a> {
    fn update(&mut self, ctx: &mut Context) {
        self.moused_over = false;
        let delta_time = ctx.timer_context.delta_time();
        self.rotate(delta_time * 0.2 * std::f32::consts::PI, (1, 1, 1).into());
    }
}

impl<'a> DrawComponent<'a> for Cuboid<'a> {
    fn register(&mut self, ctx: &mut Context<'a>) {
        let push_constant = Some(PushConstant::vertex_data(0, &[self.model_matrix()]));

        self.draw_call_handle = Some(match self.vertices {
            CubioidVertMode::Basic(verts) => rendering::register_indexed(
                ctx,
                &["reflection", "refraction", "shaded"],
                "basic",
                self.topology,
                &verts,
                &self.indices,
                0..1,
                push_constant,
                None,
            ),
            CubioidVertMode::Shaded(verts) => rendering::register_indexed(
                ctx,
                &["reflection", "refraction", "shaded"],
                "shaded",
                self.topology,
                &verts,
                &self.indices,
                0..1,
                push_constant,
                None,
            ),
        });
    }

    fn draw(&mut self, ctx: &mut Context<'a>) {
        if let Some(draw_call_handle) = self.draw_call_handle {
            draw_call_handle.set_push_constant_data(ctx, &[self.model_matrix()]);
        }
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
        /*

        ctx.set_model(self.model_matrix());
        //ctx.draw(Topology::TriangleList(PolygonMode::Line), &face_lines);

        match self.vertices {
            CubioidVertMode::Vertex(verts) => {
                ctx.draw_indexed(self.draw_mode(), &verts, &self.indices);
            }
            CubioidVertMode::ShadedVertex(verts) => {
                ctx.draw_indexed(self.draw_mode(), &verts, &self.indices);
            }
        }
        */
    }

    fn debug_draw(&mut self, _ctx: &mut Context) {
        /*
        if let Some(verts) = self.vertices.try_as_shaded() {
            let mut lines: Vec<Vertex> = vec![];
            let color = Color::new(0, 0, 0);
            let end_color = Color::new(255, 0, 255);

            for vert in verts.iter() {
                lines.push((vert.position, color).into());
                lines.push((vert.position + (3. * vert.normal), end_color).into());
            }

            ctx.set_model(self.model_matrix());

            ctx.draw(
                DrawMode::Normal(Topology::LineList(PolygonMode::Fill)),
                &lines,
            );
        }
            */
    }
}

// impl AsComponent for Cuboid {}

impl<'a> MouseComponent for Cuboid<'a> {
    fn click_start(&mut self, _ctx: &mut Context) {}
    fn click_end(&mut self, _ctx: &mut Context) {}

    fn mouse_over(&mut self, ctx: &mut Context, pos: Vec3, camera: &Camera) {
        self.moused_over = true;

        if mouse::button_pressed(ctx, MouseButton::Left) {
            //@TODO this is impossible in 3D space dont @ me
            // fix to be in relation to a ground plane
            let delta = mouse::delta(ctx);
            //println!("delta: {:#?}", delta);

            let ndc_x = delta.x / ctx.gfx_context.window_dims.width;
            let ndc_y = -delta.y / ctx.gfx_context.window_dims.height;

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

impl<'a> Drawable for Cuboid<'a> {
    fn model_matrix(&self) -> Mat4 {
        Mat4::translation::<f32>(self.position.into()) * self.rotation
    }

    fn rotate(&mut self, theta: f32, axis: Vec3) {
        let rot = Mat4::rotation(theta, axis);
        self.rotation = rot * self.rotation;
    }

    fn translate(&mut self, tuple: (f32, f32, f32)) {
        self.position += tuple.into();
    }
}
