use my_engine::context::Context;
use my_engine::event::EventHandler;
use my_engine::winit::MouseButton;

use my_engine::math::Mat4;
use my_engine::math::Vec2;
use my_engine::math::Vec3;

mod camera;

struct State {
    frame: u32,
    points: Vec<(Vec3, Vec3)>,
    camera: camera::Camera,
    mouse_down: bool,
    theta: f64,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), ()> {
        // @TODO figure out a way around this command holding thing
        self.theta += 1.0;
        let mut command = ctx.start_drawing((0, 0, 0, 1).into());

        self.frame += 1;
        if self.frame < 10 {
            println!(
                "{:#?}",
                self.camera.projection_matrix * self.camera.view_matrix
            );
        }

        ctx.gfx_context.projection_transform = (
            (1.0, 0.0, 0.0, 0.0),
            (0.0, -1.0, 0.0, 0.0),
            (0.0, 0.0, 0.5, 0.5),
            (0.0, 0.0, 0.0, 1.0),
        )
            .into();

        //println!("{:#?}", self.camera.position);
        ctx.gfx_context.model_transform = Mat4::rotation_from_degrees(self.theta, (0, 1, 0).into());
        command = ctx.draw(command, &self.points);

        ctx.gfx_context.model_transform = Mat4::translation(-0.5, -0.5, 0.0)
            * Mat4::rotation_from_degrees(self.theta, (0, 1, 0).into())
            * Mat4::rotation_from_degrees(self.theta, (1, 0, 0).into())
            * Mat4::scalar_from_one(0.5);
        command = ctx.draw(command, &self.points);

        ctx.render(command);
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

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        println!("resize_event: width: {}, height: {}", width, height);
        self.camera.projection_matrix = camera::Camera::create_projection_matrix(ctx);
    }
}

fn _get_corner_positions(row: i32, col: i32) -> [Vec3; 4] {
    [
        (col, -5, row).into(),
        (col, -5, row + 1).into(),
        (col + 1, -5, row).into(),
        (col + 1, -5, row + 1).into(),
    ]
}

fn _generate_grid(size: i32) -> Vec<Vec3> {
    let mut grid: Vec<Vec3> = vec![];

    for row in -size..size {
        for col in -size..size {
            let pos = _get_corner_positions(row, col);
            grid.push(pos[0]);
            grid.push(pos[1]);
            grid.push(pos[2]);
            grid.push(pos[2]);
            grid.push(pos[1]);
            grid.push(pos[3]);
        }
    }

    grid
}

#[inline]
fn cube_verts() -> [(Vec3, Vec3); 8] {
    [
        (Vec3::new(-0.25, -0.25, 0.25), Vec3::new(0, 0, 0)),
        (Vec3::new(-0.25, 0.25, 0.25), Vec3::new(1, 0, 0)),
        (Vec3::new(0.25, 0.25, 0.25), Vec3::new(1, 1, 0)),
        (Vec3::new(0.25, -0.25, 0.25), Vec3::new(0, 1, 0)),
        (Vec3::new(-0.25, -0.25, -0.25), Vec3::new(0, 0, 1)),
        (Vec3::new(-0.25, 0.25, -0.25), Vec3::new(1, 0, 1)),
        (Vec3::new(0.25, 0.25, -0.25), Vec3::new(1, 1, 1)),
        (Vec3::new(0.25, -0.25, -0.25), Vec3::new(0, 1, 1)),
    ]
}

fn generate_cube() -> Vec<(Vec3, Vec3)> {
    let mut verts = vec![];
    push_quad(&mut verts, [1, 0, 3, 2]);
    push_quad(&mut verts, [2, 3, 7, 6]);
    push_quad(&mut verts, [3, 0, 4, 7]);
    push_quad(&mut verts, [6, 5, 1, 2]);
    push_quad(&mut verts, [4, 5, 6, 7]);
    push_quad(&mut verts, [5, 4, 0, 1]);

    verts
}

fn push_quad(verts: &mut Vec<(Vec3, Vec3)>, indices: [usize; 4]) {
    let cube_verts = cube_verts();

    let indices = [
        indices[0], indices[1], indices[2], indices[0], indices[2], indices[3],
    ];

    for index in indices.iter() {
        verts.push(cube_verts[*index]);
    }
}

fn main() {
    let (ctx, event_loop) = Context::new();
    let my_game = State {
        frame: 0,
        points: generate_cube(),
        camera: camera::Camera::new(&ctx),
        mouse_down: false,
        theta: 0.0,
    };

    println!("{:#?}", my_game.points);

    let _ = my_engine::event::run(event_loop, ctx, my_game);
}
