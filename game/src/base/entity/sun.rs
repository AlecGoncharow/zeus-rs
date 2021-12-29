use super::Cuboid;
use super::DrawComponent;
use super::Entity;
use super::MouseComponent;
use crate::base::vertex::*;
use pantheon::Vec3;
use pantheon::{Color, Mat4, PolygonMode, Topology};

const _SUNRISE: Color = Color::floats(1., 0.7922, 0.4863);
const _NOON: Color = Color::floats(0.529, 0.81, 0.922);
const _SUNSET: Color = Color::floats(0.98, 0.8392, 0.647);
const _MIDNIGHT: Color = Color::floats(0.1254, 0.1098, 0.1804);

#[derive(Debug, Copy, Clone)]
pub struct Sun<'a> {
    pub cube: Cuboid<'a>,
    pub radians: f32,
    pub color: Color,
    pub light_color: Color,
    pub size: f32,
    pub rotating: bool,
    pub rotation_axis: Vec3,
    pub proj: Mat4,
}

impl<'a> Sun<'a> {
    pub fn new(pos: Vec3, size: f32, color: Color, light_color: Color) -> Self {
        let mut cube = Cuboid::cube(
            size,
            pos,
            None,
            VertexKind::Basic,
            Some(Topology::TriangleList(PolygonMode::Fill)),
        );

        cube.set_color(color);
        cube.invert_surface_norms();

        Self {
            cube,
            color,
            radians: 90.0f32.to_radians(),
            size,
            light_color,
            rotating: false,
            rotation_axis: (0, 1, 0).into(),
            proj: Mat4::pyramidal(90., 1.0, 1.0, 500.0),
        }
    }
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.cube.position, (0, 0, 0).into(), (0, 1, 0).into())
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::pyramidal(90., 1.0, 1.0, 500.0)
    }
}

impl<'a> Entity for Sun<'a> {
    fn init(&mut self, _ctx: &mut pantheon::context::Context) {
        // @TODO
        //ctx.gfx_context.light_uniforms.light_color = self.light_color;
    }

    fn update(&mut self, _ctx: &mut pantheon::context::Context) {
        /*
            if self.rotating {
                let delta_time = ctx.timer_context.delta_time();
                let rotate = delta_time * 0.01 * std::f32::consts::PI;

                /*
                self.radians += rotate;
                //self.radians %= (std::f32::consts::PI * 2.;

                let cos = self.radians.cos();
                let sin = self.radians.sin();
                ctx.gfx_context.clear_color = if sin > 0. {
                    if cos > 0. {
                        Color::interpolate(SUNRISE, NOON, sin)
                    } else {
                        Color::interpolate(SUNSET, NOON, sin)
                    }
                } else {
                    if cos < 0. {
                        Color::interpolate(SUNSET, MIDNIGHT, -sin)
                    } else {
                        Color::interpolate(SUNRISE, MIDNIGHT, -sin)
                    }
                }
                .into();
                */

                self.cube.position =
                    (Mat4::rotation(rotate, self.rotation_axis) * self.cube.position.vec4()).vec3();
            }

            /* @TODO
            ctx.gfx_context.light_uniforms.light_position = self.cube.position;
            ctx.gfx_context.light_uniforms.light_view_project = self.proj * self.view_matrix();
            */
        */
    }
}

impl<'a> DrawComponent<'a> for Sun<'a> {
    fn draw(&mut self, ctx: &mut pantheon::context::Context<'a>) {
        self.cube.draw(ctx);
    }

    fn debug_draw(&mut self, ctx: &mut pantheon::context::Context<'a>) {
        self.cube.debug_draw(ctx);
    }
}

impl<'a> MouseComponent for Sun<'a> {
    fn click_start(&mut self, ctx: &mut pantheon::context::Context) {
        self.cube.click_start(ctx);
    }

    fn click_end(&mut self, ctx: &mut pantheon::context::Context) {
        self.cube.click_end(ctx);
    }

    fn mouse_over(
        &mut self,
        ctx: &mut pantheon::context::Context,
        pos: pantheon::Vec3,
        cam: &super::Camera,
    ) {
        self.cube.mouse_over(ctx, pos, cam);
    }

    fn check_collision(
        &mut self,
        ctx: &mut pantheon::context::Context,
        camera_origin: pantheon::Vec3,
        mouse_direction: pantheon::Vec3,
    ) -> Option<super::component::MousePick> {
        self.cube
            .check_collision(ctx, camera_origin, mouse_direction)
    }
}
