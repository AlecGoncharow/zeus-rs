use pantheon::context::Context;
use pantheon::event::EventHandler;

use pantheon::graphics::PolygonMode;

use pantheon::input::keyboard;
use pantheon::input::mouse;

use pantheon::anyhow::*;
use pantheon::winit::event::ModifiersState;
use pantheon::winit::event::MouseButton;
use pantheon::winit::event::VirtualKeyCode;

use pantheon::math::*;

use pantheon::graphics::color::Color;

use core::camera::Camera;

use entity_manager::EntityManager;

use core::entity::Entity;
use core::entity::EntityKind;
use core::message::GameMessage;
use core::proc_gen;
use proc_gen::color::ColorGenerator;
use proc_gen::noise::Perlin;
use proc_gen::terrain::TerrainGenerator;

use hermes::client::ClientInterface;
use hermes::message::Message;

use hermes::tokio;

mod entity_manager;
mod ui;

struct State {
    frame: u32,
    entity_manager: EntityManager,
    mouse_down: bool,
    network_client: ClientInterface<GameMessage>,
    network_queue: Vec<(std::net::SocketAddr, Message<GameMessage>)>,
    fps: f32,
    debug: bool,
    sun_mesh: Option<core::entity::sun::Sun>,
    //texture: Texture,
    #[allow(unused)]
    top_right_ui: ui::TexturableQuad,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.start_drawing();
        self.frame += 1;

        self.entity_manager.draw(ctx);

        if self.debug {
            self.entity_manager.debug_draw(ctx);
        }

        //println!("{:#?}", shadow_quad);

        /* Shadow texture debug
        ctx.draw_textured(
            DrawMode::Textured(Topology::TriangleList(PolygonMode::Fill)),
            &self.top_right_ui.verts,
            //TextureKind::Custom(&self.texture),
            TextureKind::Shadow,
            //TextureKind::Depth,
        );
        */
        ctx.render();
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<()> {
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
                    let id: usize = message.pull().unwrap();
                    println!("[Networking] Got id {}", id);
                }
                GameMessage::SyncWorld => {
                    while !message.body.is_empty() {
                        let entity: EntityKind = message.pull().unwrap();
                        //println!("[Networking] Got entity: {:#?}", entity);
                        match entity {
                            EntityKind::Sun(s) => {
                                self.entity_manager.sun = s;
                                self.entity_manager.sun.init(ctx);
                            }
                            _ => self.entity_manager.push_entity(entity),
                        }
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

        if self.sun_mesh.is_some() {
            let mesh = self.entity_manager.get_sun_mesh();
            ctx.set_view(mesh.view_matrix());
            ctx.set_projection(mesh.projection_matrix());
        } else {
            ctx.set_view(self.entity_manager.camera.update_view_matrix());
            ctx.set_projection(self.entity_manager.camera.projection);
        }
        self.entity_manager.update(ctx);

        self.fps = 1.0 / ctx.timer_context.average_tick;
        if ctx.timer_context.frame_count % (pantheon::timer::MAX_SAMPLES * 10) == 0 {
            println!("FPS: {:#?}", self.fps)
        }

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: VirtualKeyCode, _repeat: bool) {
        if keycode == VirtualKeyCode::Escape {
            pantheon::event::quit(ctx);
        }

        if keycode == VirtualKeyCode::L {
            if let Some(_) = ctx.forced_draw_mode {
                ctx.forced_draw_mode = None;
            } else {
                ctx.forced_draw_mode = Some(PolygonMode::Line);
            }
        }

        if keycode == VirtualKeyCode::O {
            ctx.gfx_context.light_uniforms.toggle_light();
        }

        if keycode == VirtualKeyCode::P {
            self.debug = !self.debug;
        }

        if keycode == VirtualKeyCode::R {
            ctx.reload_shaders();
        }

        if keycode == VirtualKeyCode::T {
            let terrain_size = if cfg!(debug_assertions) { 10 } else { 250 };
            self.entity_manager.terrain = generate_terrain(terrain_size, true, None);
            self.entity_manager.terrain.init(ctx);
        }

        if keycode == VirtualKeyCode::Y {
            let terrain_size = if cfg!(debug_assertions) { 10 } else { 250 };
            self.entity_manager.terrain = generate_terrain(terrain_size, false, None);
            self.entity_manager.terrain.init(ctx);
        }

        if keycode == VirtualKeyCode::N {
            if let Some(sun) = self.sun_mesh {
                self.entity_manager.set_sun_mesh(sun.cube);
                self.sun_mesh = None;
            } else {
                let mesh = self.entity_manager.get_sun_mesh();
                self.sun_mesh = Some(mesh);
                let n_mesh = core::entity::cube::Cuboid::cube(0.1, mesh.cube.position, None, None);
                self.entity_manager.set_sun_mesh(n_mesh);
            }
        }
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
        ctx.gfx_context.uniforms.view = self.camera.view_matrix();
    }
    */

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        // prevent degenerate case where things go wrong if resize while moving camera orientation
        self.mouse_down = false;

        println!("resize_event: width: {}, height: {}", width, height);
        self.entity_manager.camera.set_aspect((width, height));
        println!("new camera: {:#?}", self.entity_manager.camera);

        ctx.set_view(self.entity_manager.camera.update_view_matrix());
        ctx.set_projection(self.entity_manager.camera.update_projection_matrix());

        println!("view_matrix: {:#?}", self.entity_manager.camera.view);
    }

