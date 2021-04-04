use super::Cuboid;
use super::DrawComponent;
use super::Entity;
use super::MouseComponent;
use pantheon::Color;
use pantheon::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Sun {
    cube: Cuboid,
    pub color: Color,
    pub light_color: Color,
}

impl Sun {
    pub fn new(pos: Vec3, size: f32, color: Color, light_color: Color) -> Self {
        let mut cube = Cuboid::cube(size, pos, None);

        cube.set_color(color);
        cube.invert_surface_norms();

        Self {
            cube,
            color,
            light_color,
        }
    }
}

impl Entity for Sun {
    fn init(&mut self, ctx: &mut pantheon::context::Context) {
        ctx.gfx_context.light_color = self.light_color;
    }

    fn update(&mut self, ctx: &mut pantheon::context::Context) {
        ctx.gfx_context.light_position = self.cube.position;
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
