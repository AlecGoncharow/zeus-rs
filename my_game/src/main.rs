use my_engine::context::Context;
use my_engine::event::EventHandler;
use my_engine::winit::MouseButton;

use my_engine::math::Vec3;

mod camera;

struct State {
    frame: u32,
    points: Vec<Vec3>,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), ()> {
        self.frame += 1;

        ctx.gfx_context.set_verts(&self.points);
        ctx.render();
        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context) -> Result<(), ()> {
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        self.points.push(Vec3::new(x, y, 0.0));
    }
}

fn main() {
    let (ctx, event_loop) = Context::new();
    let my_game = State {
        frame: 0,
        points: vec![],
    };

    let _ = my_engine::event::run(event_loop, ctx, my_game);
}
