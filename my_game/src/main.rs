use my_engine::context::Context;
use my_engine::event::EventHandler;
use my_engine::input::keyboard;
use my_engine::input::mouse;
use my_engine::winit::MouseButton;

use my_engine::math::Mat4;
use my_engine::math::Vec3;

mod camera;

struct State {
    frame: u32,
    points: Vec<(Vec3, Vec3)>,
    plane: Vec<(Vec3, Vec3)>,
    camera: camera::my_camera::Camera,
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
            // println!(
            //     "{:#?}",
            //     ctx.gfx_context.projection_transform * ctx.gfx_context.view_transform
            // );
        }

        ctx.gfx_context.model_transform = Mat4::identity();
        command = ctx.draw(command, &self.plane);
        command = ctx.draw(command, &self.points);

        //println!("{:#?}", self.camera.position);
        ctx.gfx_context.model_transform = Mat4::translation(1.5, 1.5, 5.0)
            * Mat4::rotation_from_degrees(self.theta, (0, 1, 0).into());
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
        //self.camera.update();
        //if self.mouse_down {
        //    self.camera.update_pitch_and_angle(ctx);
        //}
        for key in keyboard::pressed_keys(ctx).iter() {
            self.camera.process_keypress(*key);
        }

        if self.mouse_down {
            let delta = mouse::delta(ctx);
            self.camera
                .process_mouse_move((delta.x * 1.0, delta.y * 1.0).into());
        }

        ctx.gfx_context.view_transform = self.camera.view_matrix();
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f64, y: f64) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));

        self.mouse_down = true;
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f64, y: f64) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));
        //self.camera.update_pitch_and_angle(ctx);

        self.mouse_down = false;
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f64, y: f64) {
        println!("Mouse wheel scrolled: x: {}, y: {}", x, y);
        //self.camera.update_zoom(Vec2::new(x, y));
    }

    /*
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: VirtualKeyCode,
        _keymods: ModifiersState,
        _repeat: bool,
    ) {
        self.camera.process_keypress(keycode);
        ctx.gfx_context.view_transform = self.camera.view_matrix();
    }
    */

    fn resize_event(&mut self, ctx: &mut Context, width: f64, height: f64) {
        // prevent degenerate case where things go wrong if resize while moving camera orientation
        self.mouse_down = false;

        println!("resize_event: width: {}, height: {}", width, height);
        self.camera = camera::my_camera::Camera::new(
            self.camera.origin,
            self.camera.w,
            self.camera.world_up,
            70.0,
            width,
            height,
            self.camera.near_plane,
            self.camera.far_plane,
        );
        ctx.gfx_context.view_transform = self.camera.view_matrix();
        ctx.gfx_context.projection_transform = self.camera.projection_matrix();
    }
}

#[allow(dead_code)]
fn get_corner_positions(row: f32, col: f32) -> [(Vec3, Vec3); 4] {
    [
        ((col, -5.0f32, row).into(), Vec3::new(1, 0, 0)),
        ((col, -5.0f32, row + 1.0).into(), Vec3::new(0, 1, 0)),
        ((col + 1.0, -5.0f32, row).into(), Vec3::new(1, 0, 1)),
        ((col + 1.0, -5.0f32, row + 1.0).into(), Vec3::new(0, 1, 1)),
    ]
}

#[allow(dead_code)]
fn generate_grid(size: i32) -> Vec<(Vec3, Vec3)> {
    let mut grid: Vec<(Vec3, Vec3)> = vec![];

    for row in -size..size {
        for col in -size..size {
            let pos = get_corner_positions(row as f32, col as f32);
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
        plane: generate_grid(50),
        points: generate_cube(),
        camera: camera::my_camera::Camera::new(
            Vec3::new_from_one(1),
            Vec3::new_from_one(0),
            (0, -10, 0).into(),
            70.0,
            100.0,
            100.0,
            -10.0,
            10.0,
        ),
        mouse_down: false,
        theta: 0.0,
    };

    let _ = my_engine::event::run(event_loop, ctx, my_game);
}
