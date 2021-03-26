use pantheon::context::Context;
use pantheon::event::EventHandler;

use pantheon::graphics::PolygonMode;
use pantheon::graphics::Topology;

use pantheon::input::keyboard;
use pantheon::input::mouse;

use pantheon::winit::event::ModifiersState;
use pantheon::winit::event::MouseButton;
use pantheon::winit::event::VirtualKeyCode;

use pantheon::math::*;

use core::camera::Camera;
mod entity_manager;
use entity_manager::EntityManager;

use core::entity::EntityKind;
use core::message::GameMessage;

use hermes::client::ClientInterface;
use hermes::message::Message;

use hermes::tokio;

struct State {
    frame: u32,
    entity_manager: EntityManager,
    plane: Vec<(Vec3, Vec3)>,
    mouse_down: bool,
    network_client: ClientInterface<GameMessage>,
    network_queue: Vec<(std::net::SocketAddr, Message<GameMessage>)>,
    fps: f32,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), ()> {
        ctx.start_drawing();
        self.frame += 1;

        let fill_mode = Topology::TriangleList(PolygonMode::Fill);
        ctx.gfx_context.model_transform = Mat4::identity();
        ctx.draw(fill_mode, &self.plane);

        self.entity_manager.draw(ctx);

        ctx.render();
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), ()> {
        //self.camera.update();
        //if self.mouse_down {
        //    self.camera.update_pitch_and_angle(ctx);
        //}
        self.network_client
            .drain_message_queue(&mut self.network_queue);

        for (_source, mut message) in self.network_queue.drain(..) {
            println!("[Networking] Got msg {}", message);
            match message.header.id {
                GameMessage::GetId => {
                    let id: usize = message.pull();
                    println!("[Networking] Got id {}", id);
                }
                GameMessage::SyncWorld => {
                    while !message.body.is_empty() {
                        let entity: EntityKind = message.pull();
                        //println!("[Networking] Got entity: {:#?}", entity);
                        self.entity_manager.push_entity(entity);
                    }
                }
                _ => {}
            }
        }

        let delta_time = ctx.timer_context.delta_time();
        for key in keyboard::pressed_keys(ctx).iter() {
            self.entity_manager
                .camera
                .process_keypress(*key, delta_time);
        }

        if self.mouse_down {
            let delta = mouse::delta(ctx);
            self.entity_manager
                .camera
                .process_mouse_move((delta.x, delta.y).into(), delta_time);
        }

        ctx.gfx_context.view_transform = self.entity_manager.camera.view_matrix();
        self.entity_manager.update(ctx);

        self.fps = 1.0 / ctx.timer_context.average_tick;
        if ctx.timer_context.frame_count % pantheon::timer::MAX_SAMPLES == 0 {
            println!("FPS: {:#?}", self.fps);
        }

        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: VirtualKeyCode) {
        self.entity_manager.camera.process_keyrelease(keycode);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));

        if let MouseButton::Right = button {
            self.mouse_down = true;
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        //self.points.push(Vec3::new(x, y, 0.0));
        //self.camera.update_pitch_and_angle(ctx);

        if let MouseButton::Right = button {
            self.mouse_down = false;
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
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

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
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

    fn key_mods_changed(&mut self, _ctx: &mut Context, _modifiers_state: ModifiersState) {}
}

#[allow(dead_code)]
fn get_corner_positions(row: f32, col: f32, y: f32) -> [(Vec3, Vec3); 4] {
    [
        ((col, y, row).into(), Vec3::new(0.7, 0.7, 0.7)),
        ((col, y, row + 1.0).into(), Vec3::new(0.8, 0.8, 0.8)),
        ((col + 1.0, y, row).into(), Vec3::new(0.9, 0.9, 0.9)),
        ((col + 1.0, y, row + 1.0).into(), Vec3::new(1, 1, 1)),
    ]
}

#[allow(dead_code)]
fn populate_grid(grid: &mut Vec<(Vec3, Vec3)>, size: i32, y: f32) {
    for row in -size..size {
        for col in -size..size {
            let pos = get_corner_positions(row as f32, col as f32, y);
            grid.push(pos[0]);
            grid.push(pos[1]);
            grid.push(pos[2]);
            grid.push(pos[2]);
            grid.push(pos[1]);
            grid.push(pos[3]);
        }
    }
}

#[tokio::main]
async fn main() {
    let mut network_client: ClientInterface<GameMessage> = ClientInterface::new();
    println!(
        "Connectinon status: {:?}",
        network_client.connect("127.0.0.1", 8080).await
    );
    let message: Message<GameMessage> = Message::new(GameMessage::GetId);
    network_client.send(message).await.unwrap();
    let message: Message<GameMessage> = Message::new(GameMessage::SyncWorld);
    network_client.send(message).await.unwrap();

    let (mut ctx, event_loop) = Context::new((0.529, 0.81, 0.922, 1.0).into());
    let mut grid: Vec<(Vec3, Vec3)> = vec![];
    populate_grid(&mut grid, 50, -5.);
    populate_grid(&mut grid, 50, 15.);
    println!(
        "{:#?}",
        (
            ctx.gfx_context.window_dims.width,
            ctx.gfx_context.window_dims.height,
        )
    );

    let my_game = State {
        frame: 0,
        entity_manager: EntityManager::new(Camera::new(
            Vec3::new_from_one(1),
            Vec3::new_from_one(0),
            (0, -1, 0).into(),
            90.0,
            (
                ctx.gfx_context.window_dims.width,
                ctx.gfx_context.window_dims.height,
            ),
            (1., 100.0),
        )),
        plane: grid,

        mouse_down: false,
        fps: 0.,
        network_client,
        network_queue: vec![],
    };

    ctx.gfx_context.view_transform = my_game.entity_manager.camera.view_matrix();
    ctx.gfx_context.projection_transform = my_game.entity_manager.camera.projection_matrix();

    let _ = pantheon::event::run(event_loop, ctx, my_game);
}
