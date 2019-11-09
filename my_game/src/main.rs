use my_engine::context::Context;
use my_engine::event::EventHandler;

use my_engine::graphics::PolygonMode;
use my_engine::graphics::Topology;

use my_engine::input::keyboard;
use my_engine::input::mouse;

use my_engine::winit::ModifiersState;
use my_engine::winit::MouseButton;
use my_engine::winit::VirtualKeyCode;

use my_engine::math::*;

mod camera;
use camera::my_camera::Camera;
mod entity;

use entity::cube::Cuboid;
use entity::EntityManager;

struct State {
    frame: u32,
    entity_manager: EntityManager,
    plane: Vec<(Vec3, Vec3)>,
    mouse_down: bool,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), ()> {
        ctx.start_drawing((0, 0, 0, 1).into());
        self.frame += 1;

        let fill_mode = Topology::TriangleList(PolygonMode::Fill);
        ctx.gfx_context.model_transform = Mat4::identity();
        ctx.draw(&fill_mode, &self.plane);

        self.entity_manager.draw(ctx);

        ctx.render();
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), ()> {
        //self.camera.update();
        //if self.mouse_down {
        //    self.camera.update_pitch_and_angle(ctx);
        //}
        for key in keyboard::pressed_keys(ctx).iter() {
            self.entity_manager.camera.process_keypress(*key);
        }

        if self.mouse_down {
            let delta = mouse::delta(ctx);
            self.entity_manager
                .camera
                .process_mouse_move((delta.x * 1.0, delta.y * 1.0).into());
        }

        ctx.gfx_context.view_transform = self.entity_manager.camera.view_matrix();
        //ctx.gfx_context.view_transform = Mat4::identity();
        //ctx.gfx_context.projection_transform = Mat4::identity();
        self.entity_manager.update(ctx);
        Ok(())
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: VirtualKeyCode,
        _keymods: ModifiersState,
    ) {
        self.entity_manager.camera.process_keyrelease(keycode);
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
        self.entity_manager.camera.set_aspect((width, height));
        println!("new camera: {:#?}", self.entity_manager.camera);
        println!(
            "view_matrix: {:#?}",
            self.entity_manager.camera.view_matrix()
        );
        ctx.gfx_context.view_transform = self.entity_manager.camera.view_matrix();
        ctx.gfx_context.projection_transform = self.entity_manager.camera.projection_matrix();
    }
}

#[allow(dead_code)]
fn get_corner_positions(row: f32, col: f32) -> [(Vec3, Vec3); 4] {
    [
        ((col, -5.0f32, row).into(), Vec3::new(0.7, 0.7, 0.7)),
        ((col, -5.0f32, row + 1.0).into(), Vec3::new(0.8, 0.8, 0.8)),
        ((col + 1.0, -5.0f32, row).into(), Vec3::new(0.9, 0.9, 0.9)),
        ((col + 1.0, -5.0f32, row + 1.0).into(), Vec3::new(1, 1, 1)),
    ]

    /*
    [
        ((col, -5.0f32, row).into(), Vec3::new(1, 0, 0)),
        ((col, -5.0f32, row + 1.0).into(), Vec3::new(0, 1, 0)),
        ((col + 1.0, -5.0f32, row).into(), Vec3::new(1, 0, 1)),
        ((col + 1.0, -5.0f32, row + 1.0).into(), Vec3::new(0, 1, 1)),
    ]
        */
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

fn generate_cubes(state: &mut State) {
    let cube = Cuboid::cube(1.0, (0, 0, 0).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (10, 0, 10).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (0, 0, 10).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (10, 0, 0).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (10, 10, 10).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (0, 10, 10).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (10, 10, 0).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(1.0, (0, 10, 0).into(), None);
    state.entity_manager.push_entity(cube);

    let cube = Cuboid::cube(5.0, (5, 5, 5).into(), None);
    state.entity_manager.push_entity(cube);

    //let cube = Cuboid::cube(100.0, (0, -105, 0).into(), None);
    //state.entity_manager.push_entity(cube);
}

fn main() {
    let (ctx, event_loop) = Context::new();
    let mut my_game = State {
        frame: 0,
        entity_manager: EntityManager::new(Camera::new(
            Vec3::new_from_one(1),
            Vec3::new_from_one(0),
            (0, -1, 0).into(),
            90.0,
            100.0,
            100.0,
            0.5,
            100.0,
        )),
        plane: generate_grid(50),

        mouse_down: false,
    };

    generate_cubes(&mut my_game);

    let _ = my_engine::event::run(event_loop, ctx, my_game);
}
