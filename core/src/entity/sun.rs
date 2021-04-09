use super::Cuboid;
use super::DrawComponent;
use super::Entity;
use super::MouseComponent;
use pantheon::Vec3;
use pantheon::{Color, DrawMode, Mat4, PolygonMode, Topology};

const SUNRISE: Color = Color::floats(1., 0.7922, 0.4863);
const NOON: Color = Color::floats(0.529, 0.81, 0.922);
const SUNSET: Color = Color::floats(0.98, 0.8392, 0.647);
const MIDNIGHT: Color = Color::floats(0.1254, 0.1098, 0.1804);

#[derive(Debug, Copy, Clone)]
pub struct Sun {
    pub cube: Cuboid,
    pub radians: f32,
    pub color: Color,
    pub light_color: Color,
    pub size: f32,
}

impl Sun {
    pub fn new(pos: Vec3, size: f32, color: Color, light_color: Color) -> Self {
        let mut cube = Cuboid::cube(
            size,
            pos,
            Some(DrawMode::Normal(Topology::TriangleList(PolygonMode::Fill))),
        );

        cube.set_color(color);
        cube.invert_surface_norms();

        Self {
            cube,
            color,
            radians: 90.0f32.to_radians(),
            size,
            light_color,
        }
    }
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.cube.position, (0, 0, 0).into(), (0, 1, 0).into())
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(90., 1.0, 1.0, 500.0)
    }
}

impl Entity for Sun {
    fn init(&mut self, ctx: &mut pantheon::context::Context) {
        ctx.gfx_context.light_uniforms.light_color = self.light_color;
    }

    fn update(&mut self, ctx: &mut pantheon::context::Context) {
        //let delta_time = ctx.timer_context.delta_time();
        //let rotate = delta_time * 0.2 * std::f32::consts::PI;
        //self.radians += rotate;
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

        /*
        self.cube.position =
            (Mat4::rotation(rotate, (0, 0, 1).into()) * self.cube.position.vec4()).vec3();

        */
        ctx.gfx_context.light_uniforms.light_position = self.cube.position;
        ctx.gfx_context.light_uniforms.light_view_project =
            self.projection_matrix() * self.view_matrix();
    }
}

impl DrawComponent for Sun {
    fn draw(&mut self, ctx: &mut pantheon::context::Context) {
        self.cube.draw(ctx);
    }

    fn debug_draw(&mut self, ctx: &mut pantheon::context::Context) {
        self.cube.debug_draw(ctx);
    }
}

impl MouseComponent for Sun {
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
        cam: &crate::camera::Camera,
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
