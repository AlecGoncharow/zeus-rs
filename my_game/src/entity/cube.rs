use super::component::*;
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
    indices: [u16; 36],
    pub draw_mode: PolygonMode,
    pub position: Vec3,
    pub rotation: Mat4,
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

        Cuboid {
            vertices,
            indices: cube_indices(),
            draw_mode: draw_mode.unwrap_or(PolygonMode::Fill),
            position,
            rotation: Mat4::identity(),
        }
    }
}

impl Entity for Cuboid {
    fn update(&mut self, _ctx: &mut Context) {}
}

impl DrawComponent for Cuboid {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.gfx_context.model_transform = self.model_matrix();
        ctx.draw_indexed(&self.draw_mode(), self.vertices(), self.indices().unwrap());
    }
}

impl AsComponent for Cuboid {}
impl AsMouseable for Cuboid {}

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