    fn key_mods_changed(&mut self, _ctx: &mut Context, _modifiers_state: ModifiersState) {}
}

#[allow(dead_code)]
fn get_corner_positions(row: f32, col: f32, y: f32) -> [(Vec3, Color, Vec3); 4] {
    [
        (
            (col, y, row).into(),
            Color::floats(0.3, 0.3, 0.3),
            (0, 1, 0).into(),
        ),
        (
            (col, y, row + 1.0).into(),
            Color::floats(0.4, 0.4, 0.4),
            (0, 1, 0).into(),
        ),
        (
            (col + 1.0, y, row).into(),
            Color::floats(0.5, 0.5, 0.5),
            (0, 1, 0).into(),
        ),
        (
            (col + 1.0, y, row + 1.0).into(),
            Color::floats(1., 1., 1.),
            (0, 1, 0).into(),
        ),
    ]
}

#[allow(dead_code)]
fn populate_grid(grid: &mut Vec<(Vec3, Color, Vec3)>, size: i32, y: f32) {
    for row in -size..size {
        for col in -size..size {
            let pos = get_corner_positions(row as f32, col as f32, y);
            grid.push(pos[0]);
            grid.push(pos[1]);
            grid.push(pos[3]);
            grid.push(pos[0]);
            grid.push(pos[2]);
            grid.push(pos[3]);
        }
    }
}

fn generate_terrain(
    terrain_size: usize,
    clamped: bool,
    seed: Option<isize>,
) -> core::entity::terrain::Terrain {
    let mut perlin = Perlin::default();
    if let Some(seed) = seed {
        perlin.seed = seed;
    }

    let color_gen = ColorGenerator::new(
        vec![
            (201, 178, 99).into(),
            (135, 184, 82).into(),
            (80, 171, 93).into(),
            (120, 120, 120).into(),
            (200, 200, 210).into(),
        ],
        0.45,
    );

    let terrain_gen = TerrainGenerator::new(perlin, color_gen);
    let mut terrain = terrain_gen.generate(terrain_size, clamped);
    terrain.center = (terrain_size as f32 / 2., 0., terrain_size as f32 / 2.).into();

    terrain
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
    //populate_grid(&mut grid, 50, 15.);
    println!(
        "{:#?}",
        (
            ctx.gfx_context.window_dims.width,
            ctx.gfx_context.window_dims.height,
        )
    );

    let terrain_size = if cfg!(debug_assertions) { 10 } else { 250 };
    let mut terrain = generate_terrain(terrain_size, false, Some(0));
    terrain.center = (terrain_size as f32 / 2., 0., terrain_size as f32 / 2.).into();
    terrain.init(&mut ctx);

    /*
    let test_bytes = include_bytes!("../../dog.png");
    let texture = pantheon::graphics::texture::Texture::from_bytes(
        &ctx.device,
        &ctx.queue,
        test_bytes,
        "dog",
    )
    .unwrap();
    */

    let top_right_ui = ui::TexturableQuad::new((0.5, 0.5).into(), (1.0, 1.0).into());

    let mut my_game = State {
        frame: 0,
        entity_manager: EntityManager::new(
            Camera::new(
                (20, 20, 20).into(),
                Vec3::new_from_one(0),
                (0, 1, 0).into(),
                90.0,
                (
                    ctx.gfx_context.window_dims.width,
                    ctx.gfx_context.window_dims.height,
                ),
                (1., 500.0),
            ),
            terrain,
        ),

        mouse_down: false,
        fps: 0.,
        network_client,
        network_queue: vec![],
        debug: false,
        sun_mesh: None,
        top_right_ui,
    };

    ctx.set_view(my_game.entity_manager.camera.view);
    ctx.set_projection(my_game.entity_manager.camera.projection);
    let cube = core::entity::cube::Cuboid::cube(5.0, (0, 10, 0).into(), None, None);
    my_game.entity_manager.push_entity(EntityKind::from(cube));

    pantheon::event::run(event_loop, ctx, my_game);
}
