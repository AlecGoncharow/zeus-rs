use my_engine::context::Context;
use my_engine::event::EventHandler;
use my_engine::winit::MouseButton;

use my_engine::math::Vec2;
use my_engine::math::Vec3;

mod camera;

struct State {
    frame: u32,
    points: Vec<Vec3>,
    camera: camera::Camera,
    mouse_down: bool,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), ()> {
        self.frame += 1;
        if self.frame < 10 {
            println!(
                "{:#?}",
                self.camera.projection_matrix * self.camera.view_matrix
            );
        }
        println!("{:#?}", self.camera.distance_float);
        ctx.gfx_context.set_verts(&self.points);
        ctx.render(&self.camera);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), ()> {
        self.camera.update();
        if self.mouse_down {
            self.camera.update_pitch_and_angle(ctx);
        }
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f64, y: f64) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));
        self.camera.update_pitch_and_angle(ctx);

        self.mouse_down = true;
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f64, y: f64) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));
        self.camera.update_pitch_and_angle(ctx);

        self.mouse_down = false;
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f64, y: f64) {
        println!("Mouse wheel scrolled: x: {}, y: {}", x, y);
        self.camera.update_zoom(Vec2::new(x, y));
    }
}

fn main() {
    let (ctx, event_loop) = Context::new();
    let my_game = State {
        frame: 0,
        points: vec![
            (0.0, 0.0, 0.0).into(),
            (-0.5, -0.7, 0.5).into(),
            (0.5, -0.7, 0.5).into(),
            (-0.5, -0.7, 0.5).into(),
            (0.5, -0.7, 0.5).into(),
            (0.0, 0.7, 0.0).into(),
        ],
        camera: camera::Camera::new(&ctx),
        mouse_down: false,
    };

    let _ = my_engine::event::run(event_loop, ctx, my_game);
}
